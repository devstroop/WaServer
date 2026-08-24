pub mod config;
pub mod config_validation;
pub mod lifecycle;
pub mod manager;
pub mod metadata;
pub mod persistence;
pub mod state;
pub mod store_helpers;

pub use config::apply_config_update;
pub use config_validation::{validate_config, validated_apply_config_update, ConfigError};
pub use lifecycle::{LifecycleError, LifecyclePorts};
pub use manager::{InstanceRegistry, RegistryError};
pub use metadata::{load_or_create_metadata, save_metadata};
pub use persistence::InstanceStore;
pub use state::{InstanceState, InstanceStatusView};
