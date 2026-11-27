//! `cargo xtask doctor`: checks that this machine can build and boot the project.
//!
//! Every check runs, in order, even after a failure; each item prints one line, and the exit
//! code is non-zero exactly when some line is `FAIL`. External commands go through [`Runner`]
//! so each check is unit-tested with canned output.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

mod boot;

pub(crate) use boot::run_with_deadline;

use crate::refs::{self, RefStatus};

/// The outcome of one item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    Ok,
    Warn,
    Skip,
    Fail,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Status::Ok => "ok",
            Status::Warn => "warn",
            Status::Skip => "skip",
            Status::Fail => "FAIL",
        })
    }
}

/// One printed line: `<status> <check> <item>: <detail>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Line {
    pub status: Status,
    pub check: &'static str,
    pub item: String,
    pub detail: String,
}

impl Line {
    fn new(status: Status, check: &'static str, item: &str, detail: impl Into<String>) -> Self {
        Line {
            status,
            check,
            item: item.to_string(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}: {}",
            self.status, self.check, self.item, self.detail
        )
    }
}

/// What a finished command printed: stdout, then stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Captured {
    pub success: bool,
    /// The exit code; `None` when a signal ended the process.
    pub code: Option<i32>,
    pub text: String,
}

/// Runs external commands, so tests can replace them with canned output.
pub(crate) trait Runner {
    /// Runs `program` and captures its output; `Err` when it cannot be started.
    fn capture(&self, program: &str, args: &[&str]) -> Result<Captured, String>;
}

/// The real [`Runner`]: starts the process.
#[derive(Debug)]
pub(crate) struct SystemRunner;

impl Runner for SystemRunner {
    fn capture(&self, program: &str, args: &[&str]) -> Result<Captured, String> {
        let out = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("cannot start `{program}`: {e}"))?;
        let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&out.stderr));
        Ok(Captured {
            success: out.status.success(),
            code: out.status.code(),
            text,
        })
    }
}

/// What earlier checks found, for the later checks that depend on it.
#[derive(Debug, Default)]
pub(crate) struct State {
    /// The version `qemu-system-x86_64` reported, when check 2 found it usable.
    qemu_x86: Option<String>,
    /// Whether check 2 found `qemu-system-aarch64` usable.
    qemu_aarch64: bool,
    /// Whether check 2 found `qemu-system-riscv64` usable.
    qemu_riscv64: bool,
    /// The firmware files check 4 found, by file name, with their full paths.
    firmware: BTreeMap<&'static str, PathBuf>,
}

impl State {
    /// Whether check 2 found the QEMU binary `program` usable.
    fn qemu_usable(&self, program: &str) -> bool {
        match program {
            "qemu-system-x86_64" => self.qemu_x86.is_some(),
            "qemu-system-aarch64" => self.qemu_aarch64,
            "qemu-system-riscv64" => self.qemu_riscv64,
            _ => false,
        }
    }
}

/// The leftmost version in `text`: `[0-9]+(\.[0-9]+)+`, or an eight-digit date.
fn parse_version(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let digits = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let first = digits(i);
        let mut end = i + first;
        let mut dotted = false;
        while bytes.get(end) == Some(&b'.') && digits(end + 1) > 0 {
            end += 1 + digits(end + 1);
            dotted = true;
        }
        if dotted {
            return Some(text[i..end].to_string());
        }
        if first >= 8 {
            return Some(text[i..i + 8].to_string());
        }
        i += first;
    }
    None
}

/// Whether the dotted `version` is at least `min` (missing parts count as 0).
fn at_least(version: &str, min: [u32; 3]) -> bool {
    let mut parts = [0u32; 3];
    for (slot, part) in parts.iter_mut().zip(version.split('.')) {
        *slot = part.parse().unwrap_or(0);
    }
    parts >= min
}

/// The directories QEMU searches, from `-L help` output; only those that exist.
fn search_dirs(text: &str) -> Vec<PathBuf> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .collect()
}

/// The `[toolchain]` table of `rust-toolchain.toml`.
#[derive(Debug, Default, serde::Deserialize)]
struct ToolchainFile {
    #[serde(default)]
    toolchain: Toolchain,
}

#[derive(Debug, Default, serde::Deserialize)]
struct Toolchain {
    channel: Option<String>,
    #[serde(default)]
    components: Vec<String>,
    #[serde(default)]
    targets: Vec<String>,
}

/// Whether `installed` (a `rustup component list --installed` line such as
/// `rustfmt-aarch64-apple-darwin`, or `rust-src`) is the component `name`.
fn is_component(installed: &str, name: &str) -> bool {
    let installed = installed.trim();
    installed == name
        || installed
            .strip_prefix(name)
            .and_then(|r| r.strip_prefix('-'))
            .is_some_and(|triple| triple.matches('-').count() >= 2)
}

