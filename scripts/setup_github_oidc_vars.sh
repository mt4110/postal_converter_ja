#!/usr/bin/env bash
set -euo pipefail

REPO=""
WRITE=false
USE_DUMMY=false

usage() {
  cat <<'EOF'
Usage:
  ./scripts/setup_github_oidc_vars.sh [--repo owner/repo] [--write] [--use-dummy]

Behavior:
  - Default: dry-run (prints what would be set)
  - --write: actually writes GitHub Secrets/Variables via gh CLI
  - --use-dummy: fills missing values with dummy placeholders

Environment variables (optional):
  AWS_ROLE_TO_ASSUME
  AWS_REGION
  GCP_WORKLOAD_IDENTITY_PROVIDER
  GCP_SERVICE_ACCOUNT
  GCP_PROJECT_ID
  AZURE_CLIENT_ID
  AZURE_TENANT_ID
  AZURE_SUBSCRIPTION_ID

Examples:
  ./scripts/setup_github_oidc_vars.sh --use-dummy
  ./scripts/setup_github_oidc_vars.sh --write --repo mt4110/postal_converter_ja
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --repo)
      REPO="${2:-}"
      if [ -z "${REPO}" ]; then
        echo "--repo requires owner/repo."
        exit 1
      fi
      shift 2
      ;;
    --write)
      WRITE=true
      shift
      ;;
    --use-dummy)
      USE_DUMMY=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      usage
      exit 1
      ;;
  esac
done

if ! command -v gh >/dev/null 2>&1; then
  echo "gh command is required."
  exit 1
fi

if [ "${WRITE}" = true ]; then
  gh auth status -h github.com >/dev/null
fi

repo_args=()
if [ -n "${REPO}" ]; then
  repo_args=(--repo "${REPO}")
fi

aws_role="${AWS_ROLE_TO_ASSUME:-}"
aws_region="${AWS_REGION:-}"
gcp_wip="${GCP_WORKLOAD_IDENTITY_PROVIDER:-}"
gcp_sa="${GCP_SERVICE_ACCOUNT:-}"
gcp_project="${GCP_PROJECT_ID:-}"
azure_client="${AZURE_CLIENT_ID:-}"
azure_tenant="${AZURE_TENANT_ID:-}"
azure_sub="${AZURE_SUBSCRIPTION_ID:-}"

if [ "${USE_DUMMY}" = true ]; then
  aws_role="${aws_role:-arn:aws:iam::123456789012:role/github-actions-terraform}"
  aws_region="${aws_region:-ap-northeast-1}"
  gcp_wip="${gcp_wip:-projects/123456789/locations/global/workloadIdentityPools/github/providers/github}"
  gcp_sa="${gcp_sa:-github-actions@dummy-project.iam.gserviceaccount.com}"
  gcp_project="${gcp_project:-dummy-project}"
  azure_client="${azure_client:-00000000-0000-0000-0000-000000000000}"
  azure_tenant="${azure_tenant:-00000000-0000-0000-0000-000000000000}"
  azure_sub="${azure_sub:-00000000-0000-0000-0000-000000000000}"
fi

set_secret() {
  local key="$1"
  local value="$2"
  if [ -z "${value}" ]; then
    echo "[skip] secret ${key}: empty"
    return
  fi
  if [ "${WRITE}" = true ]; then
    gh secret set "${key}" "${repo_args[@]}" --body "${value}"
    echo "[ok] secret ${key}"
  else
    echo "[dry-run] secret ${key} = ${value}"
  fi
}

set_variable() {
  local key="$1"
  local value="$2"
  if [ -z "${value}" ]; then
    echo "[skip] variable ${key}: empty"
    return
  fi
  if [ "${WRITE}" = true ]; then
    gh variable set "${key}" "${repo_args[@]}" --body "${value}"
    echo "[ok] variable ${key}"
  else
    echo "[dry-run] variable ${key} = ${value}"
  fi
}

set_secret AWS_ROLE_TO_ASSUME "${aws_role}"
set_variable AWS_REGION "${aws_region}"

set_secret GCP_WORKLOAD_IDENTITY_PROVIDER "${gcp_wip}"
set_secret GCP_SERVICE_ACCOUNT "${gcp_sa}"
set_variable GCP_PROJECT_ID "${gcp_project}"

set_secret AZURE_CLIENT_ID "${azure_client}"
set_secret AZURE_TENANT_ID "${azure_tenant}"
set_secret AZURE_SUBSCRIPTION_ID "${azure_sub}"

if [ "${WRITE}" = true ]; then
  echo "Done: GitHub Secrets/Variables updated."
else
  echo "Dry-run completed. Re-run with --write to apply."
fi
