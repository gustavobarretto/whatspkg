//! WebSocket client connection (requires `full` feature).

use crate::client::DEFAULT_WS_URL;
use crate::error::{ConnectionError, Error};
use crate::Result;
use futures::stream::StreamExt;
use tokio_tungstenite::connect_async;

use super::framed::{FramedRecv, FramedSend};

/// Connect to the given WebSocket URL and return framed send/recv halves.
/// Default URL is `crate::client::DEFAULT_WS_URL` (`wss://web.whatsapp.com/ws`).
pub async fn connect(url: &str) -> Result<(FramedSend, FramedRecv)> {
    let (ws_stream, _response) = connect_async(url)
        .await
        .map_err(|e| Error::Connection(ConnectionError::WebSocket(e.to_string())))?;
    let (write_half, read_half) = ws_stream.split();
    Ok((FramedSend::new(write_half), FramedRecv::new(read_half)))
}

/// Connect to the default WhatsApp Web WebSocket URL.
pub async fn connect_default() -> Result<(FramedSend, FramedRecv)> {
    connect(DEFAULT_WS_URL).await
}
