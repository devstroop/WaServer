//! Instance repository — implements `InstanceStore` port for `Database` (part of #5)
//! Extracted from `services/whatsapp/instance_manager.rs:294` discovery + `persistence/service.rs:65`.
//! Keeps `application::instance::manager` rusqlite-free.

use crate::application::instance::persistence::InstanceStore;
use crate::domain::instance::{InstanceConfig, InstanceId, InstanceMetadata, InstanceSetupConfig};
use crate::infrastructure::persistence::service::Database;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// SQLite-backed `InstanceStore`
pub struct SqliteInstanceStore(pub Database);

#[async_trait]
impl InstanceStore for SqliteInstanceStore {
    async fn load_metadata(&self, id: InstanceId) -> Option<InstanceMetadata> {
        let record = self.0.get_instance(&id.to_string()).ok().flatten()?;
        let created_at = record
            .created_at
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        Some(InstanceMetadata {
            id,
            phone_number: Some(record.phone_number),
            instance_name: if record.instance_name.is_empty() {
                None
            } else {
                Some(record.instance_name)
            },
            created_at,
            first_linked_at: None,
        })
    }

    async fn save_metadata(&self, metadata: &InstanceMetadata) -> anyhow::Result<()> {
        // Metadata persists to account.json via application/instance/metadata.rs;
        // DB row is created in `create_instance_row`. Here we only update the name.
        if let Some(name) = &metadata.instance_name {
            self.0
                .update_instance_name(&metadata.id.to_string(), name)?;
        }
        Ok(())
    }

    async fn load_config(&self, id: InstanceId) -> Option<InstanceConfig> {
        let record = self.0.get_instance(&id.to_string()).ok().flatten()?;
        Some(InstanceConfig {
            instance_id: Some(id),
            instance_name: Some(record.instance_name),
            idle_timeout: record.idle_timeout,
            browser: Default::default(),
            rate_limits: Default::default(),
        })
    }

    async fn save_config(&self, id: InstanceId, config: &InstanceConfig) -> anyhow::Result<()> {
        if let Some(name) = &config.instance_name {
            self.0.update_instance_name(&id.to_string(), name)?;
        }
        self.0
            .update_idle_timeout(&id.to_string(), config.idle_timeout)?;
        Ok(())
    }
}

impl SqliteInstanceStore {
    /// Create DB row for a new instance — mirrors `instance_manager.rs:93` `db.create_instance`
    pub async fn create_instance_row(
        &self,
        id: InstanceId,
        phone_number: &str,
        instance_name: &str,
        data_dir: PathBuf,
        idle_timeout: u64,
    ) -> anyhow::Result<()> {
        self.0.create_instance(
            &id.to_string(),
            phone_number,
            instance_name,
            &data_dir.to_string_lossy(),
            idle_timeout,
        )?;
        Ok(())
    }

    /// Delete DB row — mirrors `instance_manager.rs:240`
    pub async fn delete_instance_row(&self, id: InstanceId) -> anyhow::Result<()> {
        self.0.delete_instance(&id.to_string())?;
        Ok(())
    }

    /// List all records → metadata (discovery input)
    pub async fn list_metadata(&self) -> anyhow::Result<Vec<InstanceMetadata>> {
        let records = self.0.list_instances()?;
        let mut metas = Vec::with_capacity(records.len());
        for r in records {
            let id = uuid::Uuid::parse_str(&r.id)
                .map_err(|e| anyhow::anyhow!("Invalid UUID '{}' in db: {}", r.id, e))?;
            let created_at = r
                .created_at
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);
            metas.push(InstanceMetadata {
                id,
                phone_number: Some(r.phone_number),
                instance_name: Some(r.instance_name),
                created_at,
                first_linked_at: None,
            });
        }
        Ok(metas)
    }

    /// Build setup config from a DB record — mirrors `instance_manager.rs:328`
    pub fn setup_config_for(
        &self,
        id: InstanceId,
        phone: Option<String>,
        name: Option<String>,
        data_dir: PathBuf,
    ) -> InstanceSetupConfig {
        InstanceSetupConfig {
            id,
            phone_number: phone,
            instance_name: name,
            data_dir,
            browser: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_roundtrip() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::open(dir.path()).unwrap();
        let store = SqliteInstanceStore(db);
        let id = uuid::Uuid::new_v4();
        store
            .create_instance_row(id, "1234567890", "bot", dir.path().to_path_buf(), 300)
            .await
            .unwrap();

        let meta = store.load_metadata(id).await.unwrap();
        assert_eq!(meta.id, id);
        assert_eq!(meta.phone_number.as_deref(), Some("1234567890"));

        let cfg = store.load_config(id).await.unwrap();
        assert_eq!(cfg.idle_timeout, 300);

        // update config
        let mut cfg2 = cfg.clone();
        cfg2.idle_timeout = 999;
        store.save_config(id, &cfg2).await.unwrap();
        assert_eq!(store.load_config(id).await.unwrap().idle_timeout, 999);

        // delete
        store.delete_instance_row(id).await.unwrap();
        assert!(store.load_metadata(id).await.is_none());
    }

    #[tokio::test]
    async fn test_list_metadata() {
        let dir = tempfile::TempDir::new().unwrap();
        let db = Database::open(dir.path()).unwrap();
        let store = SqliteInstanceStore(db);
        for i in 0..3 {
            let id = uuid::Uuid::new_v4();
            store
                .create_instance_row(
                    id,
                    &format!("123456789{}", i),
                    &format!("bot-{}", i),
                    dir.path().to_path_buf(),
                    300,
                )
                .await
                .unwrap();
        }
        let metas = store.list_metadata().await.unwrap();
        assert_eq!(metas.len(), 3);
    }
}
