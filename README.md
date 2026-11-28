# rust-os-experiment

An experiment in operating-system architecture in Rust: an exokernel that multiplexes raw hardware, an interposition "shim" at its trap boundary, and microkernels or library OSes running side by side on top of it. Targets x86_64, aarch64 and riscv64, run under QEMU.

## Status

Experimental, and nothing boots yet. The repository starts with its workflow and its research: M1 answers the architecture questions (where each layer runs, the boot path, the exokernel interface, where the shim sits) before any kernel code is written. See [docs/roadmap.md](docs/roadmap.md) for the milestones, the [GitHub milestones](https://github.com/SpencerAWill/rust-os-experiment/milestones) for their chunks, and [docs/research/](docs/research/README.md).

## How work is done

Work is planned as milestones of small chunks, one pull request each, each with a spec, a plan and a log ([docs/workflow.md](docs/workflow.md)). The rules are in [CONTRIBUTING.md](CONTRIBUTING.md); decisions are ADRs in [docs/adr/](docs/adr/README.md). AI assistants are used throughout, under the rules in [AGENTS.md](AGENTS.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
