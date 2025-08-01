use clap::{Arg, Command};
use std::env;
use std::fs;
use std::path::Path;

mod config;
mod merge;
mod utils;

use merge::merge_kubeconfigs;
use utils::{create_backup, find_yaml_files, print_summary};

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
                .help("Exclude files matching pattern")
                .action(clap::ArgAction::Append),
        )
        .get_matches();

    let input_dir = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let exclude_patterns: Vec<&String> = matches.get_many("exclude").unwrap_or_default().collect();

    if !Path::new(input_dir).is_dir() {
        return Err(format!("Input directory does not exist: {}", input_dir).into());
    }

    if Path::new(output_file).exists() {
        create_backup(output_file)?;
    }

    if let Some(parent) = Path::new(output_file).parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml_files = find_yaml_files(input_dir, &exclude_patterns)?;
    if yaml_files.is_empty() {
        return Err(format!("No kubeconfig YAML files found in {}", input_dir).into());
    }

    println!("Found {} kubeconfig files:", yaml_files.len());
    for file in &yaml_files {
        println!("  - {}", file.display());
    }

    let merged_config = merge_kubeconfigs(&yaml_files)?;
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
