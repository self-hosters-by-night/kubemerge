use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

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
    other: HashMap<String, serde_yaml::Value>,
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
    clusters: Option<Vec<NamedCluster>>,
    contexts: Option<Vec<NamedContext>>,
    users: Option<Vec<NamedUser>>,
    #[serde(rename = "current-context")]
    current_context: String,
    preferences: HashMap<String, serde_yaml::Value>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("merge")
        .version("1.0")
        .about("Merges multiple kubeconfig YAML files into a single file")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("DIR")
                .help("Input directory containing kubeconfig files")
                .default_value(format!(
                    "{}/.kube",
                    env::var("HOME").expect("HOME environment variable is undefined")
                )),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output file path")
                .default_value(format!(
                    "{}/.kube/config",
                    env::var("HOME").expect("HOME environment variable is undefined")
                )),
        )
        .arg(
            Arg::new("backup")
                .short('b')
                .long("backup")
                .help("Create backup of existing output file")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let input_dir = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let create_backup = matches.get_flag("backup");

    // Create backup if requested and file exists
    if create_backup && Path::new(output_file).exists() {
        let backup_name = format!("{}.backup", output_file);
        fs::copy(output_file, &backup_name)?;
        println!("Created backup: {}", backup_name);
    }

    // Find all YAML files
    let yaml_files = find_yaml_files(input_dir)?;
    if yaml_files.is_empty() {
        eprintln!("No YAML files found in {}", input_dir);
        std::process::exit(1);
    }

    println!("Found {} kubeconfig files:", yaml_files.len());
    for file in &yaml_files {
        println!("  - {}", file);
    }

    // Parse and merge configs
    let merged_config = merge_kubeconfigs(&yaml_files)?;

    // Write merged config
    let yaml_output = serde_yaml::to_string(&merged_config)?;
    fs::write(output_file, yaml_output)?;

    println!(
        "Successfully merged {} files into {}",
        yaml_files.len(),
        output_file
    );

    // Print summary
    let clusters_count = merged_config
        .clusters
        .as_ref()
        .map(|c| c.len())
        .unwrap_or(0);
    let contexts_count = merged_config
        .contexts
        .as_ref()
        .map(|c| c.len())
        .unwrap_or(0);
    let users_count = merged_config.users.as_ref().map(|u| u.len()).unwrap_or(0);

    println!("Merged config contains:");
    println!("  - {} clusters", clusters_count);
    println!("  - {} contexts", contexts_count);
    println!("  - {} users", users_count);

    Ok(())
}

fn find_yaml_files(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut yaml_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    yaml_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    yaml_files.sort();
    Ok(yaml_files)
}

fn merge_kubeconfigs(files: &[String]) -> Result<KubeConfig, Box<dyn std::error::Error>> {
    let mut all_clusters = Vec::new();
    let mut all_contexts = Vec::new();
    let mut all_users = Vec::new();
    let mut current_context = String::new();
    let mut preferences = HashMap::new();

    for file_path in files {
        println!("Processing: {}", file_path);

        let content = fs::read_to_string(file_path)?;
        let config: KubeConfig = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", file_path, e))?;

        // Merge clusters
        if let Some(clusters) = config.clusters {
            for cluster in clusters {
                if !all_clusters
                    .iter()
                    .any(|c: &NamedCluster| c.name == cluster.name)
                {
                    all_clusters.push(cluster);
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
                }
            }
        }

        // Merge users
        if let Some(users) = config.users {
            for user in users {
                if !all_users.iter().any(|u: &NamedUser| u.name == user.name) {
                    all_users.push(user);
                }
            }
        }

        // Use the first non-empty current-context
        if current_context.is_empty() && !config.current_context.is_empty() {
            current_context = config.current_context;
        }

        // Merge preferences
        for (key, value) in config.preferences {
            preferences.insert(key, value);
        }
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
