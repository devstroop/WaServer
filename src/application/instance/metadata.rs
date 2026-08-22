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
