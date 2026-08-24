//! Persistence port for instance — so `application` does not depend on `rusqlite`/`tokio::fs` directly
//! Will be implemented by `infrastructure/persistence` in `feat/foundation/platform` follow-up.

use crate::domain::instance::{InstanceConfig, InstanceId, InstanceMetadata};
use async_trait::async_trait;

#[async_trait]
pub trait InstanceStore: Send + Sync {
    async fn load_metadata(&self, id: InstanceId) -> Option<InstanceMetadata>;
    async fn save_metadata(&self, metadata: &InstanceMetadata) -> anyhow::Result<()>;
    async fn load_config(&self, id: InstanceId) -> Option<InstanceConfig>;
    async fn save_config(&self, id: InstanceId, config: &InstanceConfig) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct InMemoryStore {
        meta: Mutex<HashMap<InstanceId, InstanceMetadata>>,
        cfg: Mutex<HashMap<InstanceId, InstanceConfig>>,
    }

    impl InMemoryStore {
        fn new() -> Self {
            Self {
                meta: Mutex::new(HashMap::new()),
                cfg: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl InstanceStore for InMemoryStore {
        async fn load_metadata(&self, id: InstanceId) -> Option<InstanceMetadata> {
            self.meta.lock().unwrap().get(&id).cloned()
        }
        async fn save_metadata(&self, metadata: &InstanceMetadata) -> anyhow::Result<()> {
            self.meta
                .lock()
                .unwrap()
                .insert(metadata.id, metadata.clone());
            Ok(())
        }
        async fn load_config(&self, id: InstanceId) -> Option<InstanceConfig> {
            self.cfg.lock().unwrap().get(&id).cloned()
        }
        async fn save_config(&self, id: InstanceId, config: &InstanceConfig) -> anyhow::Result<()> {
            self.cfg.lock().unwrap().insert(id, config.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryStore::new();
        let id = uuid::Uuid::new_v4();
        let meta = InstanceMetadata::new(id, Some("1234567890".into()), Some("test".into()));
        store.save_metadata(&meta).await.unwrap();
        assert_eq!(store.load_metadata(id).await.unwrap().id, id);
        let cfg = InstanceConfig::default();
        store.save_config(id, &cfg).await.unwrap();
        assert!(store.load_config(id).await.is_some());
    }
}
