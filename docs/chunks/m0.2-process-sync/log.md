# M0.2 log: Process sync with rust-engine-experiment

- **Status:** Living (2026-11-27). Log statuses: `Living (date)` while the chunk is open, `Historical: PR #n (date)` at close-out.

<!-- Append-only. One or two lines per entry, written when it happens, not at the end:
- <date> <tag> <text>
Tags: done, decision, surprise, dead-end, lesson? (a candidate lesson for stage 6; see docs/lessons/README.md). -->
- 2026-11-27 decision No M0 row was eligible for `/chunk next` (M0.0 and M0.1 done). Maintainer chose to add the M0.2 row (process sync) and start it, per the outline agreed on 2026-11-27; M0.3 (host safety, #10) and M0.4 (spike runner, #12) are to become chunk issues in this chunk's rollout, not roadmap rows.
- 2026-11-27 done Compared the engine's process files between the port (277e9b2) and origin/main e9e6156: M3.20 (#99, planning on GitHub, chores), #136/#138 (plan pause), builder, simplify and re-verify rules from M3.3 to M3.26, the grill's API check, six general lessons. Engine-only: the xtask bench/spike package lists, the `cargo hack` rule (this repo's ci runs no `cargo hack`), `ecs` and `perf` lessons.
- 2026-11-27 done GitHub today: #3 to #8 are maintainer-authored `M1.x` chunk issues, Status Scheduled, no Order, no milestone, sub-issues of tracking issue #2 (Inbox); #10 and #12 are maintainer-authored, Inbox, titled `chore: ...`; #12 says #10 lands first. No milestones exist; Project 8 has Status, Priority and Size, no Order.
- 2026-11-27 decision Maintainer: one chunk for the whole sync (planning on GitHub and the later process changes), one rollout.
- 2026-11-27 decision Maintainer: `docs/maintainers.md` requires one Project view, Next Up (Status Scheduled, sorted by Order, grouped by milestone); other views optional.
- 2026-11-27 done Draft spec written; typos and check-docs pass on it.
