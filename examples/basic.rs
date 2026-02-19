//! Basic example: connect with in-memory store and handle QR / Connected events.
//!
//! Run with: `cargo run --example basic`

use std::sync::Arc;
use whatsapp_pkg::{store::MemoryStore, Client};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();

    let store = Arc::new(MemoryStore::new());
    let client = Client::new(store);

    client
        .add_event_handler(|evt| match evt {
            whatsapp_pkg::Event::Qr { codes } => {
                println!(
                    "[Event] QR codes (scan with WhatsApp Linked Devices): {:?}",
                    codes
                );
            }
            whatsapp_pkg::Event::Connected => {
                println!("[Event] Connected and logged in.");
            }
            whatsapp_pkg::Event::PairSuccess { id, platform, .. } => {
                println!("[Event] Pair success: {} on {}", id, platform);
            }
            whatsapp_pkg::Event::Disconnected { reason } => {
                println!("[Event] Disconnected: {}", reason);
            }
            _ => {}
        })
        .await;

    println!("Connecting...");
    client.connect().await?;

    if client.is_logged_in() {
        println!("Already logged in.");
    } else {
        println!("No session: show the QR from the event above to link this device.");
    }

    Ok(())
}
