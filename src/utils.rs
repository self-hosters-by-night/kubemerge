use crate::config::KubeConfig;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

pub fn create_backup(output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let backup_name = format!("{}.backup.{}", output_file, timestamp);
    fs::copy(output_file, &backup_name)?;
    info!("Created backup: {}", backup_name);
    Ok(())
}

pub fn find_yaml_files(
    dir: &str,
    exclude_patterns: &[&String],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut yaml_files = Vec::new();

    debug!("Scanning directory: {}", dir);
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_yaml_file(&path) && !should_exclude(&path, exclude_patterns) {
            debug!("Found YAML file: {}", path.display());
            yaml_files.push(path);
        } else if should_exclude(&path, exclude_patterns) {
            debug!("Excluded file: {}", path.display());
        }
    }

    yaml_files.sort();
    debug!("Found {} YAML files total", yaml_files.len());
    Ok(yaml_files)
}

fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "yaml" || ext == "yml")
        .unwrap_or(false)
}

fn should_exclude(path: &Path, exclude_patterns: &[&String]) -> bool {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    exclude_patterns
        .iter()
        .any(|pattern| filename.contains(pattern.as_str()))
}

pub fn print_summary(config: &KubeConfig) {
    let clusters_count = config.clusters.as_ref().map(|c| c.len()).unwrap_or(0);
    let contexts_count = config.contexts.as_ref().map(|c| c.len()).unwrap_or(0);
    let users_count = config.users.as_ref().map(|u| u.len()).unwrap_or(0);

    info!("Merged config contains:");
    info!("  - {} clusters", clusters_count);
    info!("  - {} contexts", contexts_count);
    info!("  - {} users", users_count);

    if !config.current_context.is_empty() {
        info!("  - Current context: {}", config.current_context);
    } else {
        info!("  - No current context set");
    }
}
