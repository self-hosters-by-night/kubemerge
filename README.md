# kubemerge

Simple CLI to merge multiple kubeconfigs.

## Features

- Finds all `.yaml`/`.yml` files in the input directory
- Parses each kubeconfig file
- Merges clusters, contexts, and users
- Deduplicates entries by name
- Uses first non-empty current-context found
- Outputs summary of merged resources
- Backups current kubeconfig

## Build

```shell
git clone https://github.com/self-hosters-by-night/kubemerge.git
cargo build --release
```

## Usage:

Store each kubeconfig under `$HOME/.kube` with extension `yaml` or `yml` and then run:

```shell
./target/release/kubemerge -h
```

The resulting kubeconfig will be stored (by default) in `$HOME/.kube/config`.
