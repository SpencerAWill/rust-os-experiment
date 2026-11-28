# .claude/

Status: Living (2026-11-28).

Claude Code configuration shared by everyone who works on this repository: the agent roles, skills and the push guard that run the chunk loop of [docs/workflow.md](../docs/workflow.md). Everything here is plain Markdown or shell, readable without Claude Code.

## When this applies

- You run a chunk with Claude Code: `/chunk next` starts the next scheduled one (read from GitHub), `/chunk resume` continues one, `/chunk chore #N` runs a scheduled chore, `/grill <doc>` interrogates a design document, `/triage` defines and orders the issue Inbox.
- You change how agents work here: a role, a skill, a permission or a hook.

## Contents

- `agents/`: one file per role, each pinning its model (including `researcher` and `review-sources` for research chunks); the table is in [docs/workflow.md](../docs/workflow.md#roles).
- `skills/chunk/`: the loop. `SKILL.md` dispatches `next`, `resume` and single stages; `stages/` holds one file per stage, so only the current one is read.
- `skills/grill/`: the grill stage, usable on any design document.
- `skills/triage/`: triage and define the GitHub issue Inbox with only `gh` and file reads, so it can run while a long job occupies the machine; writes only after confirmation.
- `skills/lessons-refresh/`: the milestone pass over [docs/lessons/](../docs/lessons/).
- `rules/`: one pointer per area (`xtask/`, `docs/`, `spikes/`, ...). Claude Code loads a rule when it touches a file matching the rule's `paths:`; the rule names the README to read and says to follow the matching lessons. Rules hold no rule text of their own: the README or `CONTRIBUTING.md` does.
- `hooks/guard-push.sh`: blocks agent pushes to `main` and force pushes; the logic is `cargo xtask guard-push`.
- `settings.json`: the hook, an allowlist of check commands whose flags `xtask` itself restricts (no bare `cargo test *`: cargo's `--config` and `--manifest-path` flags can run any program), deny rules, and the sandbox ([below](#sandbox-and-deny-rules)). No model is set, so each contributor keeps their own.
- `README.md`: this file.

Local state is ignored by git: `settings.local.json` (personal overrides), `worktrees/`, `agent-memory-local/`.

## Sandbox and deny rules

Decided in M0.3 ([spec](../docs/chunks/m0.3-host-safety/spec.md), measurements in its [log](../docs/chunks/m0.3-host-safety/log.md)); the host-safety rules are in [CONTRIBUTING](../CONTRIBUTING.md#host-safety). `cargo xtask check-settings` fails CI if a setting below is removed or relaxed without an allowlist entry.

**The sandbox is on for everyone** (Seatbelt on macOS), so an agent's Bash command and every process it starts can write only inside the repository, the session's temp directory and the paths below, and reach unlisted hosts only with approval. `allowUnsandboxedCommands: false`: a command that fails in the sandbox is not retried outside it. `failIfUnavailable: true`: a session whose sandbox cannot start stops.

| Setting | Entry | Reason |
|---|---|---|
| `filesystem.allowWrite` | `~/.cargo/advisory-dbs` | `cargo deny` locks and updates its advisory database there |
| `network.allowedDomains` | `github.com` | `cargo deny` fetches `RustSec/advisory-db`; `git fetch` |
| `excludedCommands` | `gh api *`, `gh issue *`, `gh pr *`, `gh project *`, `gh label *`, `gh search *`, `gh auth status` | `gh` reads its token from `~/.config/gh` and the keychain, which the sandbox denies; only the subcommands the loop uses (`gh extension` and `gh alias` would run arbitrary code) |
| `excludedCommands` | `git push origin *`, `git push -u origin *` | the credential helper's store (the keychain via `osxkeychain` on macOS) is denied to the sandbox; pinned to `origin`, since `git push --receive-pack=<cmd>`, `--exec=<cmd>` or a push to a local path runs any command, and `guard-push` refuses those flags and other remotes |
| `filesystem.denyRead` | SSH, AWS, GPG, `gh`, keychain, netrc, git and cargo credential paths | sandboxed commands cannot read credentials |
| `credentials.envVars` (`deny`) | `GH_TOKEN`, `GITHUB_TOKEN`, `GH_ENTERPRISE_TOKEN`, `CARGO_REGISTRY_TOKEN` | sandboxed commands do not see a token in the session's environment |

`network.allowMachLookup` is empty: `com.apple.sysmond` would let `pgrep` work, but also let every sandboxed command list the host's processes, and `doctor`'s leftover sweep still could not kill anything ([CONTRIBUTING](../CONTRIBUTING.md#host-safety)).

**Deny rules** (`permissions.deny`) refuse `sudo`, disk, boot, launchd, keychain and AppleScript commands and Homebrew state changes; Read of credential paths (which also blocks Edit and Write there, and becomes a sandbox read deny); Edit of `.git/hooks/**` and `.git/config`, which the excluded `git push` would run; and Edit of `.claude/settings.local.json`, which overrides this file where CI cannot see it (Claude Code's own writes there, such as the `/sandbox` panel, are not Edit calls). They match the usual form of a command: `/usr/bin/sudo`, `sh -c '...'` and `env sudo` pass them.

**What is not covered:**

- The Read, Edit and Write tools are outside the sandbox; only the deny rules limit them.
- Excluded commands run outside the sandbox with the `gh` token and the full environment, and read every file and reach every host the user can: `gh issue create --body-file <file>`, `gh api -F k=@<file>` and `gh api --hostname <host>` can send a credential file anywhere (accepted, spec §6). `gh api` can change any repository the token reaches; an agent token limited to this repository is a follow-up issue.
- The push-guard hook runs outside the sandbox and builds `xtask` from the working tree, which sandboxed commands can write, so code written there runs on the host at the next Bash call that mentions `push`. A follow-up issue redesigns the hook.
- The guard checks the push command's text, not the repository `git` acts on: a planted `.git/commondir`, or a nested repository and a `cd` into it, makes the excluded `git push origin <branch>` (and `gh pr checkout` or `merge`, which run local `git`) run planted hooks outside the sandbox. Recorded for the same follow-up.
- `!` commands are neither sandboxed nor checked, and neither are hooks or MCP servers.
- An agent that edits `settings.json` with the Edit tool gets the new settings within a command (M0.3 log: every settings change of that chunk was an Edit tool call, without a prompt in auto mode, and reached the sandbox one command late); CI catches a weakening edit, the session does not.
- The table was measured on macOS only. On Linux and WSL2 the sandbox needs `bubblewrap` and `socat` (and `failIfUnavailable` stops a session without them); native Windows has no sandbox, so use WSL2. A `filesystem` entry for a path that does not exist yet has no effect there (Claude Code settings reference, `sandbox.filesystem.allowWrite`; not measured, no Linux host), so run `! cargo deny fetch` once before the first session; a `CARGO_HOME` other than `~/.cargo` moves the advisory database out of the allowed path. Other exclusions may be needed.
- **Cold caches.** Only the advisory database is writable outside the repository, so a new or changed dependency, a fresh clone, or a toolchain change in `rust-toolchain.toml` fails sandboxed with a read-only error. Run `! cargo fetch` or `! rustup show` first. `~/.cargo/registry` and `~/.rustup` are not allowlisted because a sandboxed command could then rewrite crate sources and toolchains that other projects build and run.

**Working with exclusions.** An exclusion applies only when every command of a Bash call matches it, from the start of the command text. Run an excluded command alone and in its plain form: `git push origin <branch>`, not `cd x && git push` or `git -C x push`; `gh ... --jq '<filter>'`, not `gh ... | jq` (the pipe case follows the sandboxing docs; the `cd` and `-C` cases were measured). Otherwise it runs sandboxed and fails with an error such as `open ~/.config/gh/config.yml: operation not permitted`. To add one, add it to `excludedCommands` and to `EXCLUDED_ALLOWLIST` in `xtask/src/check_settings.rs` with the failure it fixes, in the same pull request; try a write path or domain first.

**Network.** In auto mode an unlisted host goes to the classifier, not to a prompt. `network.strictAllowlist: true`, which refuses unlisted hosts outright, has no effect in a repository's settings file; set it in your `~/.claude/settings.json` if you want it.

## How to

- **Change a role or skill:** edit the Markdown in a chunk's pull request; lessons that change how agents work may be edited the same way and are listed in its Lessons section.
- **New agent files** are loaded when a session starts. Until then, dispatch a general-purpose agent with the file's model and "follow `.claude/agents/<role>.md`" as its prompt.
