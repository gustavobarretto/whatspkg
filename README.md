# whatspkg

Rust library for the **WhatsApp web multidevice API**, modeled after [whatsmeow](https://github.com/tulir/whatsmeow) (Go).

## Relation to whatsmeow

This crate mirrors the design and public API of [tulir/whatsmeow](https://github.com/tulir/whatsmeow):

- **Same architecture**: `Client`, device `Store`, `binary` nodes, `events`, `types` (JID, MessageId).
- **Same features** (when fully implemented): QR pairing, send/receive messages (text & media), groups, receipts, app state, retry receipts.
- **Native Rust**: async/await (Tokio), no Go dependency.

The WhatsApp Web protocol (Noise handshake, binary nodes, E2E with Signal) is complex; this repo provides the structure and types. Full protocol implementation (binary encoding, Noise socket, Signal crypto) can be added incrementally or via collaboration.

## Features

- **Types**: `Jid`, `MessageId`, event enums (QR, Connected, Message, Receipt, etc.).
- **Store**: `DeviceStore` trait + in-memory implementation; pluggable persistence.
- **Client**: `Client::new(store)`, `connect()`, `disconnect()`, `add_event_handler()`, `generate_message_id()`, `complete_pairing()`.
- **Binary**: `Node` type for protocol messages (encode/decode stubs for now).
- **Errors**: Typed errors (`ConnectionError`, `PairingError`, `StoreError`, `SendError`).

## Usage

```toml
[dependencies]
whatspkg = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use std::sync::Arc;
use whatspkg::{Client, store::MemoryStore};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store = Arc::new(MemoryStore::new());
    let client = Client::new(store);

    client.add_event_handler(|evt| {
        match evt {
            whatspkg::Event::Qr { codes } => println!("Scan QR: {:?}", codes),
            whatspkg::Event::Connected => println!("Connected"),
            whatspkg::Event::Message(m) => println!("Message from {}: {}", m.from, m.id),
            _ => {}
        }
    }).await;

    client.connect().await?;
    // If no session: QR event is emitted. After pairing, use client.complete_pairing(...).
    Ok(())
}
```

## Module map (vs whatsmeow)

| whatsmeow (Go)     | whatspkg (Rust)   |
|-------------------|-------------------|
| `client.go`       | `client/`         |
| `types/jid.go`    | `types/jid.rs`    |
| `types/events/`   | `events/mod.rs`   |
| `store/`          | `store/`          |
| `binary/`         | `binary/mod.rs`   |
| `socket/`         | (future)          |
| `send.go`         | `client/send.rs`  |

## To-Do (full implementation)

The following list is need to implement to reach full feature. Items are ordered by dependency where it helps.

| Area | Task | Reference (whatsmeow) | Notes |
|------|------|----------------------|--------|
| **Binary protocol** | Implement `Node::encode()` and `Node::decode()` for the custom binary XML-like format. | `binary/` | Required for all wire communication. |
| **Socket layer** | Add WebSocket client (e.g. `tokio-tungstenite`) and frame binary nodes over the connection. | `socket/` | Connects to WhatsApp Web endpoints. |
| **Noise protocol** | Implement Noise handshake and transport; encrypt/decrypt frames before/after WebSocket. | `socket/`, handshake | Use a Noise crate (e.g. `snow`) compatible with WhatsApp’s pattern. |
| **Pairing crypto** | Complete `complete_pairing()`: verify device identity (HMAC/signatures), generate device signature, persist identity. | `pair.go`, `handshake.go`, `util/keys` | Needs ECC/signing (e.g. X25519, Ed25519) and HMAC. |
| **Signal / E2E** | Integrate Signal protocol: session setup, prekeys, identity store, encrypt/decrypt message payloads. | `go.mau.fi/libsignal`, whatsmeow usage | Use a Rust Signal impl or bindings; store identities per `store::DeviceStore`. |
| **Protobuf** | Add WhatsApp protobuf definitions (waE2E, waWeb, etc.), generate Rust with `prost` (or similar). | `proto/` | Needed for message content, app state, and server nodes. |
| **Real connect** | Wire socket + Noise + binary nodes into `Client`: open connection, handle stream, emit Connected / Disconnected. | `client.go`, `connectionevents.go` | Replace stub `connect()` / `send_node()`. |
| **Real pairing** | Emit real QR payloads from server; handle pair-device / pair-success; call `complete_pairing()` with parsed data. | `pair.go`, `qrchan.go` | Depends on binary + socket + pairing crypto. |
| **Send message** | Implement `send_message()` over the wire: build E2E message, send node, wait for ack. | `send.go`, `message.go` | Depends on Signal, binary, socket. |
| **Receive messages** | Decode incoming nodes, decrypt E2E payloads, emit `Event::Message` (and related). | `message.go`, handlers in `client.go` | Depends on binary, socket, Signal, protos. |
| **Receipts** | Send and handle delivery/read receipts; emit `Event::Receipt`. | `receipt.go` | Depends on node send/receive. |
| **Groups** | Group metadata, participants, invite links, group messages. | `group.go` | Depends on nodes + protos. |
| **App state** | Read/write app state (contacts, pin/mute, etc.). | `appstate/`, app state nodes | Depends on nodes + protos. |
| **Retry receipts** | Handle retry requests when decryption fails; resend or provide plaintext. | `retry.go`, `GetMessageForRetry` | Depends on message send/receive. |
| **Persistent store** | Implement `DeviceStore` (and identity/session stores) backed by SQLite or similar. | `store/sqlstore` | Enables restarts without re-pairing. |

Contributors can pick any item and implement it step by step; see [CONTRIBUTING.md](CONTRIBUTING.md) for how to open issues and PRs.

## Contributing

See **[CONTRIBUTING.md](CONTRIBUTING.md)** for how to contribute and where to start. Pull requests run CI on every push:

- **Build & test**: `cargo build`, `cargo test`, `cargo fmt --check`, `cargo clippy`
- **Version bump**: When the PR targets `main`/`master`, the version in `Cargo.toml` must be **greater** than the base branch (e.g. `0.1.0` → `0.1.1`). The workflow fails if the version is unchanged or lower.

To **require approval** (and ensure **only admins** can approve):

1. **Code owners** – Edit [`.github/CODEOWNERS`](.github/CODEOWNERS) and replace `@YOUR_GITHUB_USERNAME` with your GitHub username (or one per line for multiple admins). Only these accounts will count as valid reviewers when the rule below is enabled.
2. **Branch protection** – GitHub repo → **Settings** → **Branches** → add or edit the rule for `main` (or `master`):
   - Enable **Require a pull request before merging** and set **Required number of approvals** (e.g. 1).
   - Enable **Require review from Code Owners**. Merges then require approval from someone listed in `CODEOWNERS` (your admins).
   - Optionally enable **Require status checks to pass** and select **Build & test** and **Version incremented**.
   - Leave **Do not allow bypassing the above settings** with no bypass list so even admins must use a PR.

## License

MPL-2.0. See [LICENSE](LICENSE).

## References

- [whatsmeow](https://github.com/tulir/whatsmeow) – Go library this project mirrors.
- [WhatsApp Web](https://web.whatsapp.com/) – multidevice client.
