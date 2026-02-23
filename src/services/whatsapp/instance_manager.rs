//! Instance Manager Service
//!
//! Manages multiple WhatsApp instances with create/get/list/delete operations.
//! Instance IDs are UUIDs, phone numbers are unique per instance.

use super::instance::WhatsAppInstance;
use crate::{
    config::AppConfig,
    models::instance::{
        phone_to_dir_name, validate_phone_number, InstanceSetupConfig, InstanceId, InstanceListResponse,
        InstanceMetadata, CreateInstanceRequest, CreateInstanceResponse,
    },
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Instance metadata filename
const METADATA_FILE: &str = "instance.json";

/// Manages multiple WhatsApp instances
pub struct InstanceManager {
    /// Active instance instances (keyed by UUID)
    instances: Arc<RwLock<HashMap<InstanceId, Arc<WhatsAppInstance>>>>,
    /// Phone number to UUID mapping for lookups
    phone_to_id: Arc<RwLock<HashMap<String, InstanceId>>>,
    /// Base directory for all instance data
    base_dir: PathBuf,
    /// App configuration
    config: Arc<AppConfig>,
}

impl InstanceManager {
    /// Create a new InstanceManager
    pub fn new(config: Arc<AppConfig>) -> Self {
        // Use configured base directory or default to ~/.was/instances
        let base_dir = config
            .instances
            .as_ref()
            .and_then(|ac| ac.base_directory.clone())
            .unwrap_or_else(|| {
                // Get home directory from environment
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".was").join("instances")
            });

        info!("InstanceManager initialized with base_dir: {:?}", base_dir);

        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            phone_to_id: Arc::new(RwLock::new(HashMap::new())),
            base_dir,
            config,
        }
    }

    /// Get base directory for instances
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    /// Create a new instance
    pub async fn create_instance(
        &self,
        request: CreateInstanceRequest,
    ) -> Result<CreateInstanceResponse> {
        // Validate and normalize phone number
        let phone_number = validate_phone_number(&request.phone_number)
            .map_err(|e| anyhow!("Invalid phone number: {}", e))?;

        // Check if phone number is already used
        {
            let phone_map = self.phone_to_id.read().await;
            if phone_map.contains_key(&phone_number) {
                return Err(anyhow!(
                    "Instance for phone '{}' already exists",
                    phone_number
                ));
            }
        }

        // Use phone digits as directory name
        let dir_name = phone_to_dir_name(&phone_number);
        let instance_dir = self.base_dir.join(&dir_name);

        // Check if directory already exists (instance was previously created)
        if instance_dir.exists() {
            // Check for metadata file
            let metadata_path = instance_dir.join(METADATA_FILE);
            if metadata_path.exists() {
                return Err(anyhow!(
                    "Instance directory for phone '{}' already exists. Use discover_instances() to load it.",
                    phone_number
                ));
            }
        }

        // Generate new UUID for this instance
        let instance_id = Uuid::new_v4();

        // Create instance config
        let setup_config = InstanceSetupConfig {
            id: instance_id,
            phone_number: phone_number.clone(),
            display_name: request.display_name.clone(),
            data_dir: instance_dir.clone(),
            browser: request.browser.clone().unwrap_or_default(),
            auto_start: request.auto_start.unwrap_or(false),
        };

        // Create the instance
        let instance = Arc::new(WhatsAppInstance::new(setup_config, self.config.clone()).await?);

        // Store in memory
        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, instance);
        }
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.insert(phone_number.clone(), instance_id);
        }

        info!(
            "Created instance '{}' for phone '{}'",
            instance_id, phone_number
        );

        Ok(CreateInstanceResponse {
            id: instance_id,
            phone_number,
            status: "created".to_string(),
            data_directory: instance_dir.to_string_lossy().to_string(),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    /// Get an existing instance by UUID or phone number
    pub async fn get_instance(&self, id: &str) -> Option<Arc<WhatsAppInstance>> {
        // Try to parse as UUID first
        if let Ok(uuid) = Uuid::parse_str(id) {
            return self.instances.read().await.get(&uuid).cloned();
        }

        // Try to look up by phone number
        let phone = validate_phone_number(id).unwrap_or_else(|_| id.to_string());

        let phone_map = self.phone_to_id.read().await;
        if let Some(uuid) = phone_map.get(&phone) {
            return self.instances.read().await.get(uuid).cloned();
        }

        None
    }

    /// Get an instance by UUID
    pub async fn get_instance_by_id(&self, id: InstanceId) -> Option<Arc<WhatsAppInstance>> {
        self.instances.read().await.get(&id).cloned()
    }

    /// Get an instance by phone number
    pub async fn get_instance_by_phone(&self, phone: &str) -> Option<Arc<WhatsAppInstance>> {
        let phone = validate_phone_number(phone).unwrap_or_else(|_| phone.to_string());

        let phone_map = self.phone_to_id.read().await;
        if let Some(uuid) = phone_map.get(&phone) {
            return self.instances.read().await.get(uuid).cloned();
        }
        None
    }

    /// Get an instance, returning an error if not found
    pub async fn get_instance_or_error(&self, id: &str) -> Result<Arc<WhatsAppInstance>> {
        self.get_instance(id)
            .await
            .ok_or_else(|| anyhow!("Instance '{}' not found", id))
    }

    /// List all instances
    pub async fn list_instances(&self) -> InstanceListResponse {
        let instances = self.instances.read().await;
        let mut instance_infos = Vec::with_capacity(instances.len());

        for instance in instances.values() {
            instance_infos.push(instance.info().await);
        }

        // Sort by ID for consistent ordering
        instance_infos.sort_by(|a, b| a.id.cmp(&b.id));

        InstanceListResponse {
            total: instance_infos.len(),
            instances: instance_infos,
        }
    }

    /// Delete an instance
    pub async fn delete_instance(&self, id: &str, delete_data: bool) -> Result<InstanceId> {
        // Find the instance (by UUID or phone)
        let instance = self
            .get_instance(id)
            .await
            .ok_or_else(|| anyhow!("Instance '{}' not found", id))?;

        let instance_id = instance.id;
        let phone_number = instance.phone_number.clone();

        // Remove from both maps
        {
            let mut instances = self.instances.write().await;
            instances.remove(&instance_id);
        }
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.remove(&phone_number);
        }

        // Stop the instance first
        if let Err(e) = instance.stop().await {
            warn!(
                "Error stopping instance '{}' during deletion: {}",
                instance_id, e
            );
        }

        if delete_data {
            let dir_name = phone_to_dir_name(&phone_number);
            let instance_dir = self.base_dir.join(&dir_name);
            if instance_dir.exists() {
                tokio::fs::remove_dir_all(&instance_dir).await?;
                info!(
                    "Deleted instance '{}' (phone: {}) and all data",
                    instance_id, phone_number
                );
            }
        } else {
            info!(
                "Deleted instance '{}' (phone: {}) (data preserved)",
                instance_id, phone_number
            );
        }

        Ok(instance_id)
    }

    /// Start an instance's browser
    pub async fn start_instance(&self, id: &str) -> Result<()> {
        let instance = self.get_instance_or_error(id).await?;
        instance.start().await
    }

    /// Stop an instance's browser
    pub async fn stop_instance(&self, id: &str) -> Result<()> {
        let instance = self.get_instance_or_error(id).await?;
        instance.stop().await
    }

    /// Discover existing instances from filesystem
    pub async fn discover_instances(&self) -> Result<Vec<InstanceId>> {
        let mut discovered = Vec::new();

        // Ensure base directory exists
        if !self.base_dir.exists() {
            tokio::fs::create_dir_all(&self.base_dir).await?;
            return Ok(discovered);
        }

        let mut entries = tokio::fs::read_dir(&self.base_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dir_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Check for metadata file
            let metadata_path = path.join(METADATA_FILE);
            if !metadata_path.exists() {
                debug!("Skipping directory '{}' - no metadata file", dir_name);
                continue;
            }

            // Load metadata to get instance ID and phone number
            let content = match tokio::fs::read_to_string(&metadata_path).await {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to read metadata in '{}': {}", dir_name, e);
                    continue;
                }
            };

            let metadata: InstanceMetadata = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to parse metadata in '{}': {}", dir_name, e);
                    continue;
                }
            };

            let instance_id = metadata.id;
            let phone_number = metadata.phone_number.clone();

            // Skip if already loaded
            if self.instances.read().await.contains_key(&instance_id) {
                continue;
            }

            // Load the instance
            match self
                .load_instance_from_dir(instance_id, &phone_number, &path, &metadata)
                .await
            {
                Ok(()) => {
                    discovered.push(instance_id);
                }
                Err(e) => {
                    error!("Failed to load instance from '{}': {}", dir_name, e);
                }
            }
        }

        if !discovered.is_empty() {
            info!("Discovered {} existing instances", discovered.len());
        }

        Ok(discovered)
    }

    /// Load an instance from its data directory
    async fn load_instance_from_dir(
        &self,
        instance_id: InstanceId,
        phone_number: &str,
        data_dir: &PathBuf,
        metadata: &InstanceMetadata,
    ) -> Result<()> {
        // Create config from metadata
        let setup_config = InstanceSetupConfig {
            id: instance_id,
            phone_number: phone_number.to_string(),
            display_name: metadata.display_name.clone(),
            data_dir: data_dir.clone(),
            browser: Default::default(),
            auto_start: false,
        };

        // Create instance instance
        let instance = Arc::new(WhatsAppInstance::new(setup_config, self.config.clone()).await?);

        // Store in both maps
        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, instance);
        }
        {
            let mut phone_map = self.phone_to_id.write().await;
            phone_map.insert(phone_number.to_string(), instance_id);
        }

        info!(
            "Loaded instance '{}' (phone: {}) from {:?}",
            instance_id, phone_number, data_dir
        );
        Ok(())
    }

    /// Get instance count
    pub async fn count(&self) -> usize {
        self.instances.read().await.len()
    }

    /// Check if an instance exists (by UUID or phone)
    pub async fn exists(&self, id: &str) -> bool {
        self.get_instance(id).await.is_some()
    }

    /// Auto-start instances that have auto_start enabled
    pub async fn auto_start_instances(&self) -> Vec<(InstanceId, Result<()>)> {
        let instances = self.instances.read().await;
        let results = Vec::new();

        for (id, instance) in instances.iter() {
            let info = instance.info().await;
            // TODO: Check config for auto_start flag
            // For now, skip auto-start
            debug!("Instance '{}' auto_start check (disabled for now)", id);
            let _ = info;
        }

        results
    }

    /// Shutdown all instances
    pub async fn shutdown_all(&self) -> Vec<(InstanceId, Result<()>)> {
        let instances = self.instances.read().await;
        let mut results = Vec::new();

        for (id, instance) in instances.iter() {
            let result = instance.stop().await;
            results.push((*id, result));
        }

        info!("Shutdown {} instances", results.len());
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_phone_number() {
        // Valid phone numbers
        assert!(validate_phone_number("+1234567890").is_ok());
        assert!(validate_phone_number("1234567890").is_ok());
        assert!(validate_phone_number("+919876543210").is_ok());

        // Invalid phone numbers
        assert!(validate_phone_number("").is_err());
        assert!(validate_phone_number("123456").is_err()); // Too short
    }
}
