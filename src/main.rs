use chrono::Local;
use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Cluster {
    #[serde(
        rename = "certificate-authority-data",
        skip_serializing_if = "Option::is_none"
    )]
    certificate_authority_data: Option<String>,
    #[serde(
        rename = "certificate-authority",
        skip_serializing_if = "Option::is_none"
    )]
    certificate_authority: Option<String>,
    server: String,
    #[serde(
        rename = "insecure-skip-tls-verify",
        skip_serializing_if = "Option::is_none"
    )]
    insecure_skip_tls_verify: Option<bool>,
    #[serde(flatten)]
    other: HashMap<String, serde_yml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NamedCluster {
    name: String,
    cluster: Cluster,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Context {
    cluster: String,
    user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    namespace: Option<String>,
    #[serde(flatten)]
    other: HashMap<String, serde_yml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NamedContext {
    name: String,
    context: Context,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    #[serde(
        rename = "client-certificate-data",
        skip_serializing_if = "Option::is_none"
    )]
    client_certificate_data: Option<String>,
    #[serde(rename = "client-key-data", skip_serializing_if = "Option::is_none")]
    client_key_data: Option<String>,
    #[serde(rename = "client-certificate", skip_serializing_if = "Option::is_none")]
    client_certificate: Option<String>,
    #[serde(rename = "client-key", skip_serializing_if = "Option::is_none")]
    client_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(flatten)]
    other: HashMap<String, serde_yml::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NamedUser {
    name: String,
    user: User,
}

#[derive(Debug, Serialize, Deserialize)]
struct KubeConfig {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    clusters: Option<Vec<NamedCluster>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contexts: Option<Vec<NamedContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    users: Option<Vec<NamedUser>>,
    #[serde(rename = "current-context", skip_serializing_if = "String::is_empty")]
    current_context: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    preferences: HashMap<String, serde_yml::Value>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| "HOME or USERPROFILE environment variable not found")?;

    let matches = Command::new("kubemerge")
        .version("0.2.0")
        .about("Merges multiple kubeconfig YAML files into a single file")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("DIR")
                .help("Input directory containing kubeconfig files")
                .default_value(format!("{}/.kube", home_dir)),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path")
                .default_value(format!("{}/.kube/config", home_dir)),
        )
        .arg(
            Arg::new("exclude")
                .short('e')
                .long("exclude")
                .value_name("PATTERN")
                .help("Exclude files matching pattern (e.g., 'backup', 'config')")
                .action(clap::ArgAction::Append),
        )
        .get_matches();

    let input_dir = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let exclude_patterns: Vec<&String> = matches.get_many("exclude").unwrap_or_default().collect();

    // Validate input directory exists
    if !Path::new(input_dir).is_dir() {
        return Err(format!("Input directory does not exist: {}", input_dir).into());
    }

    // Always create backup if output file exists
    if Path::new(output_file).exists() {
        create_backup(output_file)?;
    }

    // Ensure output directory exists
    if let Some(parent) = Path::new(output_file).parent() {
        fs::create_dir_all(parent)?;
    }

    // Find all YAML files
    let yaml_files = find_yaml_files(input_dir, &exclude_patterns)?;
    if yaml_files.is_empty() {
        return Err(format!("No kubeconfig YAML files found in {}", input_dir).into());
    }

    println!("Found {} kubeconfig files:", yaml_files.len());
    for file in &yaml_files {
        println!("  - {}", file.display());
    }

    // Parse and merge configs
    let merged_config = merge_kubeconfigs(&yaml_files)?;

    // Validate merged config
    validate_merged_config(&merged_config)?;

    // Write merged config
    let yaml_output = serde_yml::to_string(&merged_config)?;
    fs::write(output_file, yaml_output)?;

    println!(
        "Successfully merged {} files into {}",
        yaml_files.len(),
        output_file
    );
    print_summary(&merged_config);

    Ok(())
}

fn create_backup(output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let backup_name = format!("{}.backup.{}", output_file, timestamp);
    fs::copy(output_file, &backup_name)?;
    println!("Created backup: {}", backup_name);
    Ok(())
}

