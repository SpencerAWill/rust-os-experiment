# M0.3 log: Host-safety guardrails for agents and later QEMU commands

- **Status:** Living (2026-11-27). Log statuses: `Living (date)` while the chunk is open, `Historical: PR #n (date)` at close-out.

<!-- Append-only. One or two lines per entry, written when it happens, not at the end:
- <date> <tag> <text>
Tags: done, decision, surprise, dead-end, lesson? (a candidate lesson for stage 6; see docs/lessons/README.md). -->
- 2026-11-27 done `/chunk next`: M0 is the current milestone; #10 (M0.3, Order 20) is next and unblocked; #12 (M0.4) is blocked by #10; no scheduled chores. Branch `m0.3-host-safety` pushed.
- 2026-11-27 done Read #10 and the maintainer's comment on it (agent token limited to this repository). The account owns 45 repositories (32 private), all reachable with the current broad `gh` token; this repository is public.
- 2026-11-27 surprise GitHub's token docs list "Projects owned by a user account" as a fine-grained PAT limitation; the chunk loop reads and writes user-owned Project 8, so a fine-grained token alone cannot run the loop (docs.github.com, "Managing your personal access tokens"; to re-check in the work stage).
- 2026-11-27 surprise Claude Code docs (code.claude.com/docs/en/permissions): Bash deny rules match the command text after stripping `timeout`, `nice`, `nohup` and similar, but not `env`, `sh -c` or an absolute path such as `/usr/bin/sudo`; the docs say they "aren't a security boundary". The sandbox (Seatbelt on macOS) limits writes to the working directory and a temp directory and sends network through an allowlist proxy, applies to Bash and its children only (not Read/Edit/Write), and lists `gh` on macOS as failing TLS verification under it.
- 2026-11-27 decision Maintainer: the sandbox is on in the shared `.claude/settings.json`, tested in the work stage against `cargo xtask ci`, `doctor`, `gh` and `git push`; commands that cannot run sandboxed become named exclusions; opt-in is the fallback only if the loop cannot run under it.
- 2026-11-27 decision Maintainer: the agent token (comment on #10) is split out to its own issue, opened at close-out; this chunk does not change how agents authenticate to GitHub.
- 2026-11-27 decision Maintainer: add credential protection (Read/Edit denies for credential paths, `security`, sandbox read denies and masked token variables), more system-command denies (`tmutil`, `csrutil`, `nvram`, `osascript`, `shutdown`, `reboot`, state-changing `brew`), and a CI check that keeps the required settings.
- 2026-11-27 decision Maintainer: the M2 gdb stub listens on loopback TCP (`-gdb tcp:127.0.0.1:<port>`), with the `lsof` check of bare `-s`.
- 2026-11-27 done Draft spec written.
- 2026-11-27 decision Grill round 1, cold read by `spec-reader` (dispatched by agent name). Main session's calls: `check-settings` runs as a second step of the required `check-workflows` job (no ruleset change); `Read` deny rules only (the docs say they also block Edit and Write). Facts from the Claude Code docs written in: exclusions apply only when every command of a call matches; the sandbox write-protects `.claude` settings, hooks, skills, agents and `.git/hooks`, `.git/config`; `credentials` `mask` entries are dropped from project settings, `deny` entries are honoured.
