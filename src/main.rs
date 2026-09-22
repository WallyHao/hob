// --- hob binary ---

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out = io::stdout().lock();
    let mut err = io::stderr().lock();
    ExitCode::from(hob::run(&args, &mut out, &mut err))
}
