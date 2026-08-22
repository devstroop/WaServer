pub mod config;
pub mod lifecycle;
pub mod metadata;
pub mod persistence;
pub mod state;

pub use config::apply_config_update;
pub use metadata::{load_or_create_metadata, save_metadata};
pub use persistence::InstanceStore;
pub use state::{InstanceState, InstanceStatusView};
pub use lifecycle::{LifecycleError, LifecyclePorts};
