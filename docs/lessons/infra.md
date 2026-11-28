---
applies_to:
  - "infra/github/**"
---
# Lessons: repository configuration (Pulumi)

- Status: Living (2026-11-27)

- **infra-1.** A Pulumi resource that other GitHub data points at (a milestone, which issues reference) is declared with `retainOnDelete: true` and a comment saying never to remove its entry, and any written undo step for it deletes it explicitly, since `pulumi up` then only drops it from Pulumi's state. *Why:* without it, a `pulumi up` from a revision that lacks the resource (`main` before the merge, or an entry tidied away later) deletes the milestone and unassigns every issue in it; the engine's M3.20 abandonment undo relied on that deletion until the code review caught it. *From:* rust-engine-experiment M3.20 · *Confirmed:* 2026-11-27
