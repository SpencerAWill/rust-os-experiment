# M0.3 plan: Host-safety guardrails for agents and later QEMU commands

- **Status:** Accepted (2026-11-27).
- **Spec:** [spec.md](spec.md). This plan argues from the spec; read both. Progress and surprises go in [log.md](log.md).

## 1. Approach

The manual checks come first, in the order the sandbox would otherwise get in their way: the `lsof` check of QEMU's gdb stub runs before any settings change (a sandboxed QEMU could not listen at all, spec §4.1.4), then the deny rules, then the sandbox. Claude Code reloads `.claude/settings.json` on change, so from step 3 on this session and every agent it dispatches run sandboxed: the evaluation of spec §8 is this session's own work. `check-settings` is written after the evaluation, so its allowlists hold exactly the entries the evaluation needed. Documentation comes last, describing what was measured.

## 2. Steps

### Step 1: check the gdb stub binding (R7)

- [x] **Who:** main session.
- **Files:** `docs/chunks/m0.3-host-safety/log.md`.
- **Test first:** the claim is the test: `-s` is expected to listen on `*:1234`. If it shows `127.0.0.1:1234`, spec §4.4.1's reason is wrong and the rule's wording is amended before step 5.
- **Change:** run `qemu-system-x86_64 -S -s -nographic -monitor none -nic none -name xtask-m0.3-probe` in the background, poll `lsof -nP -iTCP:1234 -sTCP:LISTEN` until it prints a line (at most 10 s), record it, kill the QEMU by pid and check with `pgrep -f xtask-m0.3-probe` that nothing is left. Repeat with `-gdb tcp:127.0.0.1:1234` in place of `-s`. Before the first run, `lsof -nP -iTCP:1234 -sTCP:LISTEN` prints nothing (no other listener). If macOS shows a firewall prompt, decline it and log that.
- **Verify:** `log.md` has both commands and both `lsof` lines; `pgrep -f xtask-m0.3-probe` prints nothing.
- **Commit:** `docs: record the gdb stub binding check (M0.3)`

### Step 2: deny rules (R2, R3)

- [ ] **Who:** main session (configuration).
- **Files:** `.claude/settings.json`, `log.md`.
- **Test first:** `jq '.permissions.deny' .claude/settings.json` prints `null`.
- **Change:** add `permissions.deny` with every rule of spec §4.2, in the spec's order: the Bash rules as `Bash(<command> *)`, the Read rules as `Read(<path>)`.
- **Verify:** `jq -e '.permissions.deny | length' .claude/settings.json` prints the rule count; in this session, `sudo true` is refused by the deny rule (not run) and a Read of `~/.ssh/m0.3-read-probe` (a file that does not exist, so nothing is read) is refused with a permission message, not "file not found"; both refusals quoted in `log.md`. If a form does not match, use the form that does and log it (spec §4.2).
- **Commit:** `feat(claude): deny system, Homebrew and credential commands to agents (M0.3)`

### Step 3: the sandbox and its evaluation (R1, R4)

- [ ] **Who:** main session (configuration and the evaluation).
- **Files:** `.claude/settings.json`, `log.md`.
- **Test first:** `touch ~/m0.3-sandbox-probe && rm ~/m0.3-sandbox-probe` succeeds (no sandbox yet); logged. Record `time cargo xtask ci` unsandboxed.
- **Change:** add the `sandbox` object of spec §4.1 with `allowWrite`, `excludedCommands` and `allowedDomains` empty, the `denyRead` paths of §4.1.5 and the credential denies of §4.1.6. Then, after `/sandbox` (or the first sandboxed Bash call) shows the sandbox is active, run spec §8's evaluation commands one at a time (`cargo xtask ci`, `cargo xtask doctor`, `gh issue view 10`, `gh project item-list 8 --owner SpencerAWill --limit 1000 --format json`, `git fetch`, `git push` of the chunk branch). For each failure, log the error, add the narrowest entry that fixes it in the order of spec §4.1.3 (write path, domain, `allowMachLookup` service, then exclusion), and re-run. Check each key against `claude`'s behaviour; a key honoured only from user settings is handled as spec §4.1.1 says. If the loop cannot run, take the fallback of spec §4.1.7 instead and stop this step there.
- **Verify:** every evaluation command passes in this session, the `touch` probe fails with a sandbox error, and `time cargo xtask ci` sandboxed is logged next to the unsandboxed time; `log.md` lists every entry added and why.
- **Commit:** `feat(claude): turn on the Claude Code sandbox for this repository (M0.3)`

### Step 4: `cargo xtask check-settings` (R5)

