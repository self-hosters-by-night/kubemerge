# merge-kubeconfigs

Simple CLI to merge multiple kubeconfigs.

## Features

- Finds all .yaml/.yml files in the input directory
- Parses each kubeconfig file
- Merges clusters, contexts, and users
- Deduplicates entries by name
- Uses first non-empty current-context found
- Creates backup if requested
- Outputs summary of merged resources
- Backup functionality

## Usage:

```shell
cargo build --release ./target/release/merge
```
