//! Main client.

mod send;

use crate::binary::Node;
use crate::error::{ConnectionError, Error};
use crate::events::Event;
use crate::store::{Device, Store};
use crate::transport::Transport;
use crate::types::{Jid, MessageId};
use sha2::Digest;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub use send::{SendRequestExtra, SendResponse};

/// Parameters for completing pairing after QR or pair-code flow.
#[derive(Clone, Debug)]
pub struct CompletePairingParams<'a> {
    /// Raw device identity from server (payload, or payload || HMAC-SHA256 tag if verifying).
    pub device_identity_bytes: &'a [u8],
    /// Request ID from the pairing flow.
    pub req_id: &'a str,
    pub business_name: &'a str,
    pub platform: &'a str,
    pub jid: Jid,
    pub lid: Jid,
    /// If set, device_identity_bytes is verified as payload || HMAC tag before use.
    pub hmac_key: Option<&'a [u8]>,
}

/// Type alias for event handlers so the client field is not overly complex and is Send + Sync.
type EventHandler = Box<dyn Fn(Event) + Send + Sync>;

/// Default WebSocket URL for WhatsApp Web.
pub const DEFAULT_WS_URL: &str = "wss://web.whatsapp.com/ws";

/// Client for the WhatsApp web multidevice API.
pub struct Client {
    store: Store,
    device: Arc<RwLock<Option<Device>>>,
    _event_tx: mpsc::UnboundedSender<Event>,
    handlers: Arc<RwLock<Vec<EventHandler>>>,
    connected: AtomicBool,
    logged_in: AtomicBool,
    /// When set, send_node() uses this transport (e.g. Noise over WebSocket). Set by connect() when feature "full" is enabled.
    transport: Arc<RwLock<Option<Arc<dyn Transport>>>>,
}

impl Client {
    /// Create a new client with the given device store.
    pub fn new(store: Store) -> Self {
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        Self {
            store,
            device: Arc::new(RwLock::new(None)),
            _event_tx: event_tx,
            handlers: Arc::new(RwLock::new(Vec::new())),
            connected: AtomicBool::new(false),
            logged_in: AtomicBool::new(false),
            transport: Arc::new(RwLock::new(None)),
        }
    }