/// The Homebrew formula and version argument of each host tool (spec section 4.3).
const HOST_TOOLS: [(&str, &str, &str); 10] = [
    ("qemu-system-x86_64", "--version", "qemu"),
    ("qemu-system-aarch64", "--version", "qemu"),
    ("qemu-system-riscv64", "--version", "qemu"),
    ("dtc", "--version", "dtc"),
    ("socat", "-V", "socat"),
    ("pdftotext", "-v", "poppler"),
    ("iasl", "-v", "acpica"),
    ("x86_64-elf-gdb", "--version", "x86_64-elf-gdb"),
    ("aarch64-elf-gdb", "--version", "aarch64-elf-gdb"),
    ("riscv64-elf-gdb", "--version", "riscv64-elf-gdb"),
];

const MIN_QEMU: [u32; 3] = [11, 1, 0];

const FIRMWARE: [&str; 4] = [
    "edk2-x86_64-code.fd",
    "edk2-aarch64-code.fd",
    "edk2-riscv-code.fd",
    "opensbi-riscv64-generic-fw_dynamic.bin",
];

/// Check 1: every component and target of `rust-toolchain.toml` is installed.
fn check_toolchain(runner: &dyn Runner, toolchain_file: Result<String, String>) -> Vec<Line> {
    const C: &str = "toolchain";
    let text = match toolchain_file {
        Ok(t) => t,
        Err(e) => return vec![Line::new(Status::Fail, C, "rust-toolchain.toml", e)],
    };
    let file = match toml::from_str::<ToolchainFile>(&text) {
        Ok(f) => f.toolchain,
        Err(e) => {
            return vec![Line::new(
                Status::Fail,
                C,
                "rust-toolchain.toml",
                format!("cannot parse: {e}"),
            )];
        }
    };
    let Some(channel) = file.channel else {
        return vec![Line::new(
            Status::Fail,
            C,
            "rust-toolchain.toml",
            "no `channel` found",
        )];
    };
    let mut lines = Vec::new();
    for (kind, wanted) in [("component", file.components), ("target", file.targets)] {
        let args = [kind, "list", "--installed", "--toolchain", &channel];
        let listing = match runner.capture("rustup", &args) {
            Ok(c) if c.success => Ok(c.text),
            Ok(c) => Err(format!(
                "`rustup {}` failed: {}",
                args.join(" "),
                c.text.trim()
            )),
            Err(e) => Err(e),
        };
        for name in wanted {
            lines.push(match &listing {
                Err(why) => Line::new(Status::Fail, C, &name, why),
                Ok(text) if text.lines().any(|l| is_installed(kind, l, &name)) => Line::new(
                    Status::Ok,
                    C,
                    &name,
                    format!("{kind} installed ({channel})"),
                ),
                Ok(_) => Line::new(
                    Status::Fail,
                    C,
                    &name,
                    format!("{kind} missing for {channel}; run `rustup show`"),
                ),
            });
        }
    }
    lines
}

/// Whether the `rustup <kind> list --installed` line `installed` is `name`.
fn is_installed(kind: &str, installed: &str, name: &str) -> bool {
    if kind == "target" {
        installed.trim() == name
    } else {
        is_component(installed, name)
    }
}

/// Check 2: each host tool runs and prints a version. Records QEMU's usability in `state`.
fn check_host_tools(runner: &dyn Runner, state: &mut State) -> Vec<Line> {
    const C: &str = "host-tools";
    HOST_TOOLS
        .iter()
        .map(|&(tool, arg, formula)| {
            let version = match runner.capture(tool, &[arg]) {
                Ok(c) if c.success => parse_version(&c.text).ok_or_else(|| {
                    format!("`{tool} {arg}` printed no version; try `brew reinstall {formula}`")
                }),
                Ok(c) => Err(format!(
                    "`{tool} {arg}` exited with failure ({}); try `brew reinstall {formula}`",
                    c.text.lines().next().unwrap_or("").trim()
                )),
                Err(e) => Err(format!("{e}; install with `brew install {formula}`")),
            };
            match version {
                Ok(v) => {
                    match tool {
                        "qemu-system-x86_64" => state.qemu_x86 = Some(v.clone()),
                        "qemu-system-aarch64" => state.qemu_aarch64 = true,
                        "qemu-system-riscv64" => state.qemu_riscv64 = true,
                        _ => {}
                    }
                    Line::new(Status::Ok, C, tool, v)
                }
                Err(e) => Line::new(Status::Fail, C, tool, e),
            }
        })
        .collect()
}

