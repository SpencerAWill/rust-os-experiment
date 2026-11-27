#![forbid(unsafe_code)]
// developer tool
#![allow(clippy::print_stdout)]

//! `cargo xtask <command>`: the single entry point that CI and humans both run.

mod check_docs;
mod check_workflows;
mod cmd;
mod doctor;
mod guard_push;

use std::process::ExitCode;

use cmd::{require_tool, run};

/// A parsed `cargo xtask` subcommand.
#[derive(Debug, PartialEq, Eq)]
enum Command {
    Fmt {
        check: bool,
    },
    Clippy,
    Test {
        release: bool,
        include_ignored: bool,
    },
    Doc,
    Deny,
    Typos,
    CheckWorkflows,
    CheckDocs,
    Doctor,
    GuardPush,
    Ci {
        quick: bool,
    },
}

/// Parses command-line arguments (without the program name).
fn parse(args: &[&str]) -> Result<Command, String> {
    let (name, flags) = args.split_first().ok_or_else(|| {
        "missing command (try: ci [--quick], fmt [--check], clippy, test, doc, deny, typos, check-workflows, check-docs, doctor)"
            .to_string()
    })?;
    let has = |flag: &str| flags.contains(&flag);
    let only = |allowed: &[&str]| -> Result<(), String> {
        match flags.iter().find(|f| !allowed.contains(f)) {
            Some(bad) => Err(format!("unknown flag `{bad}` for `{name}`")),
            None => Ok(()),
        }
    };
    match *name {
        "fmt" => only(&["--check"]).map(|()| Command::Fmt {
            check: has("--check"),
        }),
        "clippy" => only(&[]).map(|()| Command::Clippy),
        "test" => only(&["--release", "--include-ignored"]).map(|()| Command::Test {
            release: has("--release"),
            include_ignored: has("--include-ignored"),
        }),
        "doc" => only(&[]).map(|()| Command::Doc),
        "deny" => only(&[]).map(|()| Command::Deny),
        "typos" => only(&[]).map(|()| Command::Typos),
        "check-workflows" => only(&[]).map(|()| Command::CheckWorkflows),
        "check-docs" => only(&[]).map(|()| Command::CheckDocs),
        "doctor" => only(&[]).map(|()| Command::Doctor),
        "guard-push" => only(&[]).map(|()| Command::GuardPush),
        "ci" => only(&["--quick"]).map(|()| Command::Ci {
            quick: has("--quick"),
        }),
        other => Err(format!("unknown command `{other}`")),
    }
}

fn run_tests(release: bool, include_ignored: bool) -> Result<(), String> {
    require_tool("cargo-nextest", "cargo install --locked cargo-nextest")?;
    let mut args = vec!["nextest", "run", "--workspace", "--all-features"];
    if release {
        args.push("--release");
    }
    if include_ignored {
        args.extend(["--run-ignored", "all"]);
    }
    run("cargo", &args, &[])?;
    if workspace_has_library()? {
        run(
            "cargo",
            &["test", "--doc", "--workspace", "--all-features"],
            &[],
        )
    } else {
        // `cargo test --doc` fails outright on a workspace with no library target.
        println!("no library targets yet: skipping doctests");
        Ok(())
    }
}

/// Whether any workspace member has a library target (doctests need one).
fn workspace_has_library() -> Result<bool, String> {
    let out = std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|e| format!("failed to run cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err("cargo metadata failed".to_string());
    }
    let meta: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata output: {e}"))?;
    Ok(has_library_target(&meta))
}

/// Whether `cargo metadata` output lists a `lib` target in any package.
fn has_library_target(meta: &serde_json::Value) -> bool {
    meta["packages"].as_array().is_some_and(|packages| {
        packages.iter().any(|p| {
            p["targets"].as_array().is_some_and(|targets| {
                targets.iter().any(|t| {
                    t["kind"]
                        .as_array()
                        .is_some_and(|k| k.iter().any(|kind| kind == "lib"))
                })
            })
        })
    })
}

