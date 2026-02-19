//! Length-prefixed framing over WebSocket.
//! Each WebSocket binary message = one frame: 3-byte BE length + payload.

use crate::error::{ConnectionError, Error};
use crate::Result;
use async_trait::async_trait;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

use super::{read_frame_len, write_frame_len, MAX_FRAME_SIZE};

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Send half of a framed WebSocket: implements Transport.
pub struct FramedSend {
    writer: Mutex<futures::stream::SplitSink<WsStream, Message>>,
}

impl FramedSend {
    pub(crate) fn new(writer: futures::stream::SplitSink<WsStream, Message>) -> Self {
        Self {
            writer: Mutex::new(writer),
        }
    }

    /// Write one frame as a single WebSocket binary message: 3-byte length + body.
    pub async fn send_frame(&self, data: &[u8]) -> Result<()> {
        if data.len() > MAX_FRAME_SIZE {
            return Err(Error::Binary("frame too large".into()));
        }
        let mut msg = Vec::with_capacity(3 + data.len());
        msg.resize(3, 0);
        write_frame_len(&mut msg, data.len());
        msg.extend_from_slice(data);
        let mut w = self.writer.lock().await;
        w.send(Message::Binary(msg))
            .await
            .map_err(|e| Error::Connection(ConnectionError::WebSocket(e.to_string())))?;
        Ok(())
    }
}

#[async_trait]
impl crate::transport::Transport for FramedSend {
    async fn send(&self, data: &[u8]) -> Result<()> {
        self.send_frame(data).await
    }

    async fn close(&self) -> Result<()> {
        let mut w = self.writer.lock().await;
        w.close()
            .await
            .map_err(|e| Error::Connection(ConnectionError::WebSocket(e.to_string())))?;
        Ok(())
    }
}

/// Receive half: each WebSocket binary message is one frame (3-byte len + body); returns body only.
pub struct FramedRecv {
    reader: Mutex<futures::stream::SplitStream<WsStream>>,
}

impl FramedRecv {
    pub(crate) fn new(reader: futures::stream::SplitStream<WsStream>) -> Self {
        Self {
            reader: Mutex::new(reader),
        }
    }

    /// Read the next frame. Returns the payload (message with 3-byte length prefix stripped).
    pub async fn next_frame(&self) -> Result<Vec<u8>> {
        let mut r = self.reader.lock().await;
        let msg = r
            .next()
            .await
            .ok_or_else(|| Error::Connection(ConnectionError::Disconnected))?
            .map_err(|e| Error::Connection(ConnectionError::WebSocket(e.to_string())))?;
        let data = match msg {
            Message::Binary(d) => d,
            Message::Close(_) => return Err(Error::Connection(ConnectionError::Disconnected)),
            _ => {
                return Err(Error::Connection(ConnectionError::WebSocket(
                    "expected binary frame".into(),
                )))
            }
        };
        if data.len() < 3 {
            return Err(Error::Binary("frame too short".into()));
        }
        let len = read_frame_len(&data[..3]);
        if data.len() != 3 + len {
            return Err(Error::Binary("frame length mismatch".into()));
        }
        if len > MAX_FRAME_SIZE {
            return Err(Error::Binary("frame length too large".into()));
        }
        Ok(data[3..].to_vec())
    }
}