/// Check 3: QEMU is at least 11.1.0. The version is printed in every case.
fn check_qemu_version(state: &State) -> Vec<Line> {
    const C: &str = "qemu-version";
    const ITEM: &str = "qemu-system-x86_64";
    let Some(version) = &state.qemu_x86 else {
        return vec![Line::new(
            Status::Fail,
            C,
            ITEM,
            "not usable, see check host-tools",
        )];
    };
    vec![if at_least(version, MIN_QEMU) {
        Line::new(Status::Ok, C, ITEM, format!("{version} (minimum 11.1.0)"))
    } else {
        Line::new(
            Status::Fail,
            C,
            ITEM,
            format!("{version} is below the minimum 11.1.0; run `brew upgrade qemu`"),
        )
    }]
}

/// Check 4: the firmware files exist in a directory QEMU searches.
fn check_firmware(runner: &dyn Runner, state: &mut State) -> Vec<Line> {
    const C: &str = "firmware";
    let fail_all = |why: &str| -> Vec<Line> {
        FIRMWARE
            .iter()
            .map(|f| Line::new(Status::Fail, C, f, why))
            .collect()
    };
    if state.qemu_x86.is_none() {
        return fail_all("qemu-system-x86_64 not usable, see check host-tools");
    }
    let dirs = match runner.capture("qemu-system-x86_64", &["-L", "help"]) {
        Ok(c) if c.success => search_dirs(&c.text),
        Ok(c) => return fail_all(&format!("`-L help` failed: {}", c.text.trim())),
        Err(e) => return fail_all(&e),
    };
    FIRMWARE
        .iter()
        .map(|f| match dirs.iter().find(|d| d.join(f).is_file()) {
            Some(d) => {
                state.firmware.insert(f, d.join(f));
                Line::new(Status::Ok, C, f, d.join(f).display().to_string())
            }
            None => Line::new(
                Status::Fail,
                C,
                f,
                format!(
                    "not found in {} QEMU search director{}; run `brew reinstall qemu`",
                    dirs.len(),
                    if dirs.len() == 1 { "y" } else { "ies" }
                ),
            ),
        })
        .collect()
}

/// Check 5: on macOS arm64, `qemu-system-aarch64` offers `hvf`; elsewhere `skip`.
fn check_accelerator(runner: &dyn Runner, state: &State, mac_arm64: bool) -> Vec<Line> {
    const C: &str = "accelerator";
    const ITEM: &str = "hvf";
    if !mac_arm64 {
        return vec![Line::new(
            Status::Skip,
            C,
            ITEM,
            "xtask not built for macOS arm64",
        )];
    }
    if !state.qemu_aarch64 {
        return vec![Line::new(
            Status::Fail,
            C,
            ITEM,
            "qemu-system-aarch64 not usable, see check host-tools",
        )];
    }
    vec![
        match runner.capture("qemu-system-aarch64", &["-accel", "help"]) {
            Ok(c) if c.success && c.text.lines().any(|l| l.trim() == "hvf") => Line::new(
                Status::Ok,
                C,
                ITEM,
                "listed by qemu-system-aarch64 -accel help",
            ),
            Ok(c) if c.success => Line::new(
                Status::Fail,
                C,
                ITEM,
                "not listed by `qemu-system-aarch64 -accel help`",
            ),
            Ok(c) => Line::new(
                Status::Fail,
                C,
                ITEM,
                format!("`-accel help` failed: {}", c.text.trim()),
            ),
            Err(e) => Line::new(Status::Fail, C, ITEM, e),
        },
    ]
}

/// Check 7: the reference library. Never `FAIL`: a missing document only warns.
fn check_references(statuses: Result<Vec<(String, RefStatus)>, String>) -> Vec<Line> {
    const C: &str = "reference-library";
    const ITEM: &str = "manifest.toml";
    let statuses = match statuses {
        Ok(s) => s,
        Err(e) => return vec![Line::new(Status::Warn, C, ITEM, e)],
    };
    let count = |wanted: &RefStatus| statuses.iter().filter(|(_, s)| s == wanted).count();
    let (missing, changed, no_text) = (
        count(&RefStatus::Missing),
        count(&RefStatus::Changed),
        count(&RefStatus::NoText),
    );
    vec![if missing + changed + no_text == 0 {
        Line::new(
            Status::Ok,
            C,
            ITEM,
            format!("{} entries present", statuses.len()),
        )
    } else {
        Line::new(
            Status::Warn,
            C,
            ITEM,
            format!(
                "{missing} missing, {changed} changed, {no_text} no-text of {}; run `cargo xtask refs --fetch`",
                statuses.len()
            ),
        )
    }]
}

