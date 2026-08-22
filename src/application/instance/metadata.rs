//! Instance Metadata application logic — load/save `InstanceMetadata`
//! Extracted from `services/whatsapp/instance.rs:142` `load_or_create_metadata`

use crate::domain::instance::{InstanceMetadata, InstanceSetupConfig};
use std::path::Path;

pub async fn load_or_create_metadata(
    data_dir: &Path,
    config: &InstanceSetupConfig,
) -> anyhow::Result<InstanceMetadata> {
    let path = data_dir.join("account.json");
    if path.exists() {
        let content = tokio::fs::read_to_string(&path).await?;
        let metadata: InstanceMetadata = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse metadata: {}", e))?;
        Ok(metadata)
    } else {
        let metadata = InstanceMetadata::new(
            config.id,
            config.phone_number.clone(),
            config.instance_name.clone(),
        );
        let content = serde_json::to_string_pretty(&metadata)?;
        tokio::fs::write(&path, content).await?;
        Ok(metadata)
    }
}

pub async fn save_metadata(data_dir: &Path, metadata: &InstanceMetadata) -> anyhow::Result<()> {
    let path = data_dir.join("account.json");
    let content = serde_json::to_string_pretty(metadata)?;
    tokio::fs::write(path, content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_or_create_metadata() {
        let dir = TempDir::new().unwrap();
        let id = uuid::Uuid::new_v4();
        let cfg = InstanceSetupConfig {
            id,
            phone_number: Some("1234567890".into()),
            instance_name: Some("test".into()),
            data_dir: dir.path().to_path_buf(),
            browser: Default::default(),
        };
        let meta = load_or_create_metadata(dir.path(), &cfg).await.unwrap();
        assert_eq!(meta.id, id);
        // second call loads from disk
        let meta2 = load_or_create_metadata(dir.path(), &cfg).await.unwrap();
        assert_eq!(meta2.id, id);
        assert_eq!(meta2.phone_number, Some("1234567890".into()));
    }

    #[tokio::test]
    async fn test_save_metadata() {
        let dir = TempDir::new().unwrap();
        let id = uuid::Uuid::new_v4();
        let mut meta = InstanceMetadata::new(id, None, None);
        meta.instance_name = Some("save-test".into());
        save_metadata(dir.path(), &meta).await.unwrap();
        let loaded = load_or_create_metadata(
            dir.path(),
            &InstanceSetupConfig {
                id,
                phone_number: None,
                instance_name: None,
                data_dir: dir.path().to_path_buf(),
                browser: Default::default(),
            },
        )
        .await
        .unwrap();
        assert_eq!(loaded.instance_name, Some("save-test".into()));
    }
}
