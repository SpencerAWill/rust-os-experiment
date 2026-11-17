# M0.0 log: repository foundations

- **Status:** Living (2026-11-17).

- 2026-11-17 decision Maintainer: approach A (port the whole process layer in one bootstrap PR), name `rust-os-experiment`, public, commits with the personal email, research chunks as a variant of the loop built in the bootstrap, no engine ADRs, transcript distilled into notes (not committed).
- 2026-11-17 decision Maintainer asked to keep token use low: the plan's builder steps (xtask, Pulumi) are done by the main session, the skill trial and the multi-agent review are cut down (see the plan's Deviations).
- 2026-11-17 done Repository created (`gh repo create --add-readme`), branch `m0.0-foundations`.
- 2026-11-17 decision `.github` left out of `REQUIRED_READMES`: GitHub shows `.github/README.md` instead of the root README on the repository page.
- 2026-11-17 surprise `check-docs` only sees tracked files, so new files must be staged before it can pass.
- 2026-11-17 done `pulumi preview` on stack `SpencerAWill/rust-os-github/prod`: repository `= import` then `~ update` (merge settings, wiki, discussions, topics), 8 to create, no replace or delete. Needed `pulumi config set github:owner SpencerAWill` (now in `Pulumi.prod.yaml`).
