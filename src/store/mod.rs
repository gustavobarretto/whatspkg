//! Device/session store.

mod memory;

pub use memory::MemoryStore;

use crate::types::Jid;
use async_trait::async_trait;
use std::sync::Arc;

/// Device identity and keys for one linked device.
#[derive(Clone, Debug, Default)]
pub struct Device {
    /// Our JID after pairing (None if not paired).
    pub id: Option<Jid>,
    pub lid: Option<Jid>,
    pub business_name: Option<String>,
    pub platform: Option<String>,
    /// Noise public key (32 bytes).
    pub noise_key_pub: Option<[u8; 32]>,
    /// Identity key pair (32 + 32 bytes).
    pub identity_key_pub: Option<[u8; 32]>,
    pub identity_key_priv: Option<[u8; 32]>,
    /// Adv secret for pairing.
    pub adv_secret_key: Option<[u8; 32]>,
    /// Signed device identity (protobuf) after pairing.
    pub account: Option<Vec<u8>>,
    /// Registration ID for Signal.
    pub registration_id: u32,
    /// Signed prekey ID.
    pub signed_prekey_id: u32,
}

impl Device {
    pub fn is_logged_in(&self) -> bool {
        self.id.is_some()
    }
}

/// Store trait: persist and load device state.
#[async_trait]
pub trait DeviceStore: Send + Sync {
    /// Get the first (or only) device. Used to create a client.
    async fn get_first_device(&self) -> crate::Result<Option<Device>>;

    /// Get device by JID (for multi-session).
    async fn get_device(&self, jid: &Jid) -> crate::Result<Option<Device>>;

    /// Save device state (after pairing or key changes).
    async fn save(&self, device: &Device) -> crate::Result<()>;

    /// Delete device (logout).
    async fn delete(&self, jid: &Jid) -> crate::Result<()>;

    /// Get all stored devices.
    async fn get_all_devices(&self) -> crate::Result<Vec<Device>>;
}

/// Alias for boxed store (common usage).
pub type Store = Arc<dyn DeviceStore>;