/// One check: reads and updates what earlier checks found.
type Check<'a> = &'a dyn Fn(&mut State) -> Vec<Line>;

/// Runs every check in order, prints each line with `print`, and is `Err` exactly when some
/// line is `FAIL`. A failure does not stop the later checks.
fn execute(checks: &[Check<'_>], print: &mut dyn FnMut(&Line)) -> Result<(), String> {
    let mut state = State::default();
    let mut failed = 0usize;
    for check in checks {
        for line in check(&mut state) {
            if line.status == Status::Fail {
                failed += 1;
            }
            print(&line);
        }
    }
    if failed == 0 {
        Ok(())
    } else {
        Err(format!("doctor: {failed} item(s) failed"))
    }
}

/// The repository root, from `git rev-parse --show-toplevel` (lesson xtask-1).
pub(crate) fn repository_root(runner: &dyn Runner) -> Result<PathBuf, String> {
    match runner.capture("git", &["rev-parse", "--show-toplevel"])? {
        c if c.success => Ok(PathBuf::from(c.text.trim())),
        c => Err(format!(
            "`git rev-parse --show-toplevel` failed: {}",
            c.text.trim()
        )),
    }
}

fn read_toolchain_file(root: &Result<PathBuf, String>) -> Result<String, String> {
    let root: &Path = root.as_deref().map_err(Clone::clone)?;
    std::fs::read_to_string(root.join("rust-toolchain.toml"))
        .map_err(|e| format!("cannot read rust-toolchain.toml: {e}"))
}

/// One named check; the name is the `check` column its lines print.
type NamedCheck<'a> = (&'static str, Box<dyn Fn(&mut State) -> Vec<Line> + 'a>);

/// Every check in the order of spec section 4.3.
fn checks<'a>(
    runner: &'a dyn Runner,
    root: &'a Result<PathBuf, String>,
    mac_arm64: bool,
) -> Vec<NamedCheck<'a>> {
    vec![
        (
            "toolchain",
            Box::new(move |_| check_toolchain(runner, read_toolchain_file(root))),
        ),
        ("host-tools", Box::new(move |s| check_host_tools(runner, s))),
        ("qemu-version", Box::new(|s| check_qemu_version(s))),
        ("firmware", Box::new(move |s| check_firmware(runner, s))),
        (
            "accelerator",
            Box::new(move |s| check_accelerator(runner, s, mac_arm64)),
        ),
        (
            "smoke-boot",
            Box::new(move |s| boot::check(s, root, mac_arm64, runner)),
        ),
        (
            "reference-library",
            Box::new(move |_| check_references(root.clone().and_then(|r| refs::statuses(&r)))),
        ),
    ]
}

