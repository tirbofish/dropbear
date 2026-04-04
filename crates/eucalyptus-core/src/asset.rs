/// Watches for any changes in the `resources/` folder. 
pub fn start_asset_entry_watcher() {
    std::thread::Builder::new()
        .name("asset-entry-watcher".into())
        .spawn(|| {
            use std::fs;
            use std::path::{Path, PathBuf};
            use std::time::{Duration, SystemTime};
            use sha2::{Digest, Sha256};
            use crate::states::PROJECT;
            use crate::metadata::{detect_asset_type, generate_eucmeta, AssetEntry};

            fn hash_file(path: &Path) -> Option<[u8; 32]> {
                let bytes = fs::read(path).ok()?;
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&Sha256::digest(&bytes));
                Some(hash)
            }

            fn scan_and_update(dir: &Path, project_root: &Path) {
                let entries = match fs::read_dir(dir) {
                    Ok(e) => e,
                    Err(e) => {
                        log::warn!("asset watcher: cannot read {}: {e}", dir.display());
                        return;
                    }
                };

                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        scan_and_update(&path, project_root);
                        continue;
                    }

                    if path.extension().map(|e| e == "eucmeta").unwrap_or(false) {
                        continue;
                    }

                    if detect_asset_type(&path).is_none() {
                        continue;
                    }

                    let meta_path = PathBuf::from(format!("{}.eucmeta", path.display()));

                    if !meta_path.exists() {
                        match generate_eucmeta(&path, project_root) {
                            Ok(e) => log::info!("asset watcher: created entry '{}'", e.name),
                            Err(e) => log::warn!(
                                "asset watcher: failed to create entry for {}: {e}",
                                path.display()
                            ),
                        }
                    } else {
                        let Some(current_hash) = hash_file(&path) else { continue };
                        let Ok(ron_str) = fs::read_to_string(&meta_path) else { continue };
                        let Ok(mut entry) = ron::de::from_str::<AssetEntry>(&ron_str) else {
                            continue;
                        };

                        if entry.is_stale(&current_hash) {
                            entry.source_hash = current_hash;
                            entry.import_time = SystemTime::now();
                            match ron::ser::to_string_pretty(
                                &entry,
                                ron::ser::PrettyConfig::default(),
                            ) {
                                Ok(updated) => {
                                    if let Err(e) = fs::write(&meta_path, updated) {
                                        log::warn!(
                                            "asset watcher: failed to update entry for {}: {e}",
                                            path.display()
                                        );
                                    } else {
                                        log::info!(
                                            "asset watcher: refreshed stale entry '{}'",
                                            entry.name
                                        );
                                    }
                                }
                                Err(e) => log::warn!(
                                    "asset watcher: RON serialization error for {}: {e}",
                                    path.display()
                                ),
                            }
                        }
                    }
                }
            }

            loop {
                let project_path = PROJECT.read().project_path.clone();

                if project_path.as_os_str().is_empty() {
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }

                let resources_dir = project_path.join("resources");
                if resources_dir.is_dir() {
                    scan_and_update(&resources_dir, &project_path);
                }

                std::thread::sleep(Duration::from_secs(2));
            }
        })
        .expect("failed to spawn asset-entry-watcher thread");
}