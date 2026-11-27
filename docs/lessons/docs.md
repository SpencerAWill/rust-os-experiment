---
applies_to:
  - "AGENTS.md"
  - "CONTRIBUTING.md"
  - "docs/**"
  - ".claude/**"
---
# Lessons: documentation

- Status: Living (2026-11-27)

- **docs-1.** A rule that several entry files need (the waits, the escalations) has one copy; the other files link to it. *Why:* the escalation list was written three times with three wordings within one chunk (rust-engine-experiment M3.15). *From:* rust-engine-experiment M3.15 · *Confirmed:* 2026-11-15
- **docs-2.** A header field or section a stage tells agents to add belongs in the template they copy (`docs/chunks/_template/`), so its format has one copy; when a fix changes how many items a document lists, grep the plan for the old count. *Why:* chunk stage 1 told agents to add an `**Issue:**` line the template lacked, and plan step 6 still said "the two built-in workflows" after the setup grew to four (rust-engine-experiment M3.18). *From:* rust-engine-experiment M3.18 · *Confirmed:* 2026-11-16
- **docs-3.** A research note's "measured here" claim points at the `log.md` entry that holds that command's output, and the note cites no plan step, deviation or decision of the chunk (it must read on its own after the plan is historical). A literal name `typos` rejects (a tag, a commit title) is described or allow-listed, never respelt with "(sic)". *Why:* in M0.1 three "measured here" commands had no output in the log, other claims gave only "(measured here)" so a reviewer had to search the log, two sentences cited plan steps, and a respelt tag name could no longer be found. *From:* M0.1 · *Confirmed:* 2026-11-27