/// Runs `cargo xtask doctor`.
pub(crate) fn run() -> Result<(), String> {
    let runner = SystemRunner;
    let root = repository_root(&runner);
    let mac_arm64 = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    let list = checks(&runner, &root, mac_arm64);
    let list: Vec<Check<'_>> = list
        .iter()
        .map(|(_, check)| &**check as Check<'_>)
        .collect();
    execute(&list, &mut |line| println!("{line}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Real outputs recorded in the chunk's `log.md` on 2026-11-27: (text, version).
    const FIXTURES: [(&str, &str); 6] = [
        (
            "QEMU emulator version 11.1.3\nCopyright (c) 2003-2026 Fabrice Bellard and the QEMU Project developers\n",
            "11.1.3",
        ),
        ("Version: DTC 1.8.1\n", "1.8.1"),
        (
            "socat by Gerhard Rieger and contributors - see www.dest-unreach.org\nsocat version 1.8.1.3 on \n   running on Darwin version Darwin Kernel Version 25.6.0: Mon Jul 14 2026\n",
            "1.8.1.3",
        ),
        (
            "pdftotext version 26.11.0\nCopyright 2005-2026 The Poppler Developers - http://poppler.freedesktop.org\n",
            "26.11.0",
        ),
        (
            "\nIntel ACPI Component Architecture\nASL+ Optimizing Compiler/Disassembler version 20260408\n",
            "20260408",
        ),
        (
            "GNU gdb (GDB) 18.1\nCopyright (C) 2026 Free Software Foundation, Inc.\n",
            "18.1",
        ),
    ];

    /// Canned output by program name; any other program is "cannot start".
    struct Fake(Vec<(&'static str, Result<Captured, String>)>);

    impl Runner for Fake {
        fn capture(&self, program: &str, _args: &[&str]) -> Result<Captured, String> {
            self.0.iter().find(|(p, _)| *p == program).map_or_else(
                || Err(format!("cannot start `{program}`")),
                |(_, r)| r.clone(),
            )
        }
    }

    /// A runner that must never be called.
    struct Panics;

    impl Runner for Panics {
        fn capture(&self, program: &str, _args: &[&str]) -> Result<Captured, String> {
            panic!("runner called for `{program}`");
        }
    }

    fn ok(text: &str) -> Captured {
        Captured {
            success: true,
            code: Some(0),
            text: text.to_string(),
        }
    }

    fn line(status: Status) -> Line {
        Line::new(status, "fake", "item", "detail")
    }

    #[test]
    fn parse_version_reads_each_real_output_with_lf_and_crlf() {
        for (text, want) in FIXTURES {
            assert_eq!(parse_version(text).as_deref(), Some(want), "LF: {text:?}");
            let crlf = text.replace('\n', "\r\n");
            assert_eq!(
                parse_version(&crlf).as_deref(),
                Some(want),
                "CRLF: {text:?}"
            );
        }
        assert_eq!(parse_version("no digits here"), None);
    }

    #[test]
    fn at_least_is_exact_at_the_qemu_minimum() {
        assert!(!at_least("11.0.9", MIN_QEMU));
        assert!(at_least("11.1.0", MIN_QEMU));
        assert!(at_least("11.1", MIN_QEMU));
        assert!(at_least("12.0.0", MIN_QEMU));
        assert!(!at_least("9.99.99", MIN_QEMU));
    }

    #[test]
    fn a_tool_that_exits_with_failure_is_fail() {
        let mut tools: Vec<(&'static str, Result<Captured, String>)> = HOST_TOOLS
            .iter()
            .map(|&(t, _, _)| (t, Ok(ok("version 1.2.3"))))
            .collect();
        tools[4].1 = Ok(Captured {
            success: false,
            code: Some(1),
            ..ok("version 1.2.3")
        });
        let lines = check_host_tools(&Fake(tools), &mut State::default());
        assert_eq!(lines.len(), 10);
        let failed: Vec<_> = lines.iter().filter(|l| l.status == Status::Fail).collect();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].item, "socat");
        assert!(failed[0].detail.contains("brew reinstall socat"));
    }

    #[test]
    fn a_tool_that_cannot_start_names_its_formula() {
        let lines = check_host_tools(&Fake(Vec::new()), &mut State::default());
        assert!(lines.iter().all(|l| l.status == Status::Fail));
        let iasl = lines.iter().find(|l| l.item == "iasl").unwrap();
        assert!(iasl.detail.contains("brew install acpica"));
    }

    #[test]
    fn search_dirs_drops_a_directory_that_does_not_exist() {
        let base = std::env::temp_dir().join(format!("xtask-doctor-{}", std::process::id()));
        std::fs::create_dir_all(base.join("bin")).unwrap();
        std::fs::create_dir_all(base.join("share/qemu")).unwrap();
        let b = base.display();
        let text = format!("{b}/bin/../share/qemu-firmware\r\n{b}/bin/../share/qemu\r\n");
        let dirs = search_dirs(&text);
        std::fs::remove_dir_all(&base).unwrap();
        assert_eq!(dirs, vec![PathBuf::from(format!("{b}/bin/../share/qemu"))]);
    }

    #[test]
    fn execute_continues_after_a_fail_and_errs_only_with_a_fail_line() {
        let ran = RefCell::new(Vec::new());
        let first = |_: &mut State| {
            ran.borrow_mut().push(1);
            vec![line(Status::Fail), line(Status::Ok)]
        };
        let second = |_: &mut State| {
            ran.borrow_mut().push(2);
            vec![line(Status::Ok)]
        };
        let mut printed = Vec::new();
        let result = execute(&[&first, &second], &mut |l| printed.push(l.clone()));
        assert!(result.is_err());
        assert_eq!(*ran.borrow(), vec![1, 2]);
        assert_eq!(printed.len(), 3);

        let calm = |_: &mut State| vec![line(Status::Warn), line(Status::Skip), line(Status::Ok)];
        assert_eq!(execute(&[&calm], &mut |_| {}), Ok(()));
    }

    #[test]
    fn line_display_is_status_check_item_detail() {
        assert_eq!(line(Status::Ok).to_string(), "ok fake item: detail");
        assert_eq!(line(Status::Fail).to_string(), "FAIL fake item: detail");
        assert_eq!(line(Status::Skip).to_string(), "skip fake item: detail");
        assert_eq!(line(Status::Warn).to_string(), "warn fake item: detail");
    }

    #[test]
    fn checks_whose_prerequisite_failed_fail_without_running_anything() {
        let mut state = State::default();
        let version = check_qemu_version(&state);
        assert_eq!(version.len(), 1);
        assert_eq!(version[0].status, Status::Fail);
        assert!(version[0].detail.contains("host-tools"));

        let firmware = check_firmware(&Panics, &mut state);
        assert_eq!(firmware.len(), FIRMWARE.len());
        assert!(firmware.iter().all(|l| l.status == Status::Fail));
        assert!(firmware.iter().all(|l| l.detail.contains("host-tools")));

        let accel = check_accelerator(&Panics, &state, true);
        assert_eq!(accel[0].status, Status::Fail);
        assert!(accel[0].detail.contains("host-tools"));
    }

    #[test]
    fn qemu_below_the_minimum_is_fail_and_prints_its_version() {
        let old = State {
            qemu_x86: Some("11.0.9".to_string()),
            ..State::default()
        };
        let lines = check_qemu_version(&old);
        assert_eq!(lines[0].status, Status::Fail);
        assert!(lines[0].detail.contains("11.0.9"));
        let new = State {
            qemu_x86: Some("11.1.0".to_string()),
            ..State::default()
        };
        assert_eq!(check_qemu_version(&new)[0].status, Status::Ok);
    }

    #[test]
    fn accelerator_is_skipped_off_mac_and_needs_hvf_on_mac() {
        let state = State {
            qemu_aarch64: true,
            ..State::default()
        };
        assert_eq!(
            check_accelerator(&Panics, &state, false)[0].status,
            Status::Skip
        );
        assert!(
            check_accelerator(&Panics, &state, false)[0]
                .detail
                .contains("xtask not built for macOS arm64")
        );
        let with = Fake(vec![(
            "qemu-system-aarch64",
            Ok(ok(
                "Accelerators supported in QEMU binary:\r\ntcg\r\nhvf\r\n",
            )),
        )]);
        assert_eq!(check_accelerator(&with, &state, true)[0].status, Status::Ok);
        let without = Fake(vec![(
            "qemu-system-aarch64",
            Ok(ok("Accelerators:\ntcg\n")),
        )]);
        assert_eq!(
            check_accelerator(&without, &state, true)[0].status,
            Status::Fail
        );
    }

    #[test]
    fn toolchain_reports_each_missing_component_and_target() {
        let file = "[toolchain]\r\nchannel = \"1.100.0\"\r\ncomponents = [\"rustfmt\", \"rust-src\", \"llvm-tools\"]\r\ntargets = [\"x86_64-unknown-none\", \"aarch64-unknown-none\"]\r\n";
        let runner = Fake(vec![(
            "rustup",
            Ok(ok(
                "rustfmt-aarch64-apple-darwin\nrust-src\nx86_64-unknown-none\n",
            )),
        )]);
        let lines = check_toolchain(&runner, Ok(file.to_string()));
        let status = |item: &str| lines.iter().find(|l| l.item == item).unwrap().status;
        assert_eq!(lines.len(), 5);
        assert_eq!(status("rustfmt"), Status::Ok);
        assert_eq!(status("rust-src"), Status::Ok);
        assert_eq!(status("llvm-tools"), Status::Fail);
        assert_eq!(status("x86_64-unknown-none"), Status::Ok);
        assert_eq!(status("aarch64-unknown-none"), Status::Fail);
    }

    #[test]
    fn parse_version_skips_leading_digits_that_are_not_the_version() {
        assert_eq!(parse_version("x86_64 gdb 18.1").as_deref(), Some("18.1"));
        assert_eq!(
            parse_version("build 7, version 1.2.3").as_deref(),
            Some("1.2.3")
        );
    }

    /// Replies per program with `(success, text)`; `Err` for a program not listed.
    struct ByProgram(Vec<(&'static str, bool, &'static str)>);

    impl Runner for ByProgram {
        fn capture(&self, program: &str, _args: &[&str]) -> Result<Captured, String> {
            self.0
                .iter()
                .find(|(p, ..)| *p == program)
                .map(|&(_, success, text)| Captured {
                    success,
                    code: Some(i32::from(!success)),
                    text: text.to_string(),
                })
                .ok_or_else(|| format!("cannot start `{program}`"))
        }
    }

    #[test]
    fn host_tools_record_each_qemu_in_its_own_field() {
        let tools = |aarch64_ok: bool| {
            ByProgram(
                HOST_TOOLS
                    .iter()
                    .map(|&(t, ..)| match t {
                        "qemu-system-x86_64" => (t, true, "QEMU emulator version 11.1.3"),
                        "qemu-system-aarch64" => (t, aarch64_ok, "QEMU emulator version 11.1.2"),
                        "qemu-system-riscv64" => (t, true, "QEMU emulator version 11.1.1"),
                        _ => (t, true, "tool 1.0.0"),
                    })
                    .collect(),
            )
        };
        let mut state = State::default();
        check_host_tools(&tools(true), &mut state);
        assert_eq!(state.qemu_x86.as_deref(), Some("11.1.3"));
        assert!(state.qemu_aarch64 && state.qemu_riscv64);

        let mut state = State::default();
        let lines = check_host_tools(&tools(false), &mut state);
        assert_eq!(state.qemu_x86.as_deref(), Some("11.1.3"));
        assert!(!state.qemu_aarch64, "aarch64 failed");
        assert!(state.qemu_riscv64);
        assert_eq!(lines.iter().filter(|l| l.status == Status::Fail).count(), 1);
    }

    #[test]
    fn firmware_found_in_a_search_directory_is_ok_with_its_path_and_the_rest_fail() {
        let dir = std::env::temp_dir().join(format!("xtask-fw-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&dir));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("edk2-x86_64-code.fd"), "").unwrap();
        std::fs::write(dir.join("opensbi-riscv64-generic-fw_dynamic.bin"), "").unwrap();
        let listing = format!("/no/such/dir\n{}\n", dir.display());
        let runner = ByProgram(vec![(
            "qemu-system-x86_64",
            true,
            Box::leak(listing.into_boxed_str()),
        )]);
        let mut state = State {
            qemu_x86: Some("11.1.3".into()),
            ..State::default()
        };
        let lines = check_firmware(&runner, &mut state);
        let text: Vec<String> = lines.iter().map(ToString::to_string).collect();
        assert_eq!(text.len(), 4, "{text:?}");
        let p = |f: &str| dir.join(f).display().to_string();
        assert_eq!(
            text[0],
            format!(
                "ok firmware edk2-x86_64-code.fd: {}",
                p("edk2-x86_64-code.fd")
            )
        );
        assert!(
            text[1].starts_with("FAIL firmware edk2-aarch64-code.fd:"),
            "{text:?}"
        );
        assert!(text[1].contains("1 QEMU search directory"), "{text:?}");
        assert!(
            text[2].starts_with("FAIL firmware edk2-riscv-code.fd:"),
            "{text:?}"
        );
        assert_eq!(
            text[3],
            format!(
                "ok firmware opensbi-riscv64-generic-fw_dynamic.bin: {}",
                p("opensbi-riscv64-generic-fw_dynamic.bin")
            )
        );
        let found: Vec<_> = state.firmware.keys().copied().collect();
        assert_eq!(
            found,
            [
                "edk2-x86_64-code.fd",
                "opensbi-riscv64-generic-fw_dynamic.bin"
            ]
        );
        drop(std::fs::remove_dir_all(&dir));
    }

    #[test]
    fn firmware_check_fails_every_file_when_l_help_fails() {
        for runner in [
            ByProgram(vec![("qemu-system-x86_64", false, "bad option")]),
            ByProgram(vec![]),
        ] {
            let mut state = State {
                qemu_x86: Some("11.1.3".into()),
                ..State::default()
            };
            let lines = check_firmware(&runner, &mut state);
            assert_eq!(lines.len(), FIRMWARE.len());
            assert!(lines.iter().all(|l| l.status == Status::Fail), "{lines:?}");
            assert!(state.firmware.is_empty());
        }
        let mut state = State {
            qemu_x86: Some("11.1.3".into()),
            ..State::default()
        };
        let failed = check_firmware(
            &ByProgram(vec![("qemu-system-x86_64", false, "bad option")]),
            &mut state,
        );
        assert!(
            failed[0].detail.contains("`-L help` failed: bad option"),
            "{}",
            failed[0]
        );
    }

    /// Replies to `rustup component list` and `rustup target list` differently, and checks the
    /// full argument list.
    struct Rustup {
        component: Result<Captured, String>,
        target: Result<Captured, String>,
    }

    impl Runner for Rustup {
        fn capture(&self, program: &str, args: &[&str]) -> Result<Captured, String> {
            assert_eq!(program, "rustup");
            assert_eq!(
                &args[1..],
                ["list", "--installed", "--toolchain", "1.100.0"]
            );
            match args[0] {
                "component" => self.component.clone(),
                "target" => self.target.clone(),
                other => panic!("unexpected rustup {other}"),
            }
        }
    }

    const SMALL_TOOLCHAIN: &str = "[toolchain]\nchannel = \"1.100.0\"\ncomponents = [\"rust-src\"]\ntargets = [\"x86_64-unknown-none\"]\n";

    #[test]
    fn toolchain_reads_components_and_targets_from_their_own_listings() {
        let components = ok("rust-src\nrust-std-x86_64-unknown-none\n");
        let both = Rustup {
            component: Ok(components.clone()),
            target: Ok(ok("x86_64-unknown-none\n")),
        };
        let lines = check_toolchain(&both, Ok(SMALL_TOOLCHAIN.to_string()));
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| l.status == Status::Ok), "{lines:?}");
        assert_eq!(lines[0].item, "rust-src");
        // The component listing names the target's rust-std, but the target listing is empty.
        let no_target = Rustup {
            component: Ok(components),
            target: Ok(ok("")),
        };
        let lines = check_toolchain(&no_target, Ok(SMALL_TOOLCHAIN.to_string()));
        assert_eq!(lines[0].status, Status::Ok);
        assert_eq!(lines[1].item, "x86_64-unknown-none");
        assert_eq!(lines[1].status, Status::Fail);
    }

    #[test]
    fn toolchain_fails_every_item_when_rustup_fails() {
        let failed = Captured {
            success: false,
            code: Some(1),
            text: "rustup: broken\n".into(),
        };
        for r in [Ok(failed), Err("cannot start `rustup`".to_string())] {
            let runner = Rustup {
                component: r.clone(),
                target: r,
            };
            let lines = check_toolchain(&runner, Ok(SMALL_TOOLCHAIN.to_string()));
            assert_eq!(lines.len(), 2);
            assert!(lines.iter().all(|l| l.status == Status::Fail), "{lines:?}");
            assert!(
                lines.iter().all(|l| l.detail.contains("rustup")),
                "{lines:?}"
            );
        }
    }

    #[test]
    fn toolchain_without_a_channel_is_one_fail_and_runs_nothing() {
        let lines = check_toolchain(
            &Panics,
            Ok("[toolchain]\ncomponents = [\"rustfmt\"]\n".into()),
        );
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].status, Status::Fail);
        assert!(lines[0].detail.contains("no `channel`"), "{}", lines[0]);
    }

    #[test]
    fn the_checks_run_in_the_order_of_spec_section_4_3() {
        // Inputs that start nothing real: no program starts, and the root is unknown.
        let root = Err("unused".to_string());
        let runner = Fake(vec![]);
        let list = checks(&runner, &root, true);
        let order = [
            "toolchain",
            "host-tools",
            "qemu-version",
            "firmware",
            "accelerator",
            "smoke-boot",
            "reference-library",
        ];
        let names: Vec<&str> = list.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, order);
        let mut state = State::default();
        for (name, check) in &list {
            let lines = check(&mut state);
            assert!(!lines.is_empty(), "{name}");
            for l in &lines {
                assert_eq!(l.check, *name, "{l}");
            }
        }
    }

    fn references(statuses: &[RefStatus]) -> Vec<Line> {
        check_references(Ok(statuses
            .iter()
            .enumerate()
            .map(|(i, s)| (format!("d{i}"), s.clone()))
            .collect()))
    }

    #[test]
    fn reference_statuses_missing_changed_and_no_text_warn() {
        for s in [RefStatus::Missing, RefStatus::Changed, RefStatus::NoText] {
            let lines = references(&[RefStatus::Ok, s.clone()]);
            assert_eq!(lines.len(), 1, "{s:?}");
            assert_eq!(lines[0].status, Status::Warn, "{s:?}");
            assert!(
                lines[0].detail.contains("cargo xtask refs --fetch"),
                "{s:?}"
            );
        }
    }

    #[test]
    fn reference_statuses_ok_and_unpinned_are_ok() {
        for s in [RefStatus::Ok, RefStatus::Unpinned("x".into())] {
            assert_eq!(
                references(std::slice::from_ref(&s))[0].status,
                Status::Ok,
                "{s:?}"
            );
        }
        assert_eq!(references(&[])[0].status, Status::Ok);
    }

    #[test]
    fn an_unreadable_manifest_warns_and_never_fails() {
        let lines = check_references(Err("cannot read".into()));
        assert_eq!(lines[0].status, Status::Warn);
        assert!(lines[0].detail.contains("cannot read"));
    }

    #[test]
    fn reference_warning_counts_missing_and_changed() {
        let lines = references(&[
            RefStatus::Missing,
            RefStatus::Missing,
            RefStatus::Changed,
            RefStatus::NoText,
        ]);
        let d = &lines[0].detail;
        assert!(
            d.contains("2 missing") && d.contains("1 changed") && d.contains("1 no-text"),
            "{d}"
        );
    }
}