    /// Add an event handler (called for every event). Mirrors AddEventHandler.
    pub async fn add_event_handler<F>(&self, f: F)
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        self.handlers.write().await.push(Box::new(f));
    }

    /// Load device from store and mark logged in if present.
    pub async fn load_device(&self) -> crate::Result<()> {
        let device = self.store.get_first_device().await?;
        if let Some(d) = &device {
            if d.is_logged_in() {
                self.logged_in.store(true, Ordering::SeqCst);
            }
        }
        *self.device.write().await = device;
        Ok(())
    }

    /// Connect to WhatsApp servers. If no session, will emit QR events for pairing.
    /// With feature "full", performs a real WebSocket + Noise handshake and stores the transport.
    pub async fn connect(&self) -> crate::Result<()> {
        self.load_device().await?;
        let device = self.device.read().await.clone();
        if device.as_ref().is_none_or(|d| !d.is_logged_in()) {
            self.dispatch_event(Event::Qr {
                codes: vec!["STUB_QR_CODE".to_string()],
            })
            .await;
            return Ok(());
        }
        #[cfg(feature = "full")]
        {
            if let Ok((noise_tx, noise_rx)) = crate::socket::connect_noise_default().await {
                let transport: Arc<dyn Transport> = Arc::new(noise_tx);
                *self.transport.write().await = Some(Arc::clone(&transport));
                tokio::spawn(Self::recv_loop(noise_rx));
            }
        }
        self.connected.store(true, Ordering::SeqCst);
        self.logged_in.store(true, Ordering::SeqCst);
        self.dispatch_event(Event::Connected).await;
        Ok(())
    }

    #[cfg(feature = "full")]
    async fn recv_loop(noise_rx: crate::socket::NoiseRecv) {
        while let Ok(frame) = noise_rx.next_decrypted_frame().await {
            if let Ok(node) = Node::decode(&frame) {
                tracing::debug!(tag = %node.tag, "incoming node");
                // TODO: dispatch node to handlers / handle server nodes
                let _ = node;
            }
        }
    }

    /// Disconnect and optionally clear session. Clears the transport when present.
    pub async fn disconnect(&self, logout: bool) -> crate::Result<()> {
        if logout {
            let device = self.device.read().await.clone();
            if let Some(ref d) = device {
                if let Some(ref jid) = d.id {
                    self.store.delete(jid).await?;
                }
            }
            *self.device.write().await = None;
            self.logged_in.store(false, Ordering::SeqCst);
        }
        *self.transport.write().await = None;
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Log out (unpair) and disconnect.
    pub async fn logout(&self) -> crate::Result<()> {
        self.disconnect(true).await
    }

    /// Whether the client is connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Whether the client has a logged-in session.
    pub fn is_logged_in(&self) -> bool {
        self.logged_in.load(Ordering::SeqCst)
    }

    /// Get our JID if logged in.
    pub async fn get_own_id(&self) -> Option<Jid> {
        self.device.read().await.as_ref().and_then(|d| d.id.clone())
    }

    /// Generate a message ID (3EB0 + hex of hash).
    pub fn generate_message_id(&self) -> MessageId {
        use std::time::{SystemTime, UNIX_EPOCH};
        let mut data = Vec::with_capacity(8 + 20 + 16);
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        data.extend_from_slice(&t.to_be_bytes());
        data.extend_from_slice(b"@c.us");
        data.extend_from_slice(&rand::random::<[u8; 16]>());
        let hash = sha2::Sha256::digest(&data);
        format!("3EB0{}", hex::encode(&hash[..9]))
    }

    async fn dispatch_event(&self, evt: Event) {
        let handlers = self.handlers.read().await;
        for f in handlers.iter() {
            f(evt.clone());
        }
    }

    /// Send a raw node over the transport when connected (with feature "full"). Used when send_message is implemented over the wire.
    #[allow(dead_code)]
    pub(crate) async fn send_node(&self, node: &Node) -> crate::Result<()> {
        let transport = self.transport.read().await;
        let t = transport
            .as_ref()
            .ok_or(Error::Connection(ConnectionError::Disconnected))?;
        let data = node.encode()?;
        t.send(&data).await
    }

    /// Send a text message. Stub: requires live connection.
    pub async fn send_message(
        &self,
        _to: &Jid,
        _body: &str,
        extra: Option<SendRequestExtra>,
    ) -> crate::Result<SendResponse> {
        if !self.is_connected() {
            return Err(Error::NotConnected);
        }
        let id = extra
            .as_ref()
            .and_then(|e| e.id.clone())
            .unwrap_or_else(|| self.generate_message_id());
        Ok(SendResponse {
            timestamp: std::time::SystemTime::now(),
            id: id.clone(),
            server_id: None,
            sender: self.get_own_id().await,
        })
    }

    /// Parse pair-success and save device. Called when QR is scanned.
    /// If `params.hmac_key` is set, verifies `params.device_identity_bytes` (payload || HMAC-SHA256 tag) before proceeding.
    /// Generates pairing keys (Noise, identity, adv secret), signs the verified payload for storage, and persists the device.
    pub async fn complete_pairing(&self, params: CompletePairingParams<'_>) -> crate::Result<()> {
        let verified_payload = if let Some(key) = params.hmac_key {
            crate::pairing::verify_device_identity(params.device_identity_bytes, key)?.payload
        } else {
            params.device_identity_bytes.to_vec()
        };

        let keys = crate::pairing::generate_pairing_keys();
        let account =
            crate::pairing::sign_device_identity(&verified_payload, &keys.identity_private)?;

        let mut device = self.store.get_first_device().await?.unwrap_or_default();
        device.id = Some(params.jid.clone());
        device.lid = Some(params.lid.clone());
        device.business_name = Some(params.business_name.to_string());
        device.platform = Some(params.platform.to_string());
        device.noise_key_pub = Some(keys.noise_public);
        device.identity_key_pub = Some(keys.identity_public);
        device.identity_key_priv = Some(keys.identity_private);
        device.adv_secret_key = Some(keys.adv_secret);
        device.account = Some(account);
        device.registration_id = 0;
        device.signed_prekey_id = 0;
        self.store.save(&device).await?;
        *self.device.write().await = Some(device);
        self.logged_in.store(true, Ordering::SeqCst);
        self.dispatch_event(Event::PairSuccess {
            id: params.jid.clone(),
            lid: params.lid.clone(),
            business_name: params.business_name.to_string(),
            platform: params.platform.to_string(),
        })
        .await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::store::{DeviceStore, MemoryStore};

    #[test]
    fn generate_message_id_format() {
        let store = Arc::new(MemoryStore::new());
        let client = Client::new(store);
        let id = client.generate_message_id();
        assert!(id.starts_with("3EB0"));
        assert!(id.len() > 4);
        assert!(id[4..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn connect_emits_qr_when_no_session() {
        let store = Arc::new(MemoryStore::new());
        let client = Client::new(store);
        let qr_received = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let qr_received_clone = Arc::clone(&qr_received);
        client
            .add_event_handler(move |evt| {
                if let Event::Qr { .. } = evt {
                    qr_received_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            })
            .await;
        client.connect().await.unwrap();
        assert!(qr_received.load(std::sync::atomic::Ordering::SeqCst));
        assert!(!client.is_logged_in());
    }

    #[tokio::test]
    async fn connect_emits_connected_when_session_exists() {
        let store = Arc::new(MemoryStore::new());
        let mut dev = crate::store::Device::default();
        dev.id = Some(Jid::new("123", "s.whatsapp.net"));
        store.save(&dev).await.unwrap();

        let client = Client::new(store);
        let connected_received = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let connected_received_clone = Arc::clone(&connected_received);
        client
            .add_event_handler(move |evt| {
                if let Event::Connected = evt {
                    connected_received_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
            })
            .await;
        client.connect().await.unwrap();
        assert!(connected_received.load(std::sync::atomic::Ordering::SeqCst));
        assert!(client.is_logged_in());
        assert!(client.is_connected());
    }

    #[tokio::test]
    async fn disconnect_clears_state_on_logout() {
        let store = Arc::new(MemoryStore::new());
        let mut dev = crate::store::Device::default();
        dev.id = Some(Jid::new("123", "s.whatsapp.net"));
        store.save(&dev).await.unwrap();

        let client = Client::new(store);
        client.connect().await.unwrap();
        assert!(client.is_logged_in());
        client.disconnect(true).await.unwrap();
        assert!(!client.is_logged_in());
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn send_message_fails_when_not_connected() {
        let store = Arc::new(MemoryStore::new());
        let client = Client::new(store);
        let to = Jid::new("123", "s.whatsapp.net");
        let res = client.send_message(&to, "hello", None).await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), crate::Error::NotConnected));
    }

    #[tokio::test]
    async fn complete_pairing_persists_keys_and_account() {
        let store = Arc::new(MemoryStore::new());
        let client = Client::new(store.clone());
        let payload = b"device-identity-payload";
        client
            .complete_pairing(CompletePairingParams {
                device_identity_bytes: payload,
                req_id: "req1",
                business_name: "Biz",
                platform: "Rust",
                jid: Jid::new("123", "s.whatsapp.net"),
                lid: Jid::new("0", "lid.whatsapp.net"),
                hmac_key: None,
            })
            .await
            .unwrap();
        assert!(client.is_logged_in());
        let device = store.get_first_device().await.unwrap().unwrap();
        assert!(device.identity_key_pub.is_some());
        assert!(device.identity_key_priv.is_some());
        assert!(device.noise_key_pub.is_some());
        assert!(device.adv_secret_key.is_some());
        assert!(device.account.is_some());
        let account = device.account.as_ref().unwrap();
        assert!(account.len() >= 32 + 64);
        let verified = crate::pairing::verify_signed_identity(account).unwrap();
        assert_eq!(verified, payload);
    }
}