fn find_yaml_files(
    dir: &str,
    exclude_patterns: &[&String],
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut yaml_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if (extension == "yaml" || extension == "yml")
                    && !should_exclude(&path, exclude_patterns)
                {
                    yaml_files.push(path);
                }
            }
        }
    }

    yaml_files.sort();
    Ok(yaml_files)
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

fn merge_kubeconfigs(files: &[PathBuf]) -> Result<KubeConfig, Box<dyn std::error::Error>> {
    let mut all_clusters = Vec::new();
    let mut all_contexts = Vec::new();
    let mut all_users = Vec::new();
    let mut current_context = String::new();
    let mut preferences = HashMap::new();
    let mut processed_files = 0;

    for file_path in files {
        println!("Processing: {}", file_path.display());

        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

        if content.trim().is_empty() {
            println!("  Skipping empty file");
            continue;
        }

        let config: KubeConfig = serde_yml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", file_path.display(), e))?;

        let mut added_items = 0;

        // Merge clusters
        if let Some(clusters) = config.clusters {
            for cluster in clusters {
                if !all_clusters
                    .iter()
                    .any(|c: &NamedCluster| c.name == cluster.name)
                {
                    all_clusters.push(cluster);
                    added_items += 1;
                } else {
                    println!("  Skipping duplicate cluster: {}", cluster.name);
                }
            }
        }

        // Merge contexts
        if let Some(contexts) = config.contexts {
            for context in contexts {
                if !all_contexts
                    .iter()
                    .any(|c: &NamedContext| c.name == context.name)
                {
                    all_contexts.push(context);
                    added_items += 1;
                } else {
                    println!("  Skipping duplicate context: {}", context.name);
                }
            }
        }

        // Merge users
        if let Some(users) = config.users {
            for user in users {
                if !all_users.iter().any(|u: &NamedUser| u.name == user.name) {
                    all_users.push(user);
                    added_items += 1;
                } else {
                    println!("  Skipping duplicate user: {}", user.name);
                }
            }
        }

        // Use the first non-empty current-context
        if current_context.is_empty() && !config.current_context.is_empty() {
            current_context = config.current_context;
            println!("  Using current-context: {}", current_context);
        }

        // Merge preferences
        for (key, value) in config.preferences {
            preferences.insert(key, value);
        }

        if added_items > 0 {
            processed_files += 1;
            println!("  Added {} items", added_items);
        } else {
            println!("  No new items added");
        }
    }

    if processed_files == 0 {
        return Err("No valid kubeconfig files were processed".into());
    }

    Ok(KubeConfig {
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
    })
}

fn validate_merged_config(config: &KubeConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Check if current-context exists in contexts
    if !config.current_context.is_empty() {
        if let Some(contexts) = &config.contexts {
            if !contexts.iter().any(|c| c.name == config.current_context) {
                return Err(format!(
                    "Current context '{}' not found in merged contexts",
                    config.current_context
                )
                .into());
            }
        }
    }

    // Validate context references
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
                println!(
                    "Warning: Context '{}' references missing cluster '{}'",
                    context.name, context.context.cluster
                );
            }
            if !user_names.contains(&&context.context.user) {
                println!(
                    "Warning: Context '{}' references missing user '{}'",
                    context.name, context.context.user
                );
            }
        }
    }

    Ok(())
}

fn print_summary(config: &KubeConfig) {
    let clusters_count = config.clusters.as_ref().map(|c| c.len()).unwrap_or(0);
    let contexts_count = config.contexts.as_ref().map(|c| c.len()).unwrap_or(0);
    let users_count = config.users.as_ref().map(|u| u.len()).unwrap_or(0);

    println!("\nMerged config contains:");
    println!("  - {} clusters", clusters_count);
    println!("  - {} contexts", contexts_count);
    println!("  - {} users", users_count);

    if !config.current_context.is_empty() {
        println!("  - Current context: {}", config.current_context);
    } else {
        println!("  - No current context set");
    }
}
