#!/usr/bin/env bash
set -euo pipefail

REPO=""
REF=""
ENVIRONMENT="dev"
ACTION="plan"
CONFIRM_APPLY=""
CONFIRM_DESTROY=""
WATCH=true

usage() {
  cat <<'EOF'
Usage:
  ./scripts/run_terraform_workflow.sh [--repo owner/repo] [--ref branch] --action validate|plan|apply|destroy [--environment dev|stg|prod] [--confirm-apply APPLY_AWS] [--confirm-destroy DESTROY_AWS] [--no-watch]

Examples:
  ./scripts/run_terraform_workflow.sh --action plan --environment dev --ref feature/v0.8.0
  ./scripts/run_terraform_workflow.sh --action apply --environment dev --confirm-apply APPLY_AWS --ref feature/v0.8.0
  ./scripts/run_terraform_workflow.sh --action destroy --environment dev --confirm-destroy DESTROY_AWS --ref feature/v0.8.0
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
    --ref)
      REF="${2:-}"
      if [ -z "${REF}" ]; then
        echo "--ref requires branch."
        exit 1
      fi
      if [ "${REF#refs/tags/}" != "${REF}" ]; then
        echo "--ref must be a branch, tag refs are not supported."
        exit 1
      fi
      shift 2
      ;;
    --environment)
      ENVIRONMENT="${2:-}"
      shift 2
      ;;
    --action)
      ACTION="${2:-}"
      shift 2
      ;;
    --confirm-apply)
      CONFIRM_APPLY="${2:-}"
      shift 2
      ;;
    --confirm-destroy)
      CONFIRM_DESTROY="${2:-}"
      shift 2
      ;;
    --no-watch)
      WATCH=false
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

case "${ENVIRONMENT}" in
  dev|stg|prod) ;;
  *)
    echo "Invalid environment: ${ENVIRONMENT}"
    exit 1
    ;;
esac

case "${ACTION}" in
  validate|plan|apply|destroy) ;;
  *)
    echo "Invalid action: ${ACTION}"
    exit 1
    ;;
esac

if ! command -v gh >/dev/null 2>&1; then
  echo "gh command is required."
  exit 1
fi

gh auth status -h github.com >/dev/null

repo_args=()
if [ -n "${REPO}" ]; then
  repo_args=(--repo "${REPO}")
fi

ref_args=()
if [ -n "${REF}" ]; then
  ref_args=(--ref "${REF}")
fi

if [ "${ACTION}" = "apply" ] && [ "${CONFIRM_APPLY}" != "APPLY_AWS" ]; then
  echo "apply requires --confirm-apply APPLY_AWS"
  exit 1
fi

if [ "${ACTION}" = "destroy" ] && [ "${CONFIRM_DESTROY}" != "DESTROY_AWS" ]; then
  echo "destroy requires --confirm-destroy DESTROY_AWS"
  exit 1
fi

dispatch_epoch="$(date -u +%s)"

gh workflow run terraform-multiplatform.yml "${repo_args[@]}" "${ref_args[@]}" \
  -f environment="${ENVIRONMENT}" \
  -f action="${ACTION}" \
  -f confirm_apply="${CONFIRM_APPLY}" \
  -f confirm_destroy="${CONFIRM_DESTROY}"

echo "workflow dispatched: action=${ACTION}, environment=${ENVIRONMENT}"

if [ "${WATCH}" = true ]; then
  if ! command -v jq >/dev/null 2>&1; then
    echo "jq command is required for --watch mode."
    exit 1
  fi

  watch_branch="${REF}"
  if [ -z "${watch_branch}" ]; then
    watch_branch="$(git rev-parse --abbrev-ref HEAD)"
  fi

  run_id=""
  for _ in $(seq 1 15); do
    run_id="$(gh run list "${repo_args[@]}" \
      --workflow "Terraform AWS Baseline" \
      --branch "${watch_branch}" \
      --limit 20 \
      --json databaseId,event,createdAt \
      | jq -r --argjson t "${dispatch_epoch}" '
          [.[] | select(.event=="workflow_dispatch") | select((.createdAt | fromdateiso8601) >= ($t - 60))][0].databaseId
        ')"
    if [ -n "${run_id}" ] && [ "${run_id}" != "null" ]; then
      break
    fi
    sleep 2
  done

  if [ -z "${run_id}" ] || [ "${run_id}" = "null" ]; then
    echo "Could not resolve latest run id for branch ${watch_branch}."
    echo "Recent runs:"
    gh run list "${repo_args[@]}" --workflow "Terraform AWS Baseline" --branch "${watch_branch}" --limit 5
    exit 1
  fi

  gh run watch "${run_id}" "${repo_args[@]}" --exit-status
fi
