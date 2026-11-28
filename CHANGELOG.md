# Changelog

All notable changes are listed here, newest first. The project has no releases yet.

## Unreleased

- M0.3: host-safety guardrails: the Claude Code sandbox on in the shared settings (with `gh` and `git push` as the only exclusions), deny rules for system, Homebrew and credential access, `cargo xtask check-settings` in CI, and CONTRIBUTING "Host safety" with the rules later QEMU commands follow.
- M0.2: process sync with rust-engine-experiment: planning state moves to GitHub (chunk issues, milestones M0 to M5 in Pulumi, the Project's Order field, "blocked by" links), and the roadmap keeps milestone goals and history only; `/chunk next` reads GitHub; `/chunk chore #N` runs small scheduled build work; a pause after the plan; newer builder, simplify, re-verify and grill rules; six lessons ported.
- M0.1: development environment: bare-metal targets and tool components in the toolchain, a `Brewfile`, `cargo xtask doctor` (host checks and QEMU smoke boots), the reference library (`reference/`, `cargo xtask refs`), and the development environment research note.
- M0.0: repository foundations: the chunk workflow (build and research chunks), agent roles and skills, the push guard, `cargo xtask` process checks, CI, repository settings in Pulumi, ADRs 0001 and 0002, and research notes seeded from the initial architecture conversation.
