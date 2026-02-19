use super::{Device, DeviceStore};
use crate::{error::StoreError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

/// In-memory device store (for testing or single-run; not persistent).
pub struct MemoryStore {
    devices: RwLock<HashMap<String, Device>>,
    first_jid: RwLock<Option<String>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(HashMap::new()),
            first_jid: RwLock::new(None),
        }
    }

    fn first_jid_key() -> String {
        "__first".to_string()
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DeviceStore for MemoryStore {
    async fn get_first_device(&self) -> Result<Option<Device>> {
        let first = self
            .first_jid
            .read()
            .map_err(|e| StoreError::Load(e.to_string()))?
            .clone();
        let default_key = Self::first_jid_key();
        let key = first.as_deref().unwrap_or_else(|| default_key.as_str());
        let devices = self
            .devices
            .read()
            .map_err(|e| StoreError::Load(e.to_string()))?;
        Ok(devices.get(key).cloned())
    }

    async fn get_device(&self, jid: &crate::types::Jid) -> Result<Option<Device>> {
        let devices = self
            .devices
            .read()
            .map_err(|e| StoreError::Load(e.to_string()))?;
        Ok(devices.get(&jid.to_string()).cloned())
    }

    async fn save(&self, device: &Device) -> Result<()> {
        let key = device
            .id
            .as_ref()
            .map(|j| j.to_string())
            .unwrap_or_else(Self::first_jid_key);
        if device.id.is_some() {
            *self
                .first_jid
                .write()
                .map_err(|e| StoreError::Save(e.to_string()))? = Some(key.clone());
        }
        self.devices
            .write()
            .map_err(|e| StoreError::Save(e.to_string()))?
            .insert(key, device.clone());
        Ok(())
    }

    async fn delete(&self, jid: &crate::types::Jid) -> Result<()> {
        let key = jid.to_string();
        self.devices
            .write()
            .map_err(|e| StoreError::Save(e.to_string()))?
            .remove(&key);
        let mut first = self
            .first_jid
            .write()
            .map_err(|e| StoreError::Save(e.to_string()))?;
        if *first == Some(key) {
            *first = None;
        }
        Ok(())
    }

    async fn get_all_devices(&self) -> Result<Vec<Device>> {
        let devices = self
            .devices
            .read()
            .map_err(|e| StoreError::Load(e.to_string()))?;
        Ok(devices
            .values()
            .filter(|d| d.id.is_some())
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Jid;

    #[tokio::test]
    async fn memory_store_save_and_get_first() {
        let store = MemoryStore::new();
        let mut dev = Device::default();
        dev.id = Some(Jid::new("123", "s.whatsapp.net"));

        store.save(&dev).await.unwrap();
        let loaded = store.get_first_device().await.unwrap().unwrap();
        assert_eq!(
            loaded.id.as_ref().unwrap().to_string(),
            "123@s.whatsapp.net"
        );
    }

    #[tokio::test]
    async fn memory_store_get_device_by_jid() {
        let store = MemoryStore::new();
        let jid = Jid::new("456", "s.whatsapp.net");
        let mut dev = Device::default();
        dev.id = Some(jid.clone());

        store.save(&dev).await.unwrap();
        let loaded = store.get_device(&jid).await.unwrap().unwrap();
        assert!(loaded.id.is_some());
    }

    #[tokio::test]
    async fn memory_store_delete() {
        let store = MemoryStore::new();
        let jid = Jid::new("789", "s.whatsapp.net");
        let mut dev = Device::default();
        dev.id = Some(jid.clone());

        store.save(&dev).await.unwrap();
        assert!(store.get_device(&jid).await.unwrap().is_some());
        store.delete(&jid).await.unwrap();
        assert!(store.get_device(&jid).await.unwrap().is_none());
        assert!(store.get_first_device().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn memory_store_get_all_devices() {
        let store = MemoryStore::new();
        let mut d1 = Device::default();
        d1.id = Some(Jid::new("1", "s.whatsapp.net"));
        let mut d2 = Device::default();
        d2.id = Some(Jid::new("2", "s.whatsapp.net"));
        store.save(&d1).await.unwrap();
        store.save(&d2).await.unwrap();
        let all = store.get_all_devices().await.unwrap();
        assert_eq!(all.len(), 2);
    }
}
