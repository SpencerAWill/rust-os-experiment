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

/// The leftover tagged QEMU processes among the pids in `pgrep -f` output: pid and program
/// basename. Each pid is checked on its own with `ps`, so a process whose arguments forge a line
/// is not believed; one that has exited (`ps` exits non-zero) is skipped, and a `ps` that cannot
/// be started is an error. `-ww` because procps-ng takes the width from an exported `COLUMNS`
/// even into a pipe.
fn leftovers(runner: &dyn Runner, pgrep_output: &str) -> Result<Vec<(u32, String)>, String> {
    let ps = |field: &str, id: &str| -> Result<Option<String>, String> {
        let c = runner.capture("ps", &["-ww", "-o", field, "-p", id])?;
        Ok(c.success.then_some(c.text))
    };
    let mut found = Vec::new();
    for pid in pgrep_output
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
    {
        let id = pid.to_string();
        let Some(comm) = ps("comm=", &id)? else {
            continue;
        };
        let Some(base) = comm.trim().rsplit('/').next().map(str::to_string) else {
            continue;
        };
        let Some(args) = ps("args=", &id)? else {
            continue;
        };
        if base.starts_with("qemu-system-") && args.contains("-name xtask-doctor-") {
            found.push((pid, base));
        }
    }
    Ok(found)
}

/// Kills every QEMU left by an earlier `doctor` (spec section 4.3, amendment of 2026-11-27).
fn sweep(runner: &dyn Runner) -> Vec<Line> {
    sweep_within(runner, Duration::from_secs(5))
}

