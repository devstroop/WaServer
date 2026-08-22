//! Store helpers — batch operations over `InstanceStore` (part of #5)
//! Keeps `manager.rs` free of store-iteration details.

use crate::application::instance::manager::RegistryError;
use crate::application::instance::persistence::InstanceStore;
use crate::domain::instance::InstanceMetadata;

/// List all metadata from a store. `SqliteInstanceStore` implements `list_metadata`;
/// for stores without it, this returns an empty vec scaffold.
pub async fn list_all_metadata(
    _store: &dyn InstanceStore,
) -> Result<Vec<InstanceMetadata>, RegistryError> {
    // Scaffold: full implementation lands with `infrastructure/persistence/instance_repo.rs`
    // which will expose `list_metadata`. Registry discovery via `Database::list_instances`
    // remains in the legacy facade until then.
    Ok(Vec::new())
}
