# M0.1 plan: Development environment

- **Status:** Accepted (2026-11-27).
- **Spec:** [spec.md](spec.md). This plan argues from the spec; read both. Progress and surprises go in [log.md](log.md).

## 1. Approach

Configuration first (toolchain, `Brewfile`, allow rules), then `doctor` in two builder steps (host checks, then smoke boots), then the reference library (its README and ignore rules, the `refs` code, the manifest's entries), then the research note in two researcher steps, then the documents that point at all of it. External commands go through a `Runner` trait so every check is unit-tested with canned output; only the evidence steps run the real tools.

Choices the spec leaves to implementation:

- `doctor` is a module directory: `xtask/src/doctor.rs` (statuses, lines, the runner, checks 1-5 and 7) and `xtask/src/doctor/boot.rs` (check 6). `refs` is `xtask/src/refs.rs`.
- The manifest is read with `toml` plus `serde` (`derive`), with `#[serde(deny_unknown_fields)]`; the checksum is `sha2`. Versions from `cargo add`; `cargo xtask deny` must pass.
- The rule "check a specification claim against `reference/text/` first" has one copy, in `reference/README.md` ("Citing a specification"); `AGENTS.md`, `researcher.md` and `review-sources.md` each get a one-line pointer to it (lesson docs-1; spec §4.6's "same line" is that pointer).

## 2. Steps

### Step 1: toolchain, Brewfile, allow rules (R1, R2, R13 in part)

- [x] **Who:** main session
- **Files:** `rust-toolchain.toml`, `Brewfile`, `.claude/settings.json`, `log.md`
- **Test first:** `brew bundle check --file Brewfile` fails before the file exists (`No such file`); `grep -c 'cargo xtask doctor' .claude/settings.json` prints 0.
- **Change:** spec §4.1 and §4.2; add exactly `Bash(cargo xtask doctor)` and `Bash(cargo xtask refs)` to the allow list. Log the CI baseline: the `test` job took 25 s and `macos` 42 s in run 44574212098 on `d814fe0`.
- **Verify:** `rustup show` lists the three targets and `rust-analyzer`, `llvm-tools`, `rust-src`; `brew bundle check --file Brewfile` prints `The Brewfile's dependencies are satisfied.`; `cargo xtask ci --quick` passes.
- **Commit:** `build: add bare-metal targets, tool components and a Brewfile (M0.1)`

### Step 2: `doctor` checks 1-5 (R3, R4, R6)

- [x] **Who:** builder
- **Files:** `xtask/src/main.rs`, `xtask/src/doctor.rs`, `xtask/src/cmd.rs` (only if a shared helper moves there)
- **Signatures:**
  - `pub(crate) enum Status { Ok, Warn, Skip, Fail }`
  - `pub(crate) struct Line { pub status: Status, pub check: &'static str, pub item: String, pub detail: String }`, with `Display` giving `<status> <check> <item>: <detail>` (status lowercase except `FAIL`)
  - `pub(crate) struct Captured { pub success: bool, pub text: String }` (stdout then stderr)
  - `pub(crate) trait Runner { fn capture(&self, program: &str, args: &[&str]) -> Result<Captured, String>; }`, with `SystemRunner` the real one
  - `fn parse_version(text: &str) -> Option<String>`; `fn at_least(version: &str, min: [u32; 3]) -> bool`; `fn search_dirs(text: &str) -> Vec<PathBuf>` (existing directories only)
  - `pub(crate) fn run() -> Result<(), String>`: runs the checks, prints each line, `Err` when any line is `Fail`; the checks are a list run in order by a function that is testable with fake checks.
- **Test first** (one per row, lesson specs-3): `parse(&["doctor", "--x"])` is an error (fails: no `doctor` command); `parse_version` on each fixture in `log.md` (QEMU, dtc, socat's three lines, pdftotext, iasl's dotless date, gdb) returns the logged version, and on a CRLF copy of each (lesson xtask-2); `at_least("11.0.9", [11,1,0])` is false and `"11.1.0"` true; a tool whose `Captured.success` is false is `Fail`; `search_dirs` on the logged `-L help` output drops the absent `qemu-firmware`; the runner function continues after a `Fail` and its result is `Err` only with a `Fail` line; a check whose prerequisite failed reports `Fail` naming it without calling the runner (fake runner that panics on call).
- **Change:** spec §4.3 checks 1-5 and check 7's slot left for step 5 (no line printed yet).
- **Verify:** `cargo xtask ci --quick` passes; `cargo xtask doctor` on this Mac prints `ok` for every item of checks 1-5.
- **Commit:** `feat(xtask): add doctor with toolchain, tool, firmware and accelerator checks (M0.1)`

### Step 3: `doctor` smoke boots (R5)

- [x] **Who:** builder
- **Files:** `xtask/src/doctor.rs`, `xtask/src/doctor/boot.rs`
- **Signatures:** `struct ChildGuard(Child)` whose `Drop` kills and waits; `fn wait_for_marker(reader: impl Read + Send + 'static, marker: &str, deadline: Duration, log: File) -> Result<bool, String>` (a reader thread sending chunks over a channel, `recv_timeout` against the remaining time, every chunk written to `log`, the marker searched across chunk boundaries); `const BOOTS: [Boot; 5]` holding spec §4.3's table.
- **Test first:** marker found in a `Cursor` (true) and the log holds the bytes; marker split across two chunks is found; a reader that never yields (the read end of `std::io::pipe()` with the writer kept open) returns false within deadline plus 1 s; EOF before the marker returns false; a `ChildGuard` around `sleep 30` dropped inside a `catch_unwind` panic leaves the child reaped (its pid no longer running, checked with `kill -0`).
- **Change:** spec §4.3 check 6; logs under `<repo root>/target/doctor/` (lesson xtask-1); the aarch64 command switches to `-cpu max -accel tcg` off macOS arm64.
- **Verify:** `cargo xtask ci --quick` passes; `cargo xtask doctor` prints `ok smoke-boot <name>` for all five; `pgrep -fl qemu-system` prints nothing afterwards; `target/doctor/*.log` has five files.
- **Commit:** `feat(xtask): smoke-boot each architecture's firmware in doctor (M0.1)`

### Step 4: the reference folder (R10)

- [ ] **Who:** main session
- **Files:** `reference/README.md`, `reference/manifest.toml` (a header comment and no entries), `.gitignore`
- **Test first:** `git check-ignore reference/files/x reference/text/x` prints nothing (exit 1).
- **Change:** the README in the shape of docs/README.md Conventions (status, When this applies, Contents naming `manifest.toml`, `files/` and `text/`), the manifest fields (spec §4.4), the licence statement, and "Citing a specification" (search `reference/text/`, cite id, version and section; fall back to the web, then "unverified"). Ignore `reference/files/` and `reference/text/`.
- **Verify:** `git check-ignore reference/files/x reference/text/x` prints both paths; `cargo xtask check-docs` reports 0 problems.
- **Commit:** `docs: add the reference library folder (M0.1)`

### Step 5: `cargo xtask refs` (R7, R8, R9) and doctor check 7

- [ ] **Who:** builder
- **Files:** `xtask/Cargo.toml`, `Cargo.lock`, `xtask/src/main.rs`, `xtask/src/refs.rs`, `xtask/src/doctor.rs`, `xtask/src/check_docs.rs` (`reference` joins `REQUIRED_READMES`; the array length changes from 9 to 10, lesson docs-2)
- **Signatures:** `Command::Refs { fetch: bool }` parsing `refs [--fetch]`; `struct Doc` (the manifest fields); `fn parse_manifest(text: &str) -> Result<Vec<Doc>, String>`; `enum RefStatus { Ok, Missing, Changed, NoText, Unpinned(String) }`; `fn status(root: &Path, doc: &Doc) -> RefStatus`; `fn place(tmp: &Path, dest: &Path, sha256: &str) -> Result<(), String>` (verify, then rename; delete `tmp` on a mismatch); `fn curl_args(url: &str, out: &Path) -> Vec<String>`; `fn fetch_all(root: &Path, docs: &[Doc], download: &dyn Fn(&Doc, &Path) -> Result<(), String>, convert: &dyn Fn(&Path, &Path) -> Result<(), String>) -> Result<(), String>` (the real `download` runs `curl`, the real `convert` runs `pdftotext` into a temporary name and renames).
- **Test first** (one per rule of spec §4.4, lesson specs-3): unknown field, missing required field, duplicate id, non-kebab id, `http` URL, unknown format, bad sha256, `how` without `manual`, `manual` without `how`, `sha256` missing on a non-manual entry: each rejected; a valid manifest parses, also as CRLF (xtask-2). `status` in a temporary tree gives each of the five statuses. `place` with the right digest moves the file; with a wrong one leaves nothing at `dest` and removes `tmp`. `curl_args` contains `--fail`, `--proto`, `=https`, `--proto-redir`, `--max-filesize` and the URL as separate elements. `fetch_all` with a fake `download` that fails for the first of two entries still attempts the second and returns `Err`; with a fake `convert` that fails, nothing is left in `text/` and the result is `Err`; a `no-text` entry is converted without calling `download`; a `manual` missing entry calls neither. Doctor check 7 maps `Missing`, `Changed`, `NoText` to `warn` and `Ok`, `Unpinned` to `ok`.
- **Change:** spec §4.4.
- **Verify:** `cargo xtask ci` passes (deny included); `cargo xtask refs` with the empty manifest prints nothing and exits 0; `cargo xtask doctor` passes.
- **Commit:** `feat(xtask): add refs, the reference library manifest and fetch (M0.1)`

### Step 6: the manifest's entries (R11, R14 in part)

- [ ] **Who:** builder
- **Files:** `reference/manifest.toml`, `log.md`
- **Test first:** `cargo xtask refs` lists 0 of the spec §4.4 documents.
- **Change:** one entry per document of spec §4.4, at the latest version on the publisher's site (versioned URL where one exists), with the checksum taken from a download; `manual` with `how` where no stable direct URL exists. Licence text as the publisher states it.
- **Verify:** `cargo xtask refs --fetch` exits 0; `cargo xtask refs` shows every non-manual entry `ok`; `du -sh reference/files reference/text` and the `refs` output are logged with the commit (lessons specs-2, specs-4).
- **Commit:** `docs: fill the reference library manifest (M0.1)`

### Step 7: research note, sections 1-3 (R12 in part)

- [ ] **Who:** researcher
- **Files:** `docs/research/dev-environment.md`, `log.md`
- **Test first:** the evidence per claim: versions and boots from `log.md` (measured here), the HVF patch series and Apple's EL2 support, AARCHMRS's licence text (read from the package, not inferred), UDB's state, Sail, Bochs' VMX option, Spike, Arm FVP hosts.
- **Change:** provenance line and spec §4.5 topics 1-3; the reference library is cited by path.
- **Verify:** every factual sentence carries a citation, `measured here`, or `unverified`; `cargo xtask check-docs` passes.
- **Commit:** `docs: add the development environment note, tools and ground truth (M0.1)`

### Step 8: research note, sections 4-6, and the Corrections (R12)

- [ ] **Who:** researcher
- **Files:** `docs/research/dev-environment.md`, `docs/research/toolchain-and-boot.md`, `log.md`
- **Test first:** the evidence for Verus, Atmosphere and `vostd`, Miri, Kani, `loom`, `build-std` (RFCs and the 2026 goal), CHERI's standardisation and QEMU; the two Corrections are measured here (log lines of 2026-11-27).
- **Change:** spec §4.5 topics 4-6 and the two Corrections entries.
- **Verify:** as step 7.
- **Commit:** `docs: finish the development environment note and correct the boot note (M0.1)`

### Step 9: documents and agent pointers (R13)

- [ ] **Who:** main session
- **Files:** `AGENTS.md`, `.claude/agents/researcher.md`, `.claude/agents/review-sources.md`, `CONTRIBUTING.md`, `xtask/README.md`, `docs/research/README.md`, `CHANGELOG.md`
- **Test first:** `grep -c 'reference/' AGENTS.md .claude/agents/researcher.md .claude/agents/review-sources.md` prints 0 for each.
- **Change:** spec §4.6 and §4.7 (pointers to `reference/README.md`, `doctor` in the Commands block, Setting up lines, the commands table, the note in Contents, the changelog line).
- **Verify:** `cargo xtask check-docs` reports 0 problems; `wc -l < AGENTS.md` is under 80.
- **Commit:** `docs: point agents and contributors at doctor and the reference library (M0.1)`

## 3. Final verification

`cargo xtask ci`; `cargo xtask doctor` and `cargo xtask refs` on the last commit, logged with its hash (lesson specs-4); the CI time pair (spec §4.1) logged from the pull request's run; the spec's Done-when list.

## 4. Lessons

- **xtask-1** followed: `doctor` logs and `refs` paths resolve from the repository root.
- **xtask-2** followed: version fixtures and the manifest are also tested as CRLF.
- **tooling-3** followed: only the exact command lines `cargo xtask doctor` and `cargo xtask refs` are allowed.
- **specs-1** followed: the "nothing left" checks name their scope (`pgrep -fl qemu-system`, `git ls-files reference/`).
- **specs-2** followed: the evidence §8 puts in `log.md` is in steps 1, 3, 6 and the final verification.
- **specs-3** followed: steps 2, 3 and 5 name one test per row.
- **specs-4** followed: evidence names its commit; the final verification re-runs `doctor` and `refs`.
- **docs-1** followed: the citation rule has one copy in `reference/README.md`.
- **docs-2** followed: the `REQUIRED_READMES` length is updated with the list.

## 5. Deviations

- Step 2: the `doctor.rs` tests were written after the code (only the `parse` test ran red first); stage 5's code review is asked to check that each can fail. `doctor` reads `rust-toolchain.toml` with a hand parser until step 5 adds `toml`, and finds the root with `git rev-parse --show-toplevel`.
- Step 3: `riscv64-opensbi` runs with `-bios default` but still requires check 4 to have found `opensbi-riscv64-generic-fw_dynamic.bin` (the file `default` loads); the spec does not say.
