# Running existing software

Status: Reference: not a decision (2026-11-17).

> **Provenance.** Distilled on 2026-11-17 from a conversation the maintainer had with an AI model (Google Gemini) in November 2026 ([transcript](sources/gemini-architecture-conversation.txt)), rewritten in our own words. Nothing here is verified unless a source is cited; the conversation cited none. Code from the conversation is not reused ([ADR 0001](../adr/0001-licence-and-originality.md)). Research chunks check these claims before a decision relies on them.

Because domains run side by side, compatibility can live in its own domain or personality, away from a clean experimental kernel. Four ways, easiest first, as described:

| Way | How | Effort claimed | Trade-off |
|---|---|---|---|
| WebAssembly (WASI) | Embed a Rust Wasm runtime as a service or library OS; implement the WASI calls on our rings | Days to weeks | Many existing programs compile to it; sandboxed; no native binaries |
| Source compatibility | Port a small C library (musl, newlib) and add a Rust `std` target whose `File`, `TcpStream` and threads map to our capabilities and rings | Weeks to months | Recompiled programs become native; needs source |
| Linux system-call personality | Catch Linux system calls and translate them (as WSL 1, FreeBSD's Linuxulator, Fuchsia's Starnix) | Months | Runs unmodified binaries; `fork`, file descriptors and `epoll` are hard to map |
| Paravirtualized Linux domain | Run a small Linux in one domain, talking over shared rings (as Qubes OS, ChromeOS Crostini) | Shortest path to full compatibility | Depends on the virtualization model chosen in M1.1 |

Expected speed, as claimed: CPU-bound programs near native; I/O-heavy programs written against blocking POSIX calls do not get faster through an adapter, only when rewritten for the rings.

## Corrections

- Effort and speed figures are unsourced.