/// The steps of `cargo xtask ci`, in order.
fn ci_steps(quick: bool) -> Vec<Command> {
    let mut steps = vec![
        Command::Fmt { check: true },
        Command::Clippy,
        Command::Test {
            release: false,
            include_ignored: false,
        },
    ];
    if !quick {
        steps.extend([
            Command::Test {
                release: true,
                include_ignored: true,
            },
            Command::Doc,
            Command::CheckWorkflows,
            Command::CheckDocs,
            Command::Typos,
            Command::Deny,
        ]);
    }
    steps
}

fn execute(command: &Command) -> Result<(), String> {
    match command {
        Command::Fmt { check } => {
            let mut args = vec!["fmt", "--all"];
            if *check {
                args.extend(["--", "--check"]);
            }
            run("cargo", &args, &[])
        }
        Command::Clippy => run(
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
            &[],
        ),
        Command::Test {
            release,
            include_ignored,
        } => run_tests(*release, *include_ignored),
        Command::Doc => run(
            "cargo",
            &["doc", "--workspace", "--no-deps", "--all-features"],
            &[("RUSTDOCFLAGS", "-D warnings")],
        ),
        Command::Deny => {
            require_tool("cargo-deny", "cargo install --locked cargo-deny")?;
            run("cargo", &["deny", "check"], &[])
        }
        Command::Typos => {
            require_tool("typos", "cargo install --locked typos-cli")?;
            run("typos", &[], &[])
        }
        Command::CheckWorkflows => check_workflows::run(),
        Command::CheckDocs => check_docs::run(),
        Command::Doctor => doctor::run(),
        // `main` runs this one directly, since it exits with a code other than 1.
        Command::GuardPush => Err("guard-push is run by `main`".to_string()),
        Command::Ci { quick } => ci_steps(*quick).iter().try_for_each(execute),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    // The push guard exits 2 to block, which Claude Code treats as "refuse", so it bypasses the
    // `Result` path below (which exits 1).
    if args.first() == Some(&"guard-push") && parse(&args) == Ok(Command::GuardPush) {
        return guard_push::run();
    }
    match parse(&args).and_then(|c| execute(&c)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("xtask: {msg}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_commands() {
        assert_eq!(parse(&["fmt", "--check"]), Ok(Command::Fmt { check: true }));
        assert_eq!(parse(&["ci", "--quick"]), Ok(Command::Ci { quick: true }));
        assert_eq!(parse(&["check-docs"]), Ok(Command::CheckDocs));
        assert_eq!(parse(&["guard-push"]), Ok(Command::GuardPush));
        assert!(parse(&["test", "--bogus"]).is_err());
    }

    #[test]
    fn doctor_takes_no_flags() {
        assert_eq!(parse(&["doctor"]), Ok(Command::Doctor));
        assert!(parse(&["doctor", "--x"]).is_err());
    }

    #[test]
    fn library_targets_are_detected_from_cargo_metadata() {
        let bin_only = serde_json::json!({"packages": [{"targets": [{"kind": ["bin"]}]}]});
        let with_lib = serde_json::json!({"packages": [
            {"targets": [{"kind": ["bin"]}]},
            {"targets": [{"kind": ["lib"]}]}
        ]});
        assert!(!has_library_target(&bin_only));
        assert!(has_library_target(&with_lib));
    }

    #[test]
    fn dropped_commands_are_unknown() {
        for name in ["bench", "mutants", "no-std", "fuzz", "coverage"] {
            assert!(parse(&[name]).is_err(), "{name} should be unknown");
        }
    }

    #[test]
    fn ci_steps_are_the_ported_set() {
        let quick = vec![
            Command::Fmt { check: true },
            Command::Clippy,
            Command::Test {
                release: false,
                include_ignored: false,
            },
        ];
        assert_eq!(ci_steps(true), quick);
        let mut full = quick;
        full.extend([
            Command::Test {
                release: true,
                include_ignored: true,
            },
            Command::Doc,
            Command::CheckWorkflows,
            Command::CheckDocs,
            Command::Typos,
            Command::Deny,
        ]);
        assert_eq!(ci_steps(false), full);
    }
}
