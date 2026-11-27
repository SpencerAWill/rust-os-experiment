//! Check 6 of `doctor`: smoke-boot each architecture's firmware under QEMU (spec section 4.3).

use super::{Line, Runner, State, Status};
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const C: &str = "smoke-boot";

/// Flags every boot runs with.
const COMMON: [&str; 8] = [
    "-nographic",
    "-monitor",
    "none",
    "-no-reboot",
    "-m",
    "256M",
    "-nic",
    "none",
];

/// Kills and reaps its child when dropped, on every path including a panic.
pub(super) struct ChildGuard(pub(super) Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Runs `cmd` and waits for it for at most `deadline`; past it the child is killed and reaped.
pub(crate) fn run_with_deadline(
    mut cmd: Command,
    deadline: Duration,
) -> Result<ExitStatus, String> {
    let program = cmd.get_program().to_string_lossy().into_owned();
    let mut guard = ChildGuard(
        cmd.spawn()
            .map_err(|e| format!("cannot start `{program}`: {e}"))?,
    );
    let end = Instant::now() + deadline;
    loop {
        match guard.0.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() >= end => {
                // The guard kills and reaps the child as it drops.
                return Err(format!(
                    "`{program}` did not finish within {} s and was killed",
                    deadline.as_secs()
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(e) => return Err(format!("cannot wait for `{program}`: {e}")),
        }
    }
}

/// The leftover tagged QEMU processes in `pgrep -f -l` output: pid and program basename.
fn leftovers(pgrep_output: &str) -> Vec<(u32, String)> {
    pgrep_output
        .lines()
        .filter_map(|line| {
            let (pid, command) = line.trim().split_once(' ')?;
            let pid = pid.parse().ok()?;
            let program = command.split_whitespace().next()?;
            let base = program.rsplit('/').next()?;
            (base.starts_with("qemu-system-") && command.contains("-name xtask-doctor-"))
                .then(|| (pid, base.to_string()))
        })
        .collect()
}

/// Kills every QEMU left by an earlier `doctor` (spec section 4.3, amendment of 2026-11-27).
fn sweep(runner: &dyn Runner) -> Vec<Line> {
    sweep_within(runner, Duration::from_secs(5))
}

fn sweep_within(runner: &dyn Runner, wait: Duration) -> Vec<Line> {
    const ITEM: &str = "leftover";
    let warn = |detail: String| Line::new(Status::Warn, C, ITEM, detail);
    let found = match runner.capture("pgrep", &["-f", "-l", "xtask-doctor-"]) {
        Ok(c) if c.success => leftovers(&c.text),
        Ok(c) if c.code == Some(1) => return Vec::new(),
        Ok(c) => return vec![warn(format!("`pgrep` failed: {}", c.text.trim()))],
        Err(e) => return vec![warn(e)],
    };
    found
        .into_iter()
        .map(|(pid, program)| {
            let id = pid.to_string();
            match runner.capture("kill", &["-KILL", &id]) {
                Ok(c) if c.success => {}
                Ok(c) => return warn(format!("`kill` failed for pid {pid}: {}", c.text.trim())),
                Err(e) => return warn(e),
            }
            let end = Instant::now() + wait;
            loop {
                if !runner
                    .capture("kill", &["-0", &id])
                    .is_ok_and(|c| c.success)
                {
                    return warn(format!("killed pid {pid} ({program})"));
                }
                if Instant::now() >= end {
                    return warn(format!("pid {pid} ({program}) still running"));
                }
                thread::sleep(Duration::from_millis(100));
            }
        })
        .collect()
}

/// One smoke boot: a QEMU command, the serial marker to wait for, and the deadline.
pub(super) struct Boot {
    pub name: &'static str,
    pub program: &'static str,
    /// Arguments beyond the common flags; `{fw}` is replaced by the firmware path and `{accel}` by
    /// `-cpu host -accel hvf` (macOS arm64) or `-cpu max -accel tcg`.
    pub args: &'static [&'static str],
    /// The firmware file (from check 4) this boot needs, if any.
    pub firmware: Option<&'static str>,
    pub marker: &'static str,
    pub deadline_secs: u64,
}

pub(super) const BOOTS: [Boot; 5] = [
    Boot {
        name: "x86_64-bios",
        program: "qemu-system-x86_64",
        args: &[],
        firmware: None,
        marker: "SeaBIOS",
        deadline_secs: 20,
    },
    Boot {
        name: "x86_64-uefi",
        program: "qemu-system-x86_64",
        args: &[
            "-M",
            "q35",
            "-drive",
            "if=pflash,format=raw,readonly=on,file={fw}",
        ],
        firmware: Some("edk2-x86_64-code.fd"),
        marker: "UEFI Interactive Shell",
        deadline_secs: 60,
    },
    Boot {
        name: "aarch64-uefi",
        program: "qemu-system-aarch64",
        args: &["-M", "virt", "{accel}", "-bios", "{fw}"],
        firmware: Some("edk2-aarch64-code.fd"),
        marker: "UEFI firmware",
        deadline_secs: 30,
    },
    Boot {
        name: "riscv64-opensbi",
        program: "qemu-system-riscv64",
        args: &["-M", "virt", "-bios", "default"],
        firmware: Some("opensbi-riscv64-generic-fw_dynamic.bin"),
        marker: "OpenSBI v",
        deadline_secs: 20,
    },
    Boot {
        name: "riscv64-uefi",
        program: "qemu-system-riscv64",
        args: &[
            "-M",
            "virt",
            "-drive",
            "if=pflash,unit=0,format=raw,readonly=on,file={fw}",
        ],
        firmware: Some("edk2-riscv-code.fd"),
        marker: "UEFI v",
        deadline_secs: 60,
    },
];

/// Reads `reader` until `marker` appears or `deadline` passes, writing every byte to `log`.
/// `Ok(false)` on the deadline or on end of input without the marker.
pub(super) fn wait_for_marker(
    reader: impl Read + Send + 'static,
    marker: &str,
    deadline: Duration,
    mut log: File,
) -> Result<bool, String> {
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    // The thread only forwards bytes; it ends at end of input, a read error, or when `rx` is gone.
    thread::spawn(move || {
        let mut reader = reader;
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => break,
            }
        }
    });
    let end = Instant::now() + deadline;
    let marker = marker.as_bytes();
    // The unmatched tail of earlier chunks, so a marker split across chunks is found.
    let mut window: Vec<u8> = Vec::new();
    loop {
        let left = end.saturating_duration_since(Instant::now());
        match rx.recv_timeout(left) {
            Ok(chunk) => {
                log.write_all(&chunk)
                    .map_err(|e| format!("cannot write the boot log: {e}"))?;
                window.extend_from_slice(&chunk);
                if window.windows(marker.len()).any(|w| w == marker) {
                    return Ok(true);
                }
                let keep = marker.len().saturating_sub(1);
                if window.len() > keep {
                    window.drain(..window.len() - keep);
                }
            }
            Err(_) => return Ok(false),
        }
    }
}

