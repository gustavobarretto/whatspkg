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

## License

MPL-2.0 (same as whatsmeow). See [LICENSE](LICENSE).

## References

- [whatsmeow](https://github.com/tulir/whatsmeow) – Go library this project mirrors.
- [WhatsApp Web](https://web.whatsapp.com/) – multidevice client.
