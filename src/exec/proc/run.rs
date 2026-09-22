// --- exec::proc::run ---
// Spawning a command in a session. Output is drained on separate threads while
// the child runs, because a child that fills a pipe while its parent waits for
// it to exit would deadlock.

use std::io::Write as _;
use std::process::{Command, Stdio};
use std::time::Instant;

use serde_json::{Value, json};

use crate::effect::Failure;
use crate::effect::ops::proc::Options;
use crate::exec::proc::Session;
use crate::exec::proc::spawn::{TIMEOUT_CODE, Waited, drain, trim, wait};

/// Run one program in a session.
pub(crate) fn program(
    session: &Session,
    argv: &[String],
    options: &Options,
) -> Result<Value, Failure> {
    let Some((head, tail)) = argv.split_first() else {
        return Err(Failure::new("`proc.exec` needs a program"));
    };
    let mut command = Command::new(head);
    command.args(tail);
    finish(session, &mut command, options)
}

/// Run one shell line in a session, after its profile.
pub(crate) fn shell(session: &Session, line: &str, options: &Options) -> Result<Value, Failure> {
    let script = match session.profile() {
        Some(profile) => format!("{profile}\n{line}"),
        None => line.to_owned(),
    };
    let mut command = Command::new("sh");
    command.arg("-c").arg(script);
    finish(session, &mut command, options)
}

/// Apply the context, spawn, wait and turn the result into a value.
fn finish(session: &Session, command: &mut Command, options: &Options) -> Result<Value, Failure> {
    command.current_dir(session.cwd()).envs(session.env());
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
    if !options.inherit
        && let Some(text) = &options.stdin
        && let Some(mut pipe) = child.stdin.take()
    {
        let _ = pipe.write_all(text.as_bytes());
        // Dropping the pipe here is what tells the child input ended.
    }
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (waited, out, err) = std::thread::scope(|scope| -> Result<_, Failure> {
        let out_handle = stdout.map(|pipe| scope.spawn(move || drain(pipe)));
        let err_handle = stderr.map(|pipe| scope.spawn(move || drain(pipe)));
        let waited = wait(&mut child, options.timeout_ms)?;
        let out = out_handle
            .map(|handle| handle.join().unwrap_or_default())
            .unwrap_or_default();
        let err = err_handle
            .map(|handle| handle.join().unwrap_or_default())
            .unwrap_or_default();
        Ok((waited, out, err))
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