/// The arguments of `boot` (after the program), with the firmware path filled in.
pub(super) fn command(boot: &Boot, firmware: Option<&Path>, mac_arm64: bool) -> Vec<String> {
    let mut out: Vec<String> = COMMON.iter().map(|a| (*a).to_string()).collect();
    out.extend(["-name".to_string(), format!("xtask-doctor-{}", boot.name)]);
    for arg in boot.args {
        if *arg == "{accel}" {
            let accel = if mac_arm64 {
                "-cpu host -accel hvf"
            } else {
                "-cpu max -accel tcg"
            };
            out.extend(accel.split(' ').map(str::to_string));
        } else {
            let fw = firmware.map_or_else(String::new, |p| p.display().to_string());
            out.push(arg.replace("{fw}", &fw));
        }
    }
    out
}

/// Starts `program`, waits for `marker` on its stdout and stderr, and always kills and reaps it.
pub(super) fn run_one(
    program: &str,
    args: &[String],
    marker: &str,
    deadline: Duration,
    log_path: &Path,
) -> Result<bool, String> {
    if let Some(dir) = log_path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    let log =
        File::create(log_path).map_err(|e| format!("cannot create {}: {e}", log_path.display()))?;
    let (reader, writer) = std::io::pipe().map_err(|e| format!("cannot make a pipe: {e}"))?;
    let child = {
        // Both streams share one pipe; `cmd` is dropped at the end of this block so the only
        // write ends left are the child's, and the reader sees end of input when it exits.
        let mut cmd = Command::new(program);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(writer.try_clone().map_err(|e| e.to_string())?)
            .stderr(writer);
        cmd.spawn()
            .map_err(|e| format!("cannot start `{program}`: {e}"))?
    };
    let _guard = ChildGuard(child);
    wait_for_marker(reader, marker, deadline, log)
}

