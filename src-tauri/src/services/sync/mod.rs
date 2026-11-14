pub mod index;
pub mod manifest;
pub mod provider;

pub use index::{
    detect_cloud_candidates, execute_sync, get_current_sync_status, get_sync_config,
    resolve_configured_provider, save_sync_config, update_current_sync_status, SyncConfig,
};
pub use manifest::{
    build_local_manifest, calculate_diff, compute_file_sha256, SyncDiff, SyncItem, SyncManifest,
};
pub use provider::{
    AnyCloudProvider, CloudProviderType, CloudStorageProvider, FolderStorageProvider, SyncStatus,
};
