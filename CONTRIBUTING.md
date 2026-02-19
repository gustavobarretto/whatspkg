# Contributing to whatspkg

Thanks for your interest in helping complete this library. The codebase is structured to mirror [whatsmeow](https://github.com/tulir/whatsmeow); the main work is implementing the protocol and wiring it into the existing types and client.

## What’s already done

- **Types**: `Jid`, `MessageId`, events, errors, `Device` / `DeviceStore`, binary `Node` structure.
- **Client API**: `connect`, `disconnect`, event handlers, `generate_message_id`, `send_message` (stub), `complete_pairing` (partial).
- **Store**: `DeviceStore` trait and in-memory `MemoryStore`.
- **Tests**: Unit tests for JID, store, and client behavior.

## What’s left: To-Do list

See the **[To-Do (full implementation)](README.md#to-do-full-implementation)** section in the README for the full list of items the community needs to implement. It covers:

- Binary protocol encode/decode
- WebSocket + Noise socket layer
- Signal/E2E and pairing crypto
- Protobuf definitions and codegen
- Persistent store (e.g. SQLite)
- Real connect, pairing, send/receive, groups, receipts, app state, retries

Pick an item that matches your skills (e.g. binary format, crypto, async I/O) and open an issue or PR.

## How to contribute

1. **Open an issue** (optional) – Describe what you want to do so others can avoid duplicate work.
2. **Fork and branch** – Create a branch from `main` (e.g. `feat/binary-encode`).
3. **Implement** – Follow existing style; add tests where possible.
4. **Bump version** – When your PR targets `main`, increment the version in `Cargo.toml` (e.g. `0.1.0` → `0.1.1`). CI will fail if you don’t.
5. **Open a PR** – CI runs tests, `cargo fmt`, `cargo clippy`, and the version check. Fix any failures.
6. **Review** – Maintainers (or the community) will review. Merge requires approval if branch protection is enabled.

## Code style

- Run `cargo fmt` and `cargo clippy` before pushing. CI enforces this.
- Prefer the existing patterns: `crate::Result`, typed errors, async/await with Tokio.
- When adding protocol or crypto, add references (e.g. whatsmeow file or commit) in comments or PR description so others can verify behavior.

## Reference: whatsmeow

Use [whatsmeow](https://github.com/tulir/whatsmeow) as the reference implementation:

- **Binary protocol**: `binary/` (reader/writer, node encoding).
- **Socket**: `socket/` (Noise handshake, framing).
- **Pairing**: `pair.go`, `pair-code.go`, `handshake.go`.
- **Send/receive**: `send.go`, `message.go`, `receipt.go`.
- **Store**: `store/` (device, identity, session).
- **Protos**: `proto/` (`.proto` or generated Go; you’ll need Rust equivalents, e.g. with `prost`).

If you’re unsure how something should behave, check the corresponding whatsmeow code and document the mapping in your PR.

## Questions

- Open a **Discussion** or **Issue** for design questions or “how do I…”.
- For protocol/format details, whatsmeow’s [WhatsApp protocol Q&A](https://github.com/tulir/whatsmeow/discussions/categories/whatsapp-protocol-q-a) may also help.
