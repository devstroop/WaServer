//! Instance Registry — application-level manager extracted from `services/whatsapp/instance_manager.rs:409` (part of #5)
//!
//! Pure registry keyed by UUID with phone→UUID index. Depends only on ports
//! (`InstanceStore`, `LifecyclePorts`) — no `rusqlite`/`chromiumoxide`/`axum`.
//! The legacy `InstanceManager` becomes a facade delegating here.

use crate::application::instance::persistence::InstanceStore;
use crate::domain::instance::{InstanceConfig, InstanceId, InstanceMetadata};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Typed registry errors (per #6: typed errors, not 500)
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("instance '{0}' not found")]
    NotFound(String),
    #[error("phone '{0}' already registered to another instance")]
    PhoneConflict(String),
    #[error("storage error: {0}")]
    Storage(String),
}

/// In-memory registry of instance metadata + config
pub struct InstanceRegistry {
    /// Metadata by UUID
    meta: Arc<RwLock<HashMap<InstanceId, InstanceMetadata>>>,
    /// Config by UUID
    configs: Arc<RwLock<HashMap<InstanceId, InstanceConfig>>>,
    /// phone → UUID index
    phone_to_id: Arc<RwLock<HashMap<String, InstanceId>>>,
}

impl Default for InstanceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceRegistry {
    pub fn new() -> Self {
        Self {
            meta: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            phone_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register (insert or replace) an instance
    pub async fn register(
        &self,
        metadata: InstanceMetadata,
        config: InstanceConfig,
    ) -> Result<(), RegistryError> {
        // Check phone conflict
        if let Some(phone) = &metadata.phone_number {
            let map = self.phone_to_id.read().await;
            if let Some(&existing) = map.get(phone) {
                if existing != metadata.id {
                    return Err(RegistryError::PhoneConflict(phone.clone()));
                }
            }
        }
        self.meta
            .write()
            .await
            .insert(metadata.id, metadata.clone());
        self.configs.write().await.insert(metadata.id, config);
        if let Some(phone) = &metadata.phone_number {
            self.phone_to_id
                .write()
                .await
                .insert(phone.clone(), metadata.id);
        }
        Ok(())
    }

    /// Get metadata by UUID
    pub async fn get(&self, id: InstanceId) -> Option<InstanceMetadata> {
        self.meta.read().await.get(&id).cloned()
    }

    /// Get config by UUID
    pub async fn get_config(&self, id: InstanceId) -> Option<InstanceConfig> {
        self.configs.read().await.get(&id).cloned()
    }

    /// Resolve UUID from id string (UUID or phone)
    pub async fn resolve(&self, id_or_phone: &str) -> Option<InstanceId> {
        if let Ok(uuid) = uuid::Uuid::parse_str(id_or_phone) {
            if self.meta.read().await.contains_key(&uuid) {
                return Some(uuid);
            }
            return None;
        }
        let digits: String = id_or_phone.chars().filter(|c| c.is_ascii_digit()).collect();
        self.phone_to_id.read().await.get(&digits).copied()
    }

    /// Remove instance; returns removed metadata
    pub async fn remove(&self, id: InstanceId) -> Option<InstanceMetadata> {
        let removed = self.meta.write().await.remove(&id)?;
        self.configs.write().await.remove(&id);
        if let Some(phone) = &removed.phone_number {
            self.phone_to_id.write().await.remove(phone);
        }
        Some(removed)
    }

    /// List all metadata sorted by id
    pub async fn list(&self) -> Vec<InstanceMetadata> {
        let mut all: Vec<InstanceMetadata> = self.meta.read().await.values().cloned().collect();
        all.sort_by_key(|m| m.id);
        all
    }

    pub async fn count(&self) -> usize {
        self.meta.read().await.len()
    }

    pub async fn contains(&self, id: InstanceId) -> bool {
        self.meta.read().await.contains_key(&id)
    }

    /// Load all instances from store into registry (discovery, #6: logs but returns typed error)
    pub async fn load_from_store(&self, store: &dyn InstanceStore) -> Result<usize, RegistryError> {
        let metas = crate::application::instance::store_helpers::list_all_metadata(store).await?;
        for meta in metas {
            let cfg = store.load_config(meta.id).await.unwrap_or_default();
            self.register(meta, cfg).await?;
        }
        Ok(self.count().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_resolve() {
        let reg = InstanceRegistry::new();
        let id = uuid::Uuid::new_v4();
        let mut meta = InstanceMetadata::new(id, Some("+1234567890".into()), Some("bot".into()));
        meta.phone_number = Some("1234567890".into());
        reg.register(meta, InstanceConfig::default()).await.unwrap();

        assert!(reg.contains(id).await);
        assert_eq!(reg.count().await, 1);
        assert_eq!(reg.resolve(&id.to_string()).await, Some(id));
        assert_eq!(reg.resolve("1234567890").await, Some(id));
        assert_eq!(reg.resolve("unknown").await, None);
    }

    #[tokio::test]
    async fn test_phone_conflict() {
        let reg = InstanceRegistry::new();
        let a = uuid::Uuid::new_v4();
        let b = uuid::Uuid::new_v4();
        let mut ma = InstanceMetadata::new(a, None, None);
        ma.phone_number = Some("1234567890".into());
        reg.register(ma, InstanceConfig::default()).await.unwrap();

        let mut mb = InstanceMetadata::new(b, None, None);
        mb.phone_number = Some("1234567890".into());
        assert!(matches!(
            reg.register(mb, InstanceConfig::default()).await,
            Err(RegistryError::PhoneConflict(_))
        ));
    }

    #[tokio::test]
    async fn test_remove() {
        let reg = InstanceRegistry::new();
        let id = uuid::Uuid::new_v4();
        let mut meta = InstanceMetadata::new(id, None, None);
        meta.phone_number = Some("1234567890".into());
        reg.register(meta, InstanceConfig::default()).await.unwrap();

        let removed = reg.remove(id).await.unwrap();
        assert_eq!(removed.id, id);
        assert!(!reg.contains(id).await);
        assert_eq!(reg.resolve("1234567890").await, None);
    }

    #[tokio::test]
    async fn test_list_sorted() {
        let reg = InstanceRegistry::new();
        for _ in 0..3 {
            let id = uuid::Uuid::new_v4();
            reg.register(
                InstanceMetadata::new(id, None, None),
                InstanceConfig::default(),
            )
            .await
            .unwrap();
        }
        let all = reg.list().await;
        assert_eq!(all.len(), 3);
        let ids: Vec<_> = all.iter().map(|m| m.id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }
}
