#!/bin/bash

set -euo pipefail  # Exit on error, undefined vars, pipe failures

# Function to print colored output
print_info() {
    echo -e "\033[1;34m[INFO]\033[0m $1"
}

print_success() {
    echo -e "\033[1;32m[SUCCESS]\033[0m $1"
}

print_error() {
    echo -e "\033[1;31m[ERROR]\033[0m $1"
}

print_warning() {
    echo -e "\033[1;33m[WARNING]\033[0m $1"
}

# Check if kubectl is installed
if ! command -v kubectl &> /dev/null; then
    print_error "kubectl is not installed or not in PATH"
    exit 1
fi

if ! [[ -v "KUBECONFIG" ]]; then
KUBECONFIG="$HOME/.kube/config"
fi

if [[ -f "$KUBECONFIG" ]]; then
    # Make a timestamped backup of your kubeconfig file
    BACKUP_FILE="$HOME/.kube/config-backup-$(date +%Y%m%d-%H%M%S)"
    print_info "Creating backup: $BACKUP_FILE"
    cp $KUBECONFIG "$BACKUP_FILE"
else
    print_warning "No existing kubeconfig found at $KUBECONFIG"
fi

# Find kubeconfig files - look for common patterns
print_info "Searching for kubeconfig files..."
KUBECONFIG_FILES=$(find $HOME/.kube -type f \( -name "*.yaml" -o -name "*.yml" \) 2>/dev/null)

if [[ -z "$KUBECONFIG_FILES" ]]; then
    print_error "No kubeconfig files found in current directory"
    exit 1
fi

print_info "Found kubeconfig files:"
echo "$KUBECONFIG_FILES" | while read -r file; do
    echo "  - $file"
done

# Store original kubeconfig
ORIG_KUBECONFIG=$KUBECONFIG

# Add found files to KUBECONFIG path
while IFS= read -r file; do
    KUBECONFIG="$KUBECONFIG:$file"
done <<< "$KUBECONFIG_FILES"

# Verify all configs are valid and merge them
print_info "Validating and merging kubeconfig files..."
if kubectl config view --flatten > "$ORIG_KUBECONFIG" 2>&1; then
    print_success "Successfully merged kubeconfig files"

    # Reset the KUBECONFIG to use only the merged file
    export KUBECONFIG="$ORIG_KUBECONFIG"

    # Set proper permissions
    chmod 600 $KUBECONFIG

    print_info "Available clusters:"
    kubectl config get-clusters | grep -v NAME | sed 's/^/  - /'
    
    print_info "Available contexts:"
    kubectl config get-contexts --output=name | sed 's/^/  - /'
else
    print_error "Merged kubeconfig appears to be invalid"
    print_info "Restoring from backup..."
    cp "$BACKUP_FILE" $ORIG_KUBECONFIG
    print_warning "Original configuration restored"
    exit 1
fi