fn sweep_within(runner: &dyn Runner, wait: Duration) -> Vec<Line> {
    const ITEM: &str = "leftover";
    let warn = |detail: String| Line::new(Status::Warn, C, ITEM, detail);
    let found = match runner.capture("pgrep", &["-f", "xtask-doctor-"]) {
        Ok(c) if c.success => match leftovers(runner, &c.text) {
            Ok(found) => found,
            Err(e) => return vec![warn(e)],
        },
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
                match runner.capture("kill", &["-0", &id]) {
                    Ok(c) if c.success => {}
                    Ok(_) => return warn(format!("killed pid {pid} ({program})")),
                    Err(e) => return warn(format!("cannot tell whether pid {pid} exited: {e}")),
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
    /// The firmware files (from check 4) this boot needs; the first is the one `{fw}` becomes.
    pub firmware: &'static [&'static str],
    pub marker: &'static str,
    pub deadline_secs: u64,
}

pub(super) const BOOTS: [Boot; 5] = [
    Boot {
        name: "x86_64-bios",
        program: "qemu-system-x86_64",
        args: &[],
        firmware: &[],
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
        firmware: &["edk2-x86_64-code.fd"],
        marker: "UEFI Interactive Shell",
        deadline_secs: 60,
    },
    Boot {
        name: "aarch64-uefi",
        program: "qemu-system-aarch64",
        args: &["-M", "virt", "{accel}", "-bios", "{fw}"],
        firmware: &["edk2-aarch64-code.fd"],
        marker: "UEFI firmware",
        deadline_secs: 30,
    },
    Boot {
        name: "riscv64-opensbi",
        program: "qemu-system-riscv64",
        args: &["-M", "virt", "-bios", "default"],
        firmware: &["opensbi-riscv64-generic-fw_dynamic.bin"],
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
        firmware: &[
            "edk2-riscv-code.fd",
            "opensbi-riscv64-generic-fw_dynamic.bin",
        ],
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
        // `recv_timeout` returns a queued chunk before it looks at the deadline, so check it here.
        if Instant::now() >= end {
            return Ok(false);
        }
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
        let mut firmware = None;
        for (i, file) in boot.firmware.iter().enumerate() {
            match state.firmware.get(file) {
                Some(path) if i == 0 => firmware = Some(path.as_path()),
                Some(_) => {}
                None => return fail(format!("{file} not found, see check firmware")),
            }
        }
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

    /// Returns a full non-matching chunk on each of `0` calls, then ends: a guest that keeps
    /// printing (finite, so a broken deadline ends the test instead of filling the disk).
    struct Endless(u32);

    impl Read for Endless {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.0 == 0 {
                return Ok(0);
            }
            self.0 -= 1;
            buf.fill(b'x');
            Ok(buf.len())
        }
    }

    #[test]
    fn a_reader_that_never_stops_printing_returns_false_within_the_deadline_plus_a_second() {
        let dir = temp("endless");
        // A long marker that almost matches makes each queued chunk slow to scan, so the
        // queue is never empty and `recv_timeout` never reports the deadline by itself.
        let marker = format!("{}y", "x".repeat(4000));
        let start = Instant::now();
        let found = wait_for_marker(
            Endless(8000),
            &marker,
            Duration::from_millis(100),
            File::create(dir.join("x.log")).unwrap(),
        );
        assert_eq!(found, Ok(false));
        assert!(
            start.elapsed() < Duration::from_millis(1100),
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
        command(b, b.firmware.first().map(|_| Path::new("/fw/file.fd")), mac).join(" ")
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
                &["edk2-x86_64-code.fd"][..]
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
                &["edk2-aarch64-code.fd"][..]
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
                &[
                    "edk2-riscv-code.fd",
                    "opensbi-riscv64-generic-fw_dynamic.bin"
                ][..]
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

    /// A fake keyed on program and arguments: `pgrep`, `ps -ww -o comm= -p <pid>`,
    /// `ps -ww -o args= -p <pid>`, `kill -KILL` and `kill -0`; records the calls. A pid missing from
    /// `procs` is gone (`ps` exits 1).
    struct Recording {
        pgrep: Result<Captured, String>,
        /// `(pid, comm, args)` of each live process.
        procs: Vec<(u32, &'static str, &'static str)>,
        alive: bool,
        kill0_cannot_start: bool,
        ps_cannot_start: bool,
        kill_exit: i32,
        eol: &'static str,
        calls: std::cell::RefCell<Vec<String>>,
    }

    impl Runner for Recording {
        fn capture(&self, program: &str, args: &[&str]) -> Result<Captured, String> {
            self.calls
                .borrow_mut()
                .push(format!("{program} {}", args.join(" ")));
            match (program, args) {
                ("pgrep", _) => self.pgrep.clone(),
                ("ps", _) if self.ps_cannot_start => Err("cannot start `ps`: nope".into()),
                ("ps", ["-ww", "-o", field @ ("comm=" | "args="), "-p", pid]) => {
                    let pid: u32 = pid.parse().unwrap();
                    Ok(match self.procs.iter().find(|p| p.0 == pid) {
                        Some(p) => {
                            let text = if *field == "comm=" { p.1 } else { p.2 };
                            reply(true, 0, &format!("{text}{}", self.eol))
                        }
                        None => reply(false, 1, ""),
                    })
                }
                ("kill", ["-KILL", _]) if self.kill_exit != 0 => {
                    Ok(reply(false, self.kill_exit, "Operation not permitted"))
                }
                ("kill", ["-KILL", _]) => Ok(reply(true, 0, "")),
                ("kill", ["-0", _]) if self.kill0_cannot_start => {
                    Err("cannot start `kill`: boom".into())
                }
                ("kill", ["-0", _]) if self.alive => Ok(reply(true, 0, "")),
                ("kill", ["-0", _]) => Ok(reply(false, 1, "No such process")),
                _ => Err(format!("unexpected {program} {args:?}")),
            }
        }
    }

    const PROCS: [(u32, &str, &str); 7] = [
        (
            101,
            "qemu-system-riscv64",
            "qemu-system-riscv64 -M virt -name xtask-doctor-leftover",
        ),
        (
            202,
            "/opt/homebrew/bin/qemu-system-riscv64",
            "/opt/homebrew/bin/qemu-system-riscv64 -nographic -name xtask-doctor-riscv64-uefi",
        ),
        (303, "qemu-system-x86_64", "qemu-system-x86_64 -M q35"),
        // A forged process: its arguments carry the tag and a newline then another pid.
        (
            404,
            "sh",
            "sh -c echo\n505 qemu-system-riscv64 -name xtask-doctor-x",
        ),
        // Tagged, but not a QEMU binary.
        (606, "vim", "vim -name xtask-doctor-notes"),
        // The pid the forged line names, alive and not QEMU.
        (505, "bash", "bash"),
        // Linux truncates `comm` to 15 characters.
        (
            808,
            "qemu-system-ris",
            "qemu-system-riscv64 -M virt -name xtask-doctor-x",
        ),
    ];

    fn recording(pgrep: Result<Captured, String>, alive: bool) -> Recording {
        Recording {
            pgrep,
            procs: PROCS.to_vec(),
            alive,
            kill0_cannot_start: false,
            ps_cannot_start: false,
            kill_exit: 0,
            eol: "\n",
            calls: std::cell::RefCell::default(),
        }
    }

    /// What `pgrep -f xtask-doctor-` prints for [`PROCS`], plus 505 (named by the forged line,
    /// alive as `bash`), 707 (gone) and 808 (a 15-character `comm`).
    const PGREP: &str = "101\n202\n303\n404\n505\n606\n707\n808\n";

    #[test]
    fn every_boot_has_no_network_and_a_name_tag() {
        for boot in &BOOTS {
            let args = command(
                boot,
                boot.firmware.first().map(|_| Path::new("/fw/f.fd")),
                true,
            );
            let at = args.iter().position(|a| a == "-nic").expect("-nic");
            assert_eq!(args[at + 1], "none", "{}", boot.name);
            let at = args.iter().position(|a| a == "-name").expect("-name");
            assert_eq!(args[at + 1], format!("xtask-doctor-{}", boot.name));
        }
    }

    #[test]
    fn sweep_prints_nothing_when_pgrep_exits_1() {
        let r = recording(Ok(none()), false);
        assert_eq!(sweep(&r), vec![]);
        assert_eq!(*r.calls.borrow(), ["pgrep -f xtask-doctor-"]);
    }

    #[test]
    fn sweep_warns_once_when_pgrep_exits_2_or_cannot_start() {
        let r = recording(Ok(reply(false, 2, "pgrep: bad")), false);
        let lines = sweep(&r);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0].status, Status::Warn);
        assert!(
            lines[0]
                .to_string()
                .starts_with("warn smoke-boot leftover:")
        );
        assert!(!r.calls.borrow().iter().any(|c| c.starts_with("kill")));
        let cannot = recording(Err("cannot start `pgrep`".into()), false);
        assert_eq!(sweep(&cannot).len(), 1);
    }

    #[test]
    fn sweep_kills_only_tagged_qemu_processes_also_with_a_full_path_and_crlf() {
        for eol in ["\n", "\r\n"] {
            let mut r = recording(Ok(reply(true, 0, &PGREP.replace('\n', eol))), false);
            r.eol = eol;
            let text: Vec<String> = sweep(&r).iter().map(ToString::to_string).collect();
            assert_eq!(
                text,
                [
                    "warn smoke-boot leftover: killed pid 101 (qemu-system-riscv64)",
                    "warn smoke-boot leftover: killed pid 202 (qemu-system-riscv64)",
                    "warn smoke-boot leftover: killed pid 808 (qemu-system-ris)",
                ],
                "{eol:?}"
            );
            let calls = r.calls.borrow();
            let kills: Vec<&String> = calls
                .iter()
                .filter(|c| c.starts_with("kill -KILL"))
                .collect();
            assert_eq!(
                kills,
                ["kill -KILL 101", "kill -KILL 202", "kill -KILL 808"],
                "{eol:?}"
            );
            assert_eq!(calls[0], "pgrep -f xtask-doctor-");
            // The forged line's named victim is alive and is not QEMU: never killed.
            assert!(!calls.iter().any(|c| c == "kill -KILL 505"), "{eol:?}");
            assert!(calls.iter().any(|c| c == "ps -ww -o args= -p 101"));
        }
    }

    #[test]
    fn sweep_warns_when_kill_0_cannot_start_instead_of_saying_killed() {
        let mut r = recording(Ok(reply(true, 0, "101\n")), false);
        r.kill0_cannot_start = true;
        let lines = sweep(&r);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0].status, Status::Warn);
        assert!(lines[0].detail.contains("boom"), "{}", lines[0]);
        assert!(!lines[0].detail.contains("killed"), "{}", lines[0]);
    }

    #[test]
    fn sweep_warns_once_when_ps_cannot_start() {
        let mut r = recording(Ok(reply(true, 0, "101\n202\n")), false);
        r.ps_cannot_start = true;
        let lines = sweep(&r);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(
            lines[0]
                .to_string()
                .starts_with("warn smoke-boot leftover:"),
            "{}",
            lines[0]
        );
        assert!(lines[0].detail.contains("nope"), "{}", lines[0]);
        assert!(!r.calls.borrow().iter().any(|c| c.starts_with("kill")));
    }

    #[test]
    fn sweep_warns_once_when_kill_fails_and_does_not_wait() {
        let mut r = recording(Ok(reply(true, 0, "101\n")), true);
        r.kill_exit = 1;
        let lines = sweep_within(&r, Duration::from_secs(30));
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].detail.contains("`kill` failed"), "{}", lines[0]);
        assert!(lines[0].detail.contains("101"), "{}", lines[0]);
        assert!(!r.calls.borrow().iter().any(|c| c == "kill -0 101"));
    }

    #[test]
    fn sweep_says_so_when_a_leftover_survives_the_wait() {
        let r = recording(Ok(reply(true, 0, PGREP)), true);
        let lines = sweep_within(&r, Duration::from_millis(250));
        assert_eq!(lines.len(), 3);
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
        assert_eq!(lines.len(), BOOTS.len() + 3);
        assert_eq!(lines[0].status, Status::Warn);
        assert!(lines[3..].iter().all(|l| l.status == Status::Fail));
    }

    #[test]
    fn a_child_past_its_deadline_is_killed_and_reaped() {
        // The pid is not returned, so the child records its own (kept by `exec`).
        let dir = temp("deadline-pid");
        let pidfile = dir.join("pid");
        let mut unique = Command::new("sh");
        unique.args([
            "-c",
            &format!("echo $$ > '{}'; exec sleep 30", pidfile.display()),
        ]);
        let start = Instant::now();
        let r = run_with_deadline(unique, Duration::from_secs(1));
        assert!(r.is_err(), "{r:?}");
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "{:?}",
            start.elapsed()
        );
        let pid: u32 = std::fs::read_to_string(&pidfile)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        assert!(!pid_running(pid), "sleep {pid} still running");
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

    const FW_FILES: [&str; 4] = [
        "edk2-x86_64-code.fd",
        "edk2-aarch64-code.fd",
        "edk2-riscv-code.fd",
        "opensbi-riscv64-generic-fw_dynamic.bin",
    ];

    /// Only the non-x86 QEMUs are usable, so no real QEMU can start.
    fn non_x86_state(files: &[&'static str]) -> State {
        State {
            qemu_aarch64: true,
            qemu_riscv64: true,
            firmware: files
                .iter()
                .map(|f| (*f, PathBuf::from("/fw").join(f)))
                .collect(),
            ..State::default()
        }
    }

    const NON_X86: [&str; 3] = ["aarch64-uefi", "riscv64-opensbi", "riscv64-uefi"];

    #[test]
    fn firmware_boots_without_firmware_fail_naming_check_firmware_and_create_nothing() {
        let dir = temp("no-firmware");
        let lines = check(&non_x86_state(&[]), &Ok(dir.clone()), true, &Pgrep(none()));
        assert_eq!(lines.len(), BOOTS.len());
        for name in NON_X86 {
            let l = lines.iter().find(|l| l.item == name).unwrap();
            assert_eq!(l.status, Status::Fail, "{l}");
            assert!(l.detail.contains("check firmware"), "{l}");
        }
        for name in ["x86_64-bios", "x86_64-uefi"] {
            let l = lines.iter().find(|l| l.item == name).unwrap();
            assert!(l.detail.contains("check host-tools"), "{l}");
        }
        assert!(
            !dir.join("target").exists(),
            "a boot made its log directory"
        );
    }

    #[test]
    fn riscv64_uefi_needs_opensbi_too() {
        let dir = temp("riscv-uefi-fw");
        let state = non_x86_state(&["edk2-riscv-code.fd"]);
        let lines = check(&state, &Ok(dir.clone()), true, &Pgrep(none()));
        let l = lines.iter().find(|l| l.item == "riscv64-uefi").unwrap();
        assert_eq!(l.status, Status::Fail, "{l}");
        assert!(
            l.detail.contains("opensbi-riscv64-generic-fw_dynamic.bin")
                && l.detail.contains("check firmware"),
            "{l}"
        );
        assert!(!dir.join("target").exists());
    }

    #[test]
    fn with_firmware_present_an_unknown_root_fails_naming_it() {
        let lines = check(
            &non_x86_state(&FW_FILES),
            &Err("no git".into()),
            true,
            &Pgrep(none()),
        );
        for name in NON_X86 {
            let l = lines.iter().find(|l| l.item == name).unwrap();
            assert_eq!(l.status, Status::Fail, "{l}");
            assert!(l.detail.contains("repository root unknown"), "{l}");
            assert!(l.detail.contains("no git"), "{l}");
        }
    }
}