- [ ] **Who:** builder.
- **Files:** `xtask/src/check_settings.rs` (new), `xtask/src/main.rs`.
- **Test first:** in `check_settings.rs`, the unit tests of spec §8 R5 against small JSON values built in the test, plus one that runs the check on the committed file (`include_str!("../../.claude/settings.json")`); they fail because `check` is a `todo!()`. Then the `ci_steps_are_the_ported_set` test in `main.rs` with `Command::CheckSettings` after `Command::CheckWorkflows`, which fails until `ci_steps` changes.
- **Change:**
  - `check_settings.rs`, with a module doc like `check_workflows.rs`'s listing the rules of spec §4.3: `pub(crate) fn run() -> Result<(), String>` reads `.claude/settings.json` from the repository root, taken as the parent of `env!("CARGO_MANIFEST_DIR")` (not `git rev-parse`, since spec §4.3 says the check runs no other program; see Lessons, xtask-1), prints each finding on its own line and fails with the count; `fn check(settings: &serde_json::Value) -> Vec<String>` returns one message per finding, every finding listed (no early return). Constants, each entry with a reason comment for the three allowlists: `REQUIRED_DENY`, `REQUIRED_DENY_READ`, `REQUIRED_CREDENTIAL_DENY`, `EXCLUDED_ALLOWLIST`, `WRITE_PATH_ALLOWLIST`, `DOMAIN_ALLOWLIST`, holding exactly what step 3 committed. If step 3 took the fallback, the sandbox items (spec §4.3 items 2 to 5) are not written.
  - `main.rs`: `Command::CheckSettings`, parsed from `check-settings` with no flags, run after `CheckWorkflows` in the full `ci_steps`.
- **Tests that must reach their branch (xtask-6):** an allowlisted `~/.cargo` with `~` and `~/.cargo/..` in the file (exact match, not prefix); a `Bash(sudo:*)` entry does not satisfy `Bash(sudo *)` (exact strings); a credential deny with the right name and mode `mask`; a domain `*.github.com` against an allowlisted `github.com`; a CRLF copy of the committed file passes like the LF one (xtask-2). Each test is checked against one mutation of the branch it names (the builder reports which).
- **Verify:** `cargo xtask test` passes; `cargo xtask check-settings` prints no finding and exits 0; with one required deny rule removed from a scratch copy given to the test, the finding names it (covered by the unit test).
- **Commit:** `feat(xtask): check-settings keeps the agent guardrails in place (M0.3)`

### Step 5: wire the check into CI and allow it (R5)

- [ ] **Who:** main session (configuration).
- **Files:** `.github/workflows/ci.yml`, `.claude/settings.json`.
- **Test first:** `grep -c 'cargo xtask check-settings' .github/workflows/ci.yml` prints 0.
- **Change:** a second step `- run: cargo xtask check-settings` after `cargo xtask check-workflows` in the `check-workflows` job; `"Bash(cargo xtask check-settings)"` in `permissions.allow` after `check-workflows`.
- **Verify:** `cargo xtask check-workflows` and `cargo xtask check-settings` pass; the grep prints 1.
- **Commit:** `ci: run check-settings in the check-workflows job (M0.3)`

### Step 6: documentation (R6)

- [ ] **Who:** main session.
- **Files:** `CONTRIBUTING.md`, `.claude/README.md`, `xtask/README.md`, `AGENTS.md`, `CHANGELOG.md`.
- **Test first:** `grep -c '^## Host safety' CONTRIBUTING.md` prints 0.
- **Change:** CONTRIBUTING "Host safety" after "Setting up" (spec §4.4, §4.5) and `check-settings` in "What CI enforces"; `.claude/README.md` (spec §4.6), with the settings from step 3 and their reasons, and `settings.json`'s Contents line updated; the `check-settings` row in `xtask/README.md`'s command table and `check_settings.rs` in its `src/` line; one line under "Rules that fail CI" in `AGENTS.md`; an M0.3 line at the top of `CHANGELOG.md`'s Unreleased list.
- **Verify:** `cargo xtask check-docs` reports 0 problems; `wc -l AGENTS.md` under 80; `cargo xtask typos` passes.
- **Commit:** `docs: host-safety rules and the agent sandbox (M0.3)`

## 3. Final verification

`cargo xtask ci`, run in this session (sandboxed), and the spec's Done-when checklist; the follow-up issue for the agent token is opened in stage 7.

## 4. Lessons

From `lessons-researcher` (dispatched by agent name):

- docs-1: followed; the host-safety rules live in CONTRIBUTING, and `.claude/README.md` and `AGENTS.md` link to them.
- docs-3: followed; measured claims in the docs (timings, exclusions and their reasons) point at the `log.md` entries.
- docs-4: followed; `cargo xtask typos` over the chunk folder before each commit of it, commits cited by full hash in `log.md`.
- specs-1: followed; R4's "fails" names the probe path, R7's `pgrep` names its tag.
- specs-2: followed; the log evidence of R3, R4 and R7 is steps 1 to 3.
- specs-3: followed; step 4 names one test per R5 case.
- specs-4: followed; each manual check in `log.md` names the commit it ran on, and step 3's evaluation is re-run in final verification if settings change after it.
- specs-6: followed; spec §6's claims about the existing `check-workflows` job and `WRITE_ALLOWLIST` were checked against `ci.yml` and `check_workflows.rs` while planning.
- tooling-3: followed; the new allow entry is the exact `Bash(cargo xtask check-settings)`.
- tooling-4: followed; the README calls deny rules a guardrail, and step 2 does not chase evasions.
- tooling-15: followed; JSON is piped straight into `jq`, never through `echo`.
- xtask-1: deviated: the root is the parent of `CARGO_MANIFEST_DIR`, which is also independent of the caller's directory (the lesson's concern) and keeps the check from running a program (spec §4.3).
- xtask-2: followed; a CRLF copy of the settings file in the tests.
- xtask-6: followed; step 4 lists the inputs where a prefix or loose match and the exact one disagree.

## 5. Deviations

None.
