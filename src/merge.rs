use crate::config::{KubeConfig, NamedCluster, NamedContext, NamedUser};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

pub fn merge_kubeconfigs(files: &[PathBuf]) -> Result<KubeConfig, Box<dyn std::error::Error>> {
    let mut all_clusters = Vec::new();
    let mut all_contexts = Vec::new();
    let mut all_users = Vec::new();
    let mut current_context = String::new();
    let mut preferences = HashMap::new();
    let mut processed_files = 0;

    for file_path in files {
        info!("Processing: {}", file_path.display());

        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

        if content.trim().is_empty() {
            debug!("Skipping empty file: {}", file_path.display());
            continue;
        }

        let config: KubeConfig = serde_yml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", file_path.display(), e))?;

        let added_items = merge_config_items(
            &config,
            &mut all_clusters,
            &mut all_contexts,
            &mut all_users,
        );

        if current_context.is_empty() && !config.current_context.is_empty() {
            current_context = config.current_context;
            info!("Using current-context: {}", current_context);
        }

        for (key, value) in config.preferences {
            preferences.insert(key, value);
        }

        if added_items > 0 {
            processed_files += 1;
            info!("Added {} items from {}", added_items, file_path.display());
        } else {
            debug!("No new items added from {}", file_path.display());
        }
    }

    if processed_files == 0 {
        error!("No valid kubeconfig files were processed");
        return Err("No valid kubeconfig files were processed".into());
    }

    let merged = KubeConfig {
        api_version: "v1".to_string(),
        kind: "Config".to_string(),
        clusters: if all_clusters.is_empty() {
            None
        } else {
            Some(all_clusters)
        },
        contexts: if all_contexts.is_empty() {
            None
        } else {
            Some(all_contexts)
        },
        users: if all_users.is_empty() {
            None
        } else {
            Some(all_users)
        },
        current_context,
        preferences,
    };

    validate_config(&merged)?;
    Ok(merged)
}

fn merge_config_items(
    config: &KubeConfig,
    all_clusters: &mut Vec<NamedCluster>,
    all_contexts: &mut Vec<NamedContext>,
    all_users: &mut Vec<NamedUser>,
) -> usize {
    let mut added_items = 0;

    if let Some(clusters) = &config.clusters {
        for cluster in clusters {
            if !all_clusters.iter().any(|c| c.name == cluster.name) {
                debug!("Adding cluster: {}", cluster.name);
                all_clusters.push(cluster.clone());
                added_items += 1;
            } else {
                debug!("Skipping duplicate cluster: {}", cluster.name);
            }
        }
    }

    if let Some(contexts) = &config.contexts {
        for context in contexts {
            if !all_contexts.iter().any(|c| c.name == context.name) {
                debug!("Adding context: {}", context.name);
                all_contexts.push(context.clone());
                added_items += 1;
            } else {
                debug!("Skipping duplicate context: {}", context.name);
            }
        }
    }

    if let Some(users) = &config.users {
        for user in users {
            if !all_users.iter().any(|u| u.name == user.name) {
                debug!("Adding user: {}", user.name);
                all_users.push(user.clone());
                added_items += 1;
            } else {
                debug!("Skipping duplicate user: {}", user.name);
            }
        }
    }

    added_items
}

fn validate_config(config: &KubeConfig) -> Result<(), Box<dyn std::error::Error>> {
    if !config.current_context.is_empty() {
        if let Some(contexts) = &config.contexts {
            if !contexts.iter().any(|c| c.name == config.current_context) {
                error!(
                    "Current context '{}' not found in merged contexts",
                    config.current_context
                );
                return Err(format!(
                    "Current context '{}' not found in merged contexts",
                    config.current_context
                )
                .into());
            }
        }
    }

    if let Some(contexts) = &config.contexts {
        let cluster_names: Vec<&String> = config
            .clusters
            .as_ref()
            .map(|c| c.iter().map(|cluster| &cluster.name).collect())
            .unwrap_or_default();
        let user_names: Vec<&String> = config
            .users
            .as_ref()
            .map(|u| u.iter().map(|user| &user.name).collect())
            .unwrap_or_default();

        for context in contexts {
            if !cluster_names.contains(&&context.context.cluster) {
                warn!(
                    "Context '{}' references missing cluster '{}'",
                    context.name, context.context.cluster
                );
            }
            if !user_names.contains(&&context.context.user) {
                warn!(
                    "Context '{}' references missing user '{}'",
                    context.name, context.context.user
                );
            }
        }
    }

    Ok(())
}
