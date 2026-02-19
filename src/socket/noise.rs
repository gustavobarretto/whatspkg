//! Noise protocol handshake and transport (WhatsApp: XX_25519_AESGCM_SHA256).
//! Requires `full` feature.

use crate::binary::{NOISE_START_PATTERN, WA_CONN_HEADER};
use crate::error::{ConnectionError, Error};
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::framed::{FramedRecv, FramedSend};

/// Noise pattern used by WhatsApp Web.
const NOISE_PATTERN: &str = "Noise_XX_25519_AESGCM_SHA256";

/// Prologue hashed into the handshake: WA header + pattern string (must match server).
fn prologue() -> Vec<u8> {
    let mut p = Vec::with_capacity(WA_CONN_HEADER.len() + NOISE_START_PATTERN.len());
    p.extend_from_slice(&WA_CONN_HEADER);
    p.extend_from_slice(NOISE_START_PATTERN);
    p
}

/// Run the Noise XX handshake as initiator over the framed WebSocket.
/// First frame sent is: prologue (WA header + pattern) + first handshake message.
/// Consumes the framed send/recv and returns Noise transport and recv halves.
pub async fn run_client_handshake(
    send: FramedSend,
    recv: FramedRecv,
) -> Result<(NoiseTransport, NoiseRecv)> {
    let prologue = prologue();
    let params = NOISE_PATTERN.parse().map_err(|e: snow::Error| {
        Error::Connection(ConnectionError::WebSocket(format!("noise params: {}", e)))
    })?;
    let mut handshake = snow::Builder::new(params)
        .prologue(&prologue[..])
        .map_err(|e| Error::Connection(ConnectionError::WebSocket(format!("noise init: {}", e))))?
        .build_initiator()
        .map_err(|e| {
            Error::Connection(ConnectionError::WebSocket(format!("noise build: {}", e)))
        })?;

    // XX: initiator sends e
    let mut msg_buf = [0u8; 65535];
    let len = handshake.write_message(&[], &mut msg_buf).map_err(|e| {
        Error::Connection(ConnectionError::WebSocket(format!("noise write: {}", e)))
    })?;
    let mut first_frame = prologue.clone();
    first_frame.extend_from_slice(&msg_buf[..len]);
    send.send_frame(&first_frame).await?;

    // XX: read e, ee, s, es from server
    let frame2 = recv.next_frame().await?;
    let mut payload_buf = [0u8; 65535];
    let _payload_len = handshake
        .read_message(&frame2, &mut payload_buf)
        .map_err(|e| Error::Connection(ConnectionError::WebSocket(format!("noise read: {}", e))))?;

    // XX: initiator sends s, se
    let len2 = handshake.write_message(&[], &mut msg_buf).map_err(|e| {
        Error::Connection(ConnectionError::WebSocket(format!("noise write2: {}", e)))
    })?;
    send.send_frame(&msg_buf[..len2]).await?;

    if !handshake.is_handshake_finished() {
        return Err(Error::Connection(ConnectionError::WebSocket(
            "noise handshake not finished".into(),
        )));
    }

    let transport_state = handshake.into_transport_mode().map_err(|e| {
        Error::Connection(ConnectionError::WebSocket(format!(
            "noise transport: {}",
            e
        )))
    })?;
    let state = Arc::new(Mutex::new(transport_state));

    Ok((
        NoiseTransport {
            framed: send,
            state: Arc::clone(&state),
        },
        NoiseRecv {
            framed: recv,
            state: Arc::clone(&state),
        },
    ))
}

/// Transport that encrypts payloads with Noise before sending over the framed WebSocket.
pub struct NoiseTransport {
    framed: FramedSend,
    state: Arc<Mutex<snow::TransportState>>,
}

impl NoiseTransport {
    /// Send encrypted payload (Noise transport encrypt then frame).
    pub async fn send_encrypted(&self, plaintext: &[u8]) -> Result<()> {
        if plaintext.len() > 65535 {
            return Err(Error::Binary("noise payload too large".into()));
        }
        let mut ciphertext = vec![0u8; plaintext.len() + 16];
        let len = {
            let mut st = self.state.lock().await;
            st.write_message(plaintext, &mut ciphertext).map_err(|e| {
                Error::Connection(ConnectionError::WebSocket(format!("noise encrypt: {}", e)))
            })?
        };
        ciphertext.truncate(len);
        self.framed.send_frame(&ciphertext).await
    }
}

#[async_trait]
impl crate::transport::Transport for NoiseTransport {
    async fn send(&self, data: &[u8]) -> Result<()> {
        self.send_encrypted(data).await
    }

    async fn close(&self) -> Result<()> {
        self.framed.close().await
    }
}

/// Receive half: read framed message then decrypt with Noise.
pub struct NoiseRecv {
    framed: FramedRecv,
    state: Arc<Mutex<snow::TransportState>>,
}

/// Connect to the default WebSocket URL and complete the Noise handshake.
pub async fn connect_noise_default() -> Result<(NoiseTransport, NoiseRecv)> {
    connect_noise(crate::client::DEFAULT_WS_URL).await
}

/// Connect to the given WebSocket URL and complete the Noise handshake.
pub async fn connect_noise(url: &str) -> Result<(NoiseTransport, NoiseRecv)> {
    let (send, recv) = super::ws::connect(url).await?;
    run_client_handshake(send, recv).await
}

impl NoiseRecv {
    /// Read next frame and decrypt. Returns the plaintext.
    pub async fn next_decrypted_frame(&self) -> Result<Vec<u8>> {
        let ciphertext = self.framed.next_frame().await?;
        let mut plaintext = vec![0u8; ciphertext.len()];
        let len = {
            let mut st = self.state.lock().await;
            st.read_message(&ciphertext, &mut plaintext).map_err(|e| {
                Error::Connection(ConnectionError::WebSocket(format!("noise decrypt: {}", e)))
            })?
        };
        plaintext.truncate(len);
        Ok(plaintext)
    }
}
