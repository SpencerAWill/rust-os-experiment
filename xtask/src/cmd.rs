//! Helpers for running cargo and external tool commands.

use std::process::{Command, Stdio};

/// Runs `program` with `args` and extra environment variables, streaming its output.
///
/// Returns `Err` with the failing command line if the process cannot be started
/// or exits unsuccessfully.
pub(crate) fn run(program: &str, args: &[&str], env: &[(&str, &str)]) -> Result<(), String> {
    run_with(None, program, args, env)
}

fn run_with(
    dir: Option<&str>,
    program: &str,
    args: &[&str],
    env: &[(&str, &str)],
) -> Result<(), String> {
    let line = format!("{program} {}", args.join(" "));
    match dir {
        Some(dir) => println!("$ (in {dir}) {line}"),
        None => println!("$ {line}"),
    }
    let mut command = Command::new(program);
    command.args(args).envs(env.iter().copied());
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    let status = command
        .status()
        .map_err(|e| format!("failed to start `{line}`: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed ({status}): {line}"))
    }
}

/// Checks that `binary` is installed, returning `install_hint` if it is not.
pub(crate) fn require_tool(binary: &str, install_hint: &str) -> Result<(), String> {
    let found = Command::new(binary)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if found {
        Ok(())
    } else {
        Err(format!("{binary} missing: run '{install_hint}'"))
    }
}
