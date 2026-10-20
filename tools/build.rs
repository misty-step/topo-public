use std::process::{Command, ExitCode};
fn main() -> ExitCode {
    // Product collection and rendering are native Rust. Bun is only used by the
    // optional design/json-render developer tooling, never by a distributed CLI.
    if Command::new("cargo")
        .args(["build", "--locked", "-j", "2"])
        .status()
        .is_ok_and(|s| s.success())
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
