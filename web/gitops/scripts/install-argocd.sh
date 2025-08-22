#!/bin/bash
# ArgoCD Installation Script for SIS AI-Lab
# Multi-AI Consultation Synthesized Production Setup

set -euo pipefail

# Logging function
log() {
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] $*" >&2
}

# Configuration
readonly ARGOCD_VERSION="v2.9.3"
readonly NAMESPACE="argocd"
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly CONFIG_DIR="${SCRIPT_DIR}/../clusters"

# Validate environment
validate_environment() {
    log "Validating environment..."
    
    # Check required tools
    local required_tools=("kubectl" "helm" "curl")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log "ERROR: $tool is required but not installed"
            exit 1
        fi
    done
    
    # Validate kubectl connection
    if ! kubectl cluster-info &> /dev/null; then
        log "ERROR: Unable to connect to Kubernetes cluster"
        exit 1
    fi
    
    # Check if running as non-root
    if [[ $EUID -eq 0 ]]; then
        log "ERROR: This script should not be run as root"
        exit 1
    fi
    
    log "Environment validation completed"
}

# Install ArgoCD with security hardening
install_argocd() {
    log "Installing ArgoCD version $ARGOCD_VERSION..."
    
    # Create namespace with security labels
    kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | \
    kubectl label --local -f - \
        security.kubernetes.io/enforce=restricted \
        pod-security.kubernetes.io/enforce=restricted \
        pod-security.kubernetes.io/audit=restricted \
        pod-security.kubernetes.io/warn=restricted \
        --dry-run=client -o yaml | \
    kubectl apply -f -
    
    # Download and verify ArgoCD manifests
    local manifest_url="https://raw.githubusercontent.com/argoproj/argo-cd/$ARGOCD_VERSION/manifests/install.yaml"
    local temp_manifest
    temp_manifest=$(mktemp)
    
    if ! curl -fsSL "$manifest_url" -o "$temp_manifest"; then
        log "ERROR: Failed to download ArgoCD manifests"
        rm -f "$temp_manifest"
        exit 1
    fi
    
    # Apply manifests
    kubectl apply -n "$NAMESPACE" -f "$temp_manifest"
    rm -f "$temp_manifest"
    
    # Apply security configurations
    if [[ -f "$CONFIG_DIR/argocd-installation.yaml" ]]; then
        kubectl apply -f "$CONFIG_DIR/argocd-installation.yaml"
    else
        log "WARNING: ArgoCD configuration file not found"
    fi
    
    log "ArgoCD installation completed"
}

# Install Argo Rollouts
install_argo_rollouts() {
    log "Installing Argo Rollouts..."
    
    # Create namespace
    kubectl create namespace argo-rollouts --dry-run=client -o yaml | kubectl apply -f -
    
    # Install Argo Rollouts
    kubectl apply -n argo-rollouts -f https://github.com/argoproj/argo-rollouts/releases/latest/download/install.yaml
    
    log "Argo Rollouts installation completed"
}

# Wait for deployments to be ready
wait_for_ready() {
    log "Waiting for ArgoCD components to be ready..."
    
    local deployments=("argocd-server" "argocd-dex-server" "argocd-repo-server")
    for deployment in "${deployments[@]}"; do
        if ! kubectl wait --for=condition=available --timeout=600s "deployment/$deployment" -n "$NAMESPACE"; then
            log "ERROR: $deployment failed to become ready"
            exit 1
        fi
    done
    
    # Wait for application controller
    if ! kubectl wait --for=condition=ready --timeout=600s pod -l app.kubernetes.io/name=argocd-application-controller -n "$NAMESPACE"; then
        log "ERROR: ArgoCD application controller failed to become ready"
        exit 1
    fi
    
    # Wait for Argo Rollouts
    if ! kubectl wait --for=condition=available --timeout=300s deployment/argo-rollouts -n argo-rollouts; then
        log "ERROR: Argo Rollouts failed to become ready"
        exit 1
    fi
    
    log "All components are ready"
}

# Setup initial security configurations
setup_security() {
    log "Configuring security settings..."
    
    # Disable initial admin secret auto-generation after first use
    kubectl patch secret argocd-initial-admin-secret -n "$NAMESPACE" -p '{"metadata":{"annotations":{"argocd.argoproj.io/generated":"false"}}}'
    
    # Create network policies
    cat <<EOF | kubectl apply -f -
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: argocd-network-policy
  namespace: $NAMESPACE
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/part-of: argocd
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: istio-system
    - namespaceSelector:
        matchLabels:
          name: monitoring
  egress:
  - to: []
    ports:
    - protocol: TCP
      port: 443
    - protocol: TCP
      port: 6443
EOF
    
    log "Security configuration completed"
}

# Verify installation
verify_installation() {
    log "Verifying installation..."
    
    # Check pod status
    local failed_pods
    failed_pods=$(kubectl get pods -n "$NAMESPACE" --field-selector=status.phase!=Running --no-headers 2>/dev/null | wc -l)
    
    if [[ $failed_pods -gt 0 ]]; then
        log "WARNING: $failed_pods pods are not in Running state"
        kubectl get pods -n "$NAMESPACE"
    fi
    
    # Check Argo Rollouts
    if ! kubectl get pods -n argo-rollouts -l app.kubernetes.io/name=argo-rollouts --no-headers | grep -q "Running"; then
        log "ERROR: Argo Rollouts is not running"
        exit 1
    fi
    
    log "Installation verification completed"
}

# Get initial admin password securely
get_admin_password() {
    log "Retrieving initial admin credentials..."
    
    local password
    password=$(kubectl -n "$NAMESPACE" get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" 2>/dev/null | base64 -d 2>/dev/null || echo "")
    
    if [[ -z "$password" ]]; then
        log "WARNING: Unable to retrieve initial admin password"
        log "You may need to reset the admin password manually"
        return 1
    fi
    
    log "Initial admin password retrieved"
    log "IMPORTANT: Change this password immediately after first login"
    echo "Admin password: $password"
    
    return 0
}

# Main execution
main() {
    log "Starting ArgoCD installation for SIS AI-Lab"
    log "Multi-AI Consultation: Production-Grade GitOps Setup"
    
    validate_environment
    install_argocd
    install_argo_rollouts
    wait_for_ready
    setup_security
    verify_installation
    
    if get_admin_password; then
        log "Installation completed successfully"
        log "Access ArgoCD UI: kubectl port-forward svc/argocd-server -n argocd 8080:443"
        log "Login with username: admin"
    else
        log "Installation completed with warnings"
    fi
    
    log "Next steps:"
    log "1. Change the default admin password"
    log "2. Configure OIDC authentication"
    log "3. Deploy applications using GitOps"
}

# Execute main function
main "$@"