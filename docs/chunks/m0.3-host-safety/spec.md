# M0.3: Host-safety guardrails for agents and later QEMU commands

- **Status:** Draft (2026-11-27).
- **Kind:** build · **Milestone:** M0 (repository foundations) · **Depends on:** M0.1 · **Decides:** no ADR
- **Issue:** #10
- **Estimated size:** about 450 changed lines, excluding tests, lockfiles and generated files (settings about 80, `check-settings` about 200, documents about 170)

Requirement keywords (MUST, MUST NOT, SHOULD, MAY) follow [RFC 2119](https://www.rfc-editor.org/rfc/rfc2119) as clarified by [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174): they are normative only in uppercase.

## 1. Summary

Agents run shell commands on the maintainer's only Mac, and from M2 `xtask` will start QEMU with images, a debug stub and sockets. This chunk turns on Claude Code's sandbox in the shared settings, adds deny rules for system commands and credential files, adds a CI check that keeps those settings in place, and writes the host-safety rules (including the ones later QEMU commands follow) into CONTRIBUTING. At the end, `sudo true` and a write outside the repository are refused in a Claude Code session here, and `cargo xtask ci` fails if a required setting is removed.

## 2. Goals and non-goals

Goals:

- An agent's Bash command, and every process it starts, can write only inside the repository and the caches the build needs, and reach only allowlisted domains (the sandbox, §4.1).
- The commands that damage a machine when run unattended, and the files that hold credentials, are denied by permission rules, which also cover the Read and Edit tools the sandbox does not (§4.2).
- `cargo xtask ci` fails when a required deny rule or sandbox setting is missing, or when a command is excluded from the sandbox without a reason (§4.3).
- The rules a person follows, and the rules M2's QEMU commands will follow, are written in CONTRIBUTING "Host safety" (§4.4, §4.5).

Non-goals:

- **The agent's GitHub token.** The maintainer's comment on #10 asks for a token limited to this repository. GitHub lists "Projects owned by a user account" as a fine-grained token limitation, and the loop uses user-owned Project 8, so it needs its own design. It moves to a follow-up issue opened at close-out (stage 7); this chunk's pull request closes #10 with that item moved.
- Running agents in a VM or container by default.
- Enforcing the QEMU rules in code before M2 has a QEMU command to enforce them on.
- Changing `doctor` or `refs` (M0.1).
- Personal settings: `autoAllowBashIfSandboxed` and other approval preferences stay in each contributor's `settings.local.json`.

## 3. Guide-level explanation

A contributor opens Claude Code in the repository. Bash commands now run inside the sandbox: `cargo xtask ci` works as before; `touch ~/x` fails with a permission error; `curl https://example.com` is refused or asks for approval; `sudo true` is refused before it runs. Commands that cannot run sandboxed (for example `gh` on macOS, §4.1.3) are listed in `.claude/settings.json`, each with its reason in `xtask/src/check_settings.rs`. The maintainer runs anything else that needs the host with `! <command>`, which Claude Code does not sandbox or check.

## 4. Reference-level design

### 4.1 The sandbox

The shared `.claude/settings.json` gains a `sandbox` object. Key names are those of the Claude Code settings reference at the version recorded in `log.md` (2.1.332 when this spec was written); the work stage checks each against `claude` and the docs before relying on it.

1. `enabled: true`; `allowUnsandboxedCommands: false`, so Claude cannot retry a failed command outside the sandbox (*my call*: a command that needs the host is either an exclusion, reviewed like any other change, or the maintainer's `!`); `failIfUnavailable: true`, so a session whose sandbox cannot start stops instead of running unsandboxed. If the work stage finds a key is honoured only from user or managed settings, it stays in the shared file only if harmless there, and `.claude/README.md` gives the user-settings line instead.
2. **Writes** (`filesystem.allowWrite`): beyond the default (the working directory and a per-session temp directory), only the paths the build and its tools write, each with a reason in the README: expected `~/.cargo` (registry, git checkouts, `cargo-deny`'s advisory database) and `~/.rustup` (the pinned toolchain). A path is added only when a command in §8's evaluation fails without it.
3. **Exclusions** (`excludedCommands`): an entry takes a Bash call out of the sandbox only when entries cover every command in it, so `gh issue view 1 && curl x` stays sandboxed under `gh *` alone (sandboxing docs). A command runs outside the sandbox only if it fails inside it for a reason the log records, and no setting short of exclusion fixes it. Expected candidates: `gh` (the docs list TLS verification failures on macOS), `git push` and `git fetch` if the credential helper needs the keychain, and `cargo xtask doctor` if Hypervisor.framework or QEMU cannot run sandboxed. Each exclusion is as narrow as works (`gh *`, not `*`).
4. **Network** (`network.allowedDomains`): the domains the evaluated commands reach, each with a reason (expected: `crates.io`, `index.crates.io`, `static.crates.io`, `static.rust-lang.org`, `github.com`). Other domains prompt, the default (*my call*: `strictAllowlist` would also block a research command the maintainer approves by hand). No `allowLocalBinding`, `allowUnixSockets` or `allowAllUnixSockets` (M2 decides what QEMU needs).
5. **Reads** (`filesystem.denyRead`): the credential paths of §4.2.2, so a sandboxed process cannot read what the Read tool is denied.
6. **Credentials** (`credentials.envVars`, mode `deny`): `GH_TOKEN`, `GITHUB_TOKEN`, `GH_ENTERPRISE_TOKEN`, `CARGO_REGISTRY_TOKEN`, so a sandboxed command does not see a token the session's environment holds. An excluded command still does; the README says so.
7. **Fallback.** If the loop's commands (§8, evaluation) cannot all run with the settings above, the sandbox moves to an opt-in snippet in `.claude/README.md`, the log records which command failed and why, and §4.3's sandbox checks are dropped. This is the only outcome change the work stage may make without an amendment.

### 4.2 Permission deny rules

`permissions.deny` in `.claude/settings.json` holds at least the rules below. Deny rules are evaluated before allow and ask rules, cover the Read and Edit tools as well as Bash, and also apply to sandbox exclusions. They match the command text, after Claude Code strips wrappers such as `timeout` and `nohup`, and split compound commands (`&&`, `;`, `|`, `$(...)`); `/usr/bin/sudo`, `sh -c '...'` and `env sudo` are not matched. They are a guardrail for the usual form of a command, not a sandbox ([lesson tooling-4](../../lessons/tooling.md)); the README says so.

1. **Bash** (`Bash(<command> *)`, which also matches the bare command): `sudo`, `diskutil`, `dd`, `hdiutil`, `launchctl` (from #10); `tmutil` (can delete Time Machine snapshots), `csrutil`, `nvram`, `osascript` (drives other applications), `security` (reads keychain items), `shutdown`, `reboot`, `halt`; and `brew install`, `brew reinstall`, `brew upgrade`, `brew uninstall`, `brew remove`, `brew rm`, `brew autoremove`, `brew bundle` (*my call*: Homebrew is shared with the maintainer's other projects, so its state changes only by the maintainer's `!`).
2. **Read** (`Read(<path>)`, `~/` being the home directory; the permissions docs say a `Read` deny rule also blocks the Edit and Write tools on the same path, so no `Edit` rule is needed *(main session's call)*): `~/.ssh/**`, `~/.aws/**`, `~/.gnupg/**`, `~/.config/gh/**`, `~/Library/Keychains/**`, `~/.netrc`, `~/.git-credentials`, `~/.cargo/credentials.toml`, `~/.cargo/credentials`.

The work stage confirms in a session (§8) that `Bash(sudo *)` refuses `sudo true` and `Read(~/.ssh/**)` refuses reading a file there; if a pattern form does not match as documented, the form that does is used and the deviation logged.

### 4.3 `cargo xtask check-settings`

A new `xtask` command, also a step of the full `cargo xtask ci` (after `check-workflows`), and allowed in `.claude/settings.json` as `Bash(cargo xtask check-settings)`. It parses `.claude/settings.json` with `serde_json` and fails, listing every finding, if:

1. any rule of §4.2 is missing from `permissions.deny` (the required list is a constant in `xtask/src/check_settings.rs`; more rules are allowed);
2. `sandbox.enabled` is not `true` or `sandbox.allowUnsandboxedCommands` is not `false` (unless §4.1.7's fallback was taken);
3. an entry of `sandbox.excludedCommands` is not in an `EXCLUDED_ALLOWLIST` constant, each entry of which carries a reason in a comment above it, as `WRITE_ALLOWLIST` in `check_workflows.rs` does;
4. `permissions.allow` has an entry that is a bare tool name (`Bash`, `Read`, `Edit`, `Write`) or `Bash(*)`, which would allow every command of that tool;
5. the file is not valid JSON, or a field above has the wrong type.

It reads only that file, writes nothing, and runs no other program. In GitHub Actions it is a second step of the existing `check-workflows` job in `.github/workflows/ci.yml`, which is already a required check, so the ruleset in `infra/github/` does not change *(main session's call, from the cold read: a job of its own would need a Pulumi change and an apply)*. The shared settings are the ones checked; `settings.local.json` is personal and is not.

### 4.4 Rules for later QEMU commands

Written now in CONTRIBUTING "Host safety", applied from M2's `cargo xtask run`; no code enforces them in this chunk.

1. The gdb stub listens on loopback only: `-gdb tcp:127.0.0.1:<port>`, never `-s` (which is `-gdb tcp::1234`, listening on every interface; checked with `lsof`, §8).
2. QMP and the monitor are Unix sockets under `target/`, or `-monitor none`; no TCP monitor.
3. No VNC or SPICE server (`-vnc`, `-spice`): a display, when needed, is a local window (`-display cocoa`) (*my call*: `-vnc :0` listens on every interface, as `-s` does).
4. Disk images live only under `target/`, attached with `readonly=on` or `snapshot=on` unless the guest must write.
5. `-nic none` unless a chunk needs a network; then user-mode networking, with every `hostfwd` bound to `127.0.0.1`.
6. No `xtask` command writes to a block device (`/dev/disk*`, `/dev/rdisk*`); flashing real hardware stays a manual step.
7. Every QEMU child has a deadline and a tag that a later run finds and kills, as M0.1's smoke boots do.

### 4.5 CONTRIBUTING "Host safety"

A section, after "Setting up", holding §4.4 and: deny rules and the sandbox are guardrails, with a link to `.claude/README.md`; `brew bundle --no-upgrade` for contributors who do not want `brew bundle` to upgrade formulae other projects use; long jobs (fuzzing, mutation testing) in a memory-capped container with a watchdog; a recent backup (Time Machine) before running agents unattended. "What CI enforces" gains `check-settings`, as does the CI rules list in `AGENTS.md` (which stays under 80 lines).

### 4.6 `.claude/README.md`

Records: the sandbox decision and its reason, the write paths, domains and exclusions with a reason each; that deny rules and the sandbox are guardrails, what each does not cover (the Read and Edit tools are outside the sandbox; excluded commands are outside it and see the environment's tokens; `!` commands are neither sandboxed nor checked); how to add an exclusion (a change to `EXCLUDED_ALLOWLIST` with its reason); and the follow-up issue for the agent token.

## 5. Requirements

- **M0.3-R1** `.claude/settings.json` MUST enable the sandbox with `allowUnsandboxedCommands: false` and `failIfUnavailable: true`, and the write paths, domains, read denies, credential denies and exclusions of §4.1, unless §4.1.7's fallback is taken and logged.
- **M0.3-R2** `permissions.deny` MUST hold every rule of §4.2.
- **M0.3-R3** In a Claude Code session in the repository, `sudo true` MUST be refused by the deny rule and reading a file under `~/.ssh/` with the Read tool MUST be refused.
- **M0.3-R4** In that session, a sandboxed write outside the allowed paths (`touch ~/m0.3-sandbox-probe`) MUST fail, and `cargo xtask ci` MUST pass sandboxed.
- **M0.3-R5** `cargo xtask check-settings` MUST fail on each case of §4.3 and pass on the committed settings, and the full `cargo xtask ci` MUST run it.
- **M0.3-R6** CONTRIBUTING MUST have a "Host safety" section with §4.4 and §4.5, and `.claude/README.md` MUST record §4.6.
- **M0.3-R7** The claim that `-s` listens on every interface and `-gdb tcp:127.0.0.1:1234` on loopback only MUST be checked with `lsof`, the commands and output recorded in `log.md`.

## 6. Cross-cutting concerns

- **`unsafe`:** n/a, because `check-settings` is ordinary `xtask` code and no crate gains `unsafe`.
- **Privilege and trust boundaries:** the boundary is between an agent's commands and the maintainer's host. The sandbox (Seatbelt) confines Bash and its children; permission rules cover the built-in tools; hooks, MCP servers and `!` commands run outside both. `.claude/settings.json` is trusted configuration, so a change that weakens it is caught by `check-settings` in CI, not at run time: an agent that edits it mid-session gets the weaker settings at once (Claude Code reloads settings files), which the README says. The sandbox itself write-protects the `.claude` settings files, `.claude/hooks`, `.claude/skills`, `.claude/agents`, and `.git/hooks` and `.git/config`, even inside the working directory (sandboxing docs), so a sandboxed command cannot change them; the Edit tool can, with the usual permission prompt.
- **Architecture portability:** n/a for the guest architectures. Host: the sandbox runs on macOS and Linux; Seatbelt behaviour (TLS, keychain, Hypervisor.framework) is measured on macOS only, and a Linux contributor may need other exclusions, which the README says. Paths such as `~/Library/Keychains` are macOS-only and harmless elsewhere.
- **Concurrency and memory ordering:** n/a, because nothing here runs concurrently.
- **Interfaces and ABI:** n/a; the settings file follows Claude Code's schema, which is not ours to freeze.
- **Hardware and specification assumptions:** Claude Code 2.1.332 and its docs (sandboxing, permissions, settings) as read on 2026-11-27; GitHub's token docs; QEMU 11.1.3 for the `lsof` check; macOS 26.6.2 on an M4 Pro.
- **Testing level:** host unit tests for `check-settings`; manual checks in a Claude Code session, recorded in `log.md`.
- **Performance:** n/a; the sandbox's cost on `cargo xtask ci` is recorded (wall time sandboxed and not) but not gated.
- **Documentation:** CONTRIBUTING "Host safety" and "What CI enforces", `.claude/README.md`, `xtask/README.md` (the new command), `AGENTS.md` (the CI rules line), `CHANGELOG.md`.
- **Policy changes:** a new CI-enforced check (`check-settings`, a step of the required `check-workflows` job); a CONTRIBUTING rule (host safety); deny rules and the sandbox in the shared agent settings. No required check, workflow permission or secret changes.

## 7. Alternatives considered

- **Deny rules only** (the issue's proposal): rejected as the main barrier, since the docs say they are not a security boundary and `/usr/bin/sudo` or `sh -c` pass them; kept as a second layer.
- **Sandbox opt-in per contributor:** rejected by the maintainer; kept as the fallback of §4.1.7.
- **A unit test instead of a command** for §4.3: workable, but a named command reports findings the way `check-workflows` does and can be run on its own (*my call*).
- **A fine-grained token for agents in this chunk:** split out, §2.

## 8. Verification

The evaluation is run in a Claude Code session in the repository with the sandbox settings applied; each command, whether it passed sandboxed, and any exclusion or path it needed, go in `log.md`: `cargo xtask ci`, `cargo xtask doctor`, `gh issue view 10`, `gh project item-list 8 --owner SpencerAWill --limit 1000 --format json`, `git fetch`, and `git push` of the chunk branch.

| Requirement | Verified by |
|---|---|
| R1 | the evaluation above; `cargo xtask check-settings` passing on the committed file |
| R2 | `check-settings` unit test: the committed file holds every required rule |
| R3 | in `log.md`: `sudo true` refused with the deny-rule message, and a Read of a file under `~/.ssh/` refused |
| R4 | in `log.md`: `touch ~/m0.3-sandbox-probe` failing with its error, and `cargo xtask ci` passing in the same session, with its wall time beside an unsandboxed run |
| R5 | unit tests, one per case of §4.3 (a missing deny rule, sandbox off, `allowUnsandboxedCommands` true, an unlisted exclusion, a bare `Bash` allow, `Bash(*)`, invalid JSON, a wrong type) and one on the committed file; `ci_steps` test lists `CheckSettings` |
| R6 | `review-docs`; `cargo xtask check-docs` |
| R7 | in `log.md`: `lsof -nP -iTCP:1234 -sTCP:LISTEN` while `qemu-system-x86_64 -S -s -nographic -monitor none -nic none` runs (expected `*:1234`), then with `-gdb tcp:127.0.0.1:1234` (expected `127.0.0.1:1234`); each QEMU killed after the check |

## 9. Done when

- [ ] `.claude/settings.json` holds the sandbox (or the logged fallback) and the deny rules, and `cargo xtask ci` passes.
- [ ] `cargo xtask check-settings` exists, is in `ci`, and has the tests of §8.
- [ ] `.claude/README.md` records the sandbox decision, its settings with reasons, and that deny rules are a guardrail.
- [ ] CONTRIBUTING has a "Host safety" section with the QEMU rules, `--no-upgrade`, long jobs and backups.
- [ ] The manual checks of R3, R4 and R7 are in `log.md` with commands and output.
- [ ] The agent-token follow-up issue is open and linked from #10's pull request.

## 10. Open questions

None.

## 11. Prior art and sources

- Claude Code docs, read 2026-11-27: [sandboxing](https://code.claude.com/docs/en/sandboxing), [permissions](https://code.claude.com/docs/en/permissions), [settings](https://code.claude.com/docs/en/settings).
- GitHub docs: [managing your personal access tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens) (fine-grained token limitations).
- QEMU documentation of `-gdb` and `-s` (to cite from `reference/` or the QEMU site in the work stage).
