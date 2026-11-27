# xtask/

Status: Living (2026-11-27). The task runner: `cargo xtask <command>` is the single entry point for CI and local runs.

## When this applies

- You run a check.
- You change a command, or a check that CI runs.

## Contents

- `Cargo.toml`: the `xtask` package (not published).
- `src/`: `main.rs` parses the command line and holds the `ci` steps; `cmd.rs` runs external tools; `check_docs.rs` (and `check_docs/`), `check_workflows.rs` and `guard_push.rs` are the checks, ported from `rust-engine-experiment`; `doctor.rs` (and `doctor/`) and `refs.rs` check and set up the host.

## Commands

Every command rejects flags it does not know, and prints an install hint when a tool it needs is missing.

| Command | What it does |
|---|---|
| `ci [--quick]` | Every check CI gates on, in order. `--quick`: fmt check, clippy and tests only |
| `fmt [--check]`, `clippy`, `doc` | Formatting, clippy with warnings denied, docs without warnings |
| `test [--release] [--include-ignored]` | nextest plus doctests |
| `deny` | `cargo deny`: licences, advisories, sources |
| `typos` | Spell-check |
| `check-workflows` | Every GitHub Action pinned to a commit hash, read-only token permissions (a `write` scope only for a job in `WRITE_ALLOWLIST`), no persisted checkout credentials, no `pull_request_target` |
| `check-docs` | Required READMEs and their Contents, relative links and anchors, lesson and rule files, the length of `AGENTS.md` (the conventions are in [docs/README.md](../docs/README.md#conventions)) |
| `doctor` | Checks the toolchain, host tools, QEMU version, firmware and accelerator, smoke-boots each architecture's firmware (logs in `target/doctor/`), and reports the reference library; non-zero exit on any `FAIL` line |
| `refs [--fetch]` | The reference library's state per entry (offline); `--fetch` downloads missing or changed entries over https, checks each SHA-256 and writes their text ([reference/README.md](../reference/README.md)) |
| `guard-push` | The push guard behind `.claude/hooks/guard-push.sh`; reads a hook's JSON on stdin |

Commands to build, run and test the kernels under QEMU arrive with M2.
