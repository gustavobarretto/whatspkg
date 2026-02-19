//! # whatsapp-pkg
//!
//! Rust library for the WhatsApp web multidevice API.
//!
//! ## Features
//!
//! - Sending messages to private chats and groups (text and media)
//! - Receiving messages and events
//! - QR code pairing (multidevice)
//! - Group management and invite links
//! - Typing indicators, delivery/read receipts
//! - App state (contacts, pin/mute)
//! - Retry receipts for decryption failures
//!
//! ## Example
//!
//! ```ignore
//! use std::sync::Arc;
//! use whatsapp_pkg::{Client, store::MemoryStore};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let store = Arc::new(MemoryStore::new());
//!     let client = Client::new(store);
//!     client.connect().await?;
//!     // Handle QR or existing session...
//!     Ok(())
//! }
//! ```

pub mod binary;
pub mod client;
pub mod error;
pub mod events;
pub mod pairing;
pub mod socket;
pub mod store;
pub mod transport;
pub mod types;

pub use client::{Client, CompletePairingParams, SendRequestExtra, SendResponse};
pub use error::{Error, Result};
pub use events::Event;
pub use pairing::{
    generate_pairing_keys, sign_device_identity, verify_device_identity, verify_signed_identity,
    PairingKeys, VerifiedIdentity,
};
pub use store::{Device, DeviceStore, Store};
pub use transport::Transport;
pub use types::{Jid, MessageId};
