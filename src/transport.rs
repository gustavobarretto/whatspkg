//! Transport abstraction for the connection layer.
//!
//! Implement this trait to plug in a WebSocket, Noise-wrapped socket, or other
//! transport. The client uses it for sending; receiving is typically handled by
//! a dedicated task that reads from the transport and feeds decoded nodes to the client.

use crate::Result;
use async_trait::async_trait;

/// Async trait for a connection transport (e.g. WebSocket, Noise over WebSocket).
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send raw bytes over the transport.
    async fn send(&self, data: &[u8]) -> Result<()>;

    /// Close the transport.
    async fn close(&self) -> Result<()>;
}
