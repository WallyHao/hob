// --- exec::proc::run ---
// Spawning a command in a session. Output is drained on separate threads while
// the child runs, because a child that fills a pipe while its parent waits for
// it to exit would deadlock. Every child becomes its own process group, so a
// timeout or an interrupt can kill the tree the command started.

use std::io::Write as _;
use std::process::{Command, Stdio};
use std::time::Instant;

use serde_json::{Value, json};

use crate::effect::Failure;
use crate::effect::ops::proc::Options;
use crate::exec::State;
use crate::exec::proc::Session;
use crate::exec::proc::children::kill_group;
use crate::exec::proc::spawn::{TIMEOUT_CODE, Waited, drain, trim, wait};

/// Run one program in a session.
pub(crate) fn program(
    state: &State,
    session: &Session,
    argv: &[String],
    options: &Options,
) -> Result<Value, Failure> {
    let Some((head, tail)) = argv.split_first() else {
        return Err(Failure::new("`proc.exec` needs a program"));
    };
    let mut command = Command::new(head);
    command.args(tail);
    finish(state, session, &mut command, options)
}

/// Run one shell line in a session, after its profile.
pub(crate) fn shell(
    state: &State,
    session: &Session,
    line: &str,
    options: &Options,
) -> Result<Value, Failure> {
    let script = match session.profile() {
        Some(profile) => format!("{profile}\n{line}"),
        None => line.to_owned(),
    };
    let mut command = Command::new("sh");
    command.arg("-c").arg(script);
    finish(state, session, &mut command, options)
}

/// Apply the context, spawn, wait and turn the result into a value.
fn finish(
    state: &State,
    session: &Session,
    command: &mut Command,
    options: &Options,
) -> Result<Value, Failure> {
    command.current_dir(session.cwd());
    if options.env_clear || session.env_clear() {
        command.env_clear();
    }
    command.envs(session.env());
    own_group(command);
    if options.inherit {
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
    } else {
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
    }
    let program = command.get_program().to_string_lossy().into_owned();
    let started = Instant::now();
    let mut child = command
        .spawn()
        .map_err(|error| Failure::new(format!("cannot run `{program}`: {error}")))?;
    let _tracked = state.children.track(child.id());
    // Every pipe is taken before waiting so each has one owner: the drains must
    // read while the child runs, or a child that fills a pipe before reading
    // its input would deadlock both sides.
    let stdin = child.stdin.take();
    let feed = if options.inherit {
        None
    } else {
        options.stdin.clone()
    };
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (waited, out, err) = std::thread::scope(|scope| -> Result<_, Failure> {
        // Feeding input on its own thread keeps the order free; dropping the
        // pipe when `feed` is `None` is what tells the child input ended.
        let writer = stdin.and_then(|mut pipe| {
            feed.map(|text| {
                scope.spawn(move || {
                    let _ = pipe.write_all(text.as_bytes());
                })
            })
        });
        let out_handle = stdout.map(|pipe| scope.spawn(move || drain(pipe)));
        let err_handle = stderr.map(|pipe| scope.spawn(move || drain(pipe)));
        let waited = wait(&mut child, options.timeout_ms);
        if waited.is_err() {
            // Nothing else unblocks the writer when the wait itself failed and
            // the child is still alive with a full input pipe.
            kill_group(child.id());
            let _ = child.wait();
        }
        let out = out_handle
            .map(|handle| handle.join().unwrap_or_default())
            .unwrap_or_default();
        let err = err_handle
            .map(|handle| handle.join().unwrap_or_default())
            .unwrap_or_default();
        if let Some(writer) = writer {
            let _ = writer.join();
        }
        Ok((waited?, out, err))
    })?;
    let code = match waited {
        Waited::Exited(status) => status.code().unwrap_or(-1),
        Waited::TimedOut => TIMEOUT_CODE,
    };
    Ok(json!({
        "code": code,
        "stdout": trim(out.0, options.trim),
        "stderr": trim(err.0, options.trim),
        "ok": code == 0,
        "duration_ms": u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        "truncated": out.1 || err.1,
    }))
}

/// Put the child at the head of its own process group, so a signal can reach the
/// whole tree it starts and not the run that spawned it.
#[cfg(unix)]
fn own_group(command: &mut Command) {
    use std::os::unix::process::CommandExt as _;
    command.process_group(0);
}

#[cfg(not(unix))]
fn own_group(_command: &mut Command) {}