/// Check 6: every boot of [`BOOTS`], one after another. A boot whose prerequisite an earlier
/// check found missing is `FAIL` without running anything.
pub(super) fn check(
    state: &State,
    root: &Result<PathBuf, String>,
    mac_arm64: bool,
    runner: &dyn Runner,
) -> Vec<Line> {
    let mut lines = sweep(runner);
    lines.extend(BOOTS.iter().map(|boot| {
        let fail = |detail: String| Line::new(Status::Fail, C, boot.name, detail);
        if !state.qemu_usable(boot.program) {
            return fail(format!("{} not usable, see check host-tools", boot.program));
        }
        let firmware = match boot.firmware {
            None => None,
            Some(file) => match state.firmware.get(file) {
                Some(path) => Some(path.as_path()),
                None => return fail(format!("{file} not found, see check firmware")),
            },
        };
        let root = match root {
            Ok(root) => root,
            Err(e) => return fail(format!("repository root unknown: {e}")),
        };
        let rel = format!("target/doctor/{}.log", boot.name);
        let started = Instant::now();
        let found = run_one(
            boot.program,
            &command(boot, firmware, mac_arm64),
            boot.marker,
            Duration::from_secs(boot.deadline_secs),
            &root.join(&rel),
        );
        match found {
            Ok(true) => Line::new(
                Status::Ok,
                C,
                boot.name,
                format!(
                    "\"{}\" after {:.1} s",
                    boot.marker,
                    started.elapsed().as_secs_f32()
                ),
            ),
            Ok(false) => fail(format!(
                "no \"{}\" within {} s, log {rel}",
                boot.marker, boot.deadline_secs
            )),
            Err(e) => fail(format!("{e}, log {rel}")),
        }
    }));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::Captured;
    use std::collections::VecDeque;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-boot-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Yields one queued chunk per `read` call.
    struct Chunks(VecDeque<Vec<u8>>);

    impl Read for Chunks {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            match self.0.pop_front() {
                Some(c) => {
                    buf[..c.len()].copy_from_slice(&c);
                    Ok(c.len())
                }
                None => Ok(0),
            }
        }
    }

    fn chunks(parts: &[&str]) -> Chunks {
        Chunks(parts.iter().map(|p| p.as_bytes().to_vec()).collect())
    }

    fn pid_running(pid: u32) -> bool {
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .unwrap()
            .status
            .success()
    }

    #[test]
    fn marker_found_in_a_cursor_and_the_log_holds_the_bytes() {
        let dir = temp("cursor");
        let log = dir.join("x.log");
        let found = wait_for_marker(
            std::io::Cursor::new(b"boot\nSeaBIOS (version 1)\nmore".to_vec()),
            "SeaBIOS",
            Duration::from_secs(5),
            File::create(&log).unwrap(),
        );
        assert_eq!(found, Ok(true));
        let text = std::fs::read_to_string(&log).unwrap();
        assert!(text.starts_with("boot\nSeaBIOS"), "{text:?}");
    }

    #[test]
    fn marker_split_across_two_chunks_is_found() {
        let dir = temp("split");
        let found = wait_for_marker(
            chunks(&["... UEFI Inter", "active Shell v2.2"]),
            "UEFI Interactive Shell",
            Duration::from_secs(5),
            File::create(dir.join("x.log")).unwrap(),
        );
        assert_eq!(found, Ok(true));
    }

    #[test]
    fn eof_before_the_marker_is_false() {
        let dir = temp("eof");
        let found = wait_for_marker(
            chunks(&["nothing", "to see"]),
            "OpenSBI v",
            Duration::from_secs(5),
            File::create(dir.join("x.log")).unwrap(),
        );
        assert_eq!(found, Ok(false));
    }

    #[test]
    fn a_reader_that_never_yields_returns_false_within_the_deadline_plus_a_second() {
        let dir = temp("silent");
        let (reader, _writer_kept_open) = std::io::pipe().unwrap();
        let start = Instant::now();
        let found = wait_for_marker(
            reader,
            "never",
            Duration::from_millis(300),
            File::create(dir.join("x.log")).unwrap(),
        );
        assert_eq!(found, Ok(false));
        assert!(
            start.elapsed() < Duration::from_millis(1300),
            "{:?}",
            start.elapsed()
        );
    }

    #[test]
    fn a_child_guard_dropped_in_a_panic_leaves_the_child_reaped() {
        let child = std::process::Command::new("sleep")
            .arg("30")
            .spawn()
            .unwrap();
        let pid = child.id();
        assert!(pid_running(pid));
        let result = std::panic::catch_unwind(move || {
            let _guard = ChildGuard(child);
            panic!("doctor panicked");
        });
        assert!(result.is_err());
        assert!(!pid_running(pid), "child {pid} still running");
    }

    #[test]
    fn run_one_finds_the_marker_on_stderr_and_kills_the_child() {
        let dir = temp("run-found");
        let log = dir.join("sub/x.log");
        let start = Instant::now();
        let found = run_one(
            "sh",
            &["-c".into(), "echo out; echo MARK >&2; exec sleep 30".into()],
            "MARK",
            Duration::from_secs(10),
            &log,
        );
        assert_eq!(found, Ok(true));
        assert!(start.elapsed() < Duration::from_secs(5));
        let text = std::fs::read_to_string(&log).unwrap();
        assert!(text.contains("out") && text.contains("MARK"), "{text:?}");
    }

    #[test]
    fn run_one_times_out_and_still_returns() {
        let dir = temp("run-timeout");
        let start = Instant::now();
        let found = run_one(
            "sleep",
            &["30".into()],
            "MARK",
            Duration::from_millis(300),
            &dir.join("x.log"),
        );
        assert_eq!(found, Ok(false));
        assert!(start.elapsed() < Duration::from_secs(3));
    }

    #[test]
    fn run_one_reports_a_program_that_cannot_start() {
        let dir = temp("run-missing");
        let r = run_one(
            "no-such-program-xtask",
            &[],
            "MARK",
            Duration::from_secs(1),
            &dir.join("x.log"),
        );
        assert!(r.is_err());
    }

    fn row(name: &str) -> &'static Boot {
        BOOTS.iter().find(|b| b.name == name).expect(name)
    }

    fn joined(name: &str, mac: bool) -> String {
        let b = row(name);
        command(b, b.firmware.map(|_| Path::new("/fw/file.fd")), mac).join(" ")
    }

    fn common_text(name: &str) -> String {
        format!("-nographic -monitor none -no-reboot -m 256M -nic none -name xtask-doctor-{name}")
    }

    #[test]
    fn row_x86_64_bios() {
        let b = row("x86_64-bios");
        assert_eq!(
            (b.program, b.marker, b.deadline_secs),
            ("qemu-system-x86_64", "SeaBIOS", 20)
        );
        assert_eq!(joined("x86_64-bios", true), common_text("x86_64-bios"));
    }

    #[test]
    fn row_x86_64_uefi() {
        let b = row("x86_64-uefi");
        assert_eq!(
            (b.program, b.marker, b.deadline_secs, b.firmware),
            (
                "qemu-system-x86_64",
                "UEFI Interactive Shell",
                60,
                Some("edk2-x86_64-code.fd")
            )
        );
        assert_eq!(
            joined("x86_64-uefi", true),
            format!(
                "{} -M q35 -drive if=pflash,format=raw,readonly=on,file=/fw/file.fd",
                common_text("x86_64-uefi")
            )
        );
    }

    #[test]
    fn row_aarch64_uefi_uses_hvf_on_mac_and_tcg_elsewhere() {
        let b = row("aarch64-uefi");
        assert_eq!(
            (b.program, b.marker, b.deadline_secs, b.firmware),
            (
                "qemu-system-aarch64",
                "UEFI firmware",
                30,
                Some("edk2-aarch64-code.fd")
            )
        );
        assert_eq!(
            joined("aarch64-uefi", true),
            format!(
                "{} -M virt -cpu host -accel hvf -bios /fw/file.fd",
                common_text("aarch64-uefi")
            )
        );
        assert_eq!(
            joined("aarch64-uefi", false),
            format!(
                "{} -M virt -cpu max -accel tcg -bios /fw/file.fd",
                common_text("aarch64-uefi")
            )
        );
    }

    #[test]
    fn row_riscv64_opensbi() {
        let b = row("riscv64-opensbi");
        assert_eq!(
            (b.program, b.marker, b.deadline_secs),
            ("qemu-system-riscv64", "OpenSBI v", 20)
        );
        assert_eq!(
            joined("riscv64-opensbi", true),
            format!("{} -M virt -bios default", common_text("riscv64-opensbi"))
        );
    }

    #[test]
    fn row_riscv64_uefi() {
        let b = row("riscv64-uefi");
        assert_eq!(
            (b.program, b.marker, b.deadline_secs, b.firmware),
            (
                "qemu-system-riscv64",
                "UEFI v",
                60,
                Some("edk2-riscv-code.fd")
            )
        );
        assert_eq!(
            joined("riscv64-uefi", true),
            format!(
                "{} -M virt -drive if=pflash,unit=0,format=raw,readonly=on,file=/fw/file.fd",
                common_text("riscv64-uefi")
            )
        );
    }

    /// Replies to `pgrep` with a canned result, to `kill -KILL` with success, and to `kill -0`
    /// with failure (the process is gone); records every call.
    struct Pgrep(Captured);

    fn none() -> Captured {
        Captured {
            success: false,
            code: Some(1),
            text: String::new(),
        }
    }

    fn reply(success: bool, code: i32, text: &str) -> Captured {
        Captured {
            success,
            code: Some(code),
            text: text.to_string(),
        }
    }

    impl Runner for Pgrep {
        fn capture(&self, program: &str, args: &[&str]) -> Result<Captured, String> {
            match (program, args) {
                ("pgrep", _) => Ok(self.0.clone()),
                ("kill", ["-KILL", _]) => Ok(reply(true, 0, "")),
                ("kill", ["-0", _]) => Ok(reply(false, 1, "No such process")),
                _ => Err(format!("unexpected {program} {args:?}")),
            }
        }
    }

    /// Like [`Pgrep`] but records the calls, and `kill -0` can report the process still alive.
    struct Recording {
        pgrep: Result<Captured, String>,
        alive: bool,
        calls: std::cell::RefCell<Vec<String>>,
    }

    impl Runner for Recording {
        fn capture(&self, program: &str, args: &[&str]) -> Result<Captured, String> {
            self.calls
                .borrow_mut()
                .push(format!("{program} {}", args.join(" ")));
            match (program, args) {
                ("pgrep", _) => self.pgrep.clone(),
                ("kill", ["-KILL", _]) => Ok(reply(true, 0, "")),
                ("kill", ["-0", _]) if self.alive => Ok(reply(true, 0, "")),
                ("kill", ["-0", _]) => Ok(reply(false, 1, "No such process")),
                _ => Err(format!("unexpected {program} {args:?}")),
            }
        }
    }

    fn recording(pgrep: Result<Captured, String>, alive: bool) -> Recording {
        Recording {
            pgrep,
            alive,
            calls: std::cell::RefCell::default(),
        }
    }

    const PGREP: &str = "101 qemu-system-riscv64 -M virt -name xtask-doctor-leftover\n\
        202 /opt/homebrew/bin/qemu-system-riscv64 -nographic -name xtask-doctor-riscv64-uefi\n\
        303 qemu-system-x86_64 -M q35\n\
        404 zsh -c pgrep -f -l xtask-doctor-\n";

    #[test]
    fn every_boot_has_no_network_and_a_name_tag() {
        for boot in &BOOTS {
            let args = command(boot, Some(Path::new("/fw/f.fd")), true);
            let at = args.iter().position(|a| a == "-nic").expect("-nic");
            assert_eq!(args[at + 1], "none", "{}", boot.name);
            let at = args.iter().position(|a| a == "-name").expect("-name");
            assert_eq!(args[at + 1], format!("xtask-doctor-{}", boot.name));
        }
    }

    #[test]
    fn leftovers_picks_only_tagged_qemu_lines_also_as_crlf() {
        let want = vec![
            (101, "qemu-system-riscv64".to_string()),
            (202, "qemu-system-riscv64".to_string()),
        ];
        assert_eq!(leftovers(PGREP), want);
        assert_eq!(leftovers(&PGREP.replace('\n', "\r\n")), want);
        assert_eq!(leftovers(""), vec![]);
    }

    #[test]
    fn sweep_prints_nothing_when_pgrep_exits_1() {
        let r = recording(Ok(none()), false);
        assert_eq!(sweep(&r), vec![]);
        assert_eq!(*r.calls.borrow(), ["pgrep -f -l xtask-doctor-"]);
    }

    #[test]
    fn sweep_warns_once_when_pgrep_fails_otherwise() {
        let r = recording(Ok(reply(false, 2, "pgrep: bad")), false);
        let lines = sweep(&r);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0].status, Status::Warn);
        assert!(
            lines[0]
                .to_string()
                .starts_with("warn smoke-boot leftover:")
        );
        let cannot = recording(Err("cannot start `pgrep`".into()), false);
        assert_eq!(sweep(&cannot).len(), 1);
        assert!(!r.calls.borrow().iter().any(|c| c.starts_with("kill")));
    }

    #[test]
    fn sweep_kills_each_leftover_and_warns_for_each() {
        let r = recording(Ok(reply(true, 0, PGREP)), false);
        let lines = sweep(&r);
        let text: Vec<String> = lines.iter().map(ToString::to_string).collect();
        assert_eq!(
            text,
            [
                "warn smoke-boot leftover: killed pid 101 (qemu-system-riscv64)",
                "warn smoke-boot leftover: killed pid 202 (qemu-system-riscv64)",
            ]
        );
        let calls = r.calls.borrow();
        assert_eq!(calls.iter().filter(|c| *c == "kill -KILL 101").count(), 1);
        assert_eq!(calls.iter().filter(|c| *c == "kill -KILL 202").count(), 1);
    }

    #[test]
    fn sweep_says_so_when_a_leftover_survives_the_wait() {
        let r = recording(Ok(reply(true, 0, PGREP)), true);
        let lines = sweep_within(&r, Duration::from_millis(250));
        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[0].to_string(),
            "warn smoke-boot leftover: pid 101 (qemu-system-riscv64) still running"
        );
    }

    #[test]
    fn the_sweep_runs_even_when_every_boot_prerequisite_failed() {
        let dir = temp("sweep-prereq");
        let r = recording(Ok(reply(true, 0, PGREP)), false);
        let lines = check(&State::default(), &Ok(dir), true, &r);
        assert_eq!(lines.len(), BOOTS.len() + 2);
        assert_eq!(lines[0].status, Status::Warn);
        assert!(lines[2..].iter().all(|l| l.status == Status::Fail));
    }

    #[test]
    fn a_child_past_its_deadline_is_killed_and_reaped() {
        // The pid is not returned, so look for the process by its unique argument.
        let mut unique = Command::new("sleep");
        unique.arg("30.123");
        let start = Instant::now();
        let r = run_with_deadline(unique, Duration::from_secs(1));
        assert!(r.is_err(), "{r:?}");
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "{:?}",
            start.elapsed()
        );
        let left = Command::new("pgrep")
            .args(["-f", "sleep 30.123"])
            .output()
            .unwrap();
        assert!(left.stdout.is_empty(), "sleep still running");
        // A child that finishes in time returns its status.
        let mut quick = Command::new("sh");
        quick.args(["-c", "exit 3"]);
        let status = run_with_deadline(quick, Duration::from_secs(5)).unwrap();
        assert_eq!(status.code(), Some(3));
    }

    #[test]
    fn boots_whose_prerequisite_failed_fail_naming_it_and_run_nothing() {
        let dir = temp("prereq");
        let lines = check(&State::default(), &Ok(dir.clone()), true, &Pgrep(none()));
        assert_eq!(lines.len(), BOOTS.len());
        for (line, boot) in lines.iter().zip(&BOOTS) {
            assert_eq!(line.status, Status::Fail, "{line}");
            assert_eq!(line.item, boot.name);
            assert!(
                line.detail.contains("host-tools") || line.detail.contains("firmware"),
                "{line}"
            );
        }
        assert!(
            !dir.join("target").exists(),
            "a boot ran and made its log directory"
        );
        let no_root = check(
            &State::default(),
            &Err("no git".into()),
            true,
            &Pgrep(none()),
        );
        assert!(no_root.iter().all(|l| l.status == Status::Fail));
    }
}
