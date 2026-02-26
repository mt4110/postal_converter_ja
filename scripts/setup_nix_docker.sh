#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE="dev"
INSTALL_MISSING=false
SKIP_ONBOARD=false

usage() {
  cat <<'EOF'
Usage:
  ./scripts/setup_nix_docker.sh [options]

Options:
  --profile dev|demo|sqlite-release  Onboarding profile (default: dev)
  --install-missing                  Try to install missing Nix/Docker tools (macOS only)
  --skip-onboard                     Only setup prerequisites and files, do not run onboard
  -h, --help                         Show this help

Examples:
  ./scripts/setup_nix_docker.sh --profile dev
  ./scripts/setup_nix_docker.sh --install-missing --profile demo
EOF
}

log() {
  printf '[setup] %s\n' "$1"
}

require_in_path() {
  local cmd="$1"
  command -v "$cmd" >/dev/null 2>&1
}

install_on_macos() {
  if ! require_in_path brew; then
    echo "Homebrew is required for --install-missing on macOS."
    echo "Install Homebrew first: https://brew.sh/"
    exit 1
  fi

  if ! require_in_path nix; then
    source_nix_if_available || true
  fi

  if ! require_in_path nix; then
    log "Installing Nix (Determinate Systems installer)..."
    curl -fsSL https://install.determinate.systems/nix | sh -s -- install --no-confirm
  fi

  if ! require_in_path docker; then
    log "Installing Docker CLI..."
    brew install docker
  fi

  if ! require_in_path colima; then
    log "Installing Colima..."
    brew install colima
  fi
}

source_nix_if_available() {
  local nix_profile_script="/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh"
  local nix_profile_bin="/nix/var/nix/profiles/default/bin"

  if [ -f "${nix_profile_script}" ]; then
    # shellcheck disable=SC1091
    source "${nix_profile_script}"
  fi

  if ! require_in_path nix && [ -x "${nix_profile_bin}/nix" ]; then
    export PATH="${nix_profile_bin}:$PATH"
  fi

  if require_in_path nix; then
    return 0
  fi

  return 1
}

ensure_docker_daemon() {
  if docker info >/dev/null 2>&1; then
    log "Docker daemon is available."
    return 0
  fi

  if require_in_path colima; then
    log "Starting Colima..."
    colima start >/dev/null
    if docker info >/dev/null 2>&1; then
      log "Docker daemon is available via Colima."
      return 0
    fi
  fi

  echo "Docker daemon is not available."
  echo "Start Docker Desktop or Colima, then retry."
  exit 1
}

ensure_compose_command() {
  local compose_version_output

  if compose_version_output="$(docker compose version 2>&1)" \
    && ! printf '%s' "${compose_version_output}" | grep -q "^Docker version " \
    && docker compose up --help >/dev/null 2>&1; then
    log "Docker Compose is available via: docker compose"
    return 0
  fi

  if require_in_path docker-compose; then
    log "Docker Compose is available via: docker-compose"
    return 0
  fi

  echo "Docker Compose command is not available."
  echo "Install Docker Compose plugin or docker-compose, then retry."
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --profile)
      PROFILE="${2:-}"
      if [ -z "$PROFILE" ]; then
        echo "--profile requires a value."
        exit 1
      fi
      shift 2
      ;;
    --install-missing)
      INSTALL_MISSING=true
      shift
      ;;
    --skip-onboard)
      SKIP_ONBOARD=true
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

case "$PROFILE" in
  dev|demo|sqlite-release) ;;
  *)
    echo "Invalid profile: $PROFILE"
    exit 1
    ;;
esac

if [ "$INSTALL_MISSING" = true ]; then
  case "$(uname -s)" in
    Darwin)
      install_on_macos
      ;;
    *)
      echo "--install-missing is currently supported only on macOS."
      echo "Please install Nix and Docker manually for your OS, then rerun."
      exit 1
      ;;
  esac
fi

if ! require_in_path nix; then
  if ! source_nix_if_available; then
    echo "nix command not found. Install Nix first."
    exit 1
  fi
fi

if ! require_in_path nix; then
  echo "nix command is still unavailable after sourcing."
  exit 1
fi

require_in_path docker || {
  echo "docker command not found."
  exit 1
}

ensure_docker_daemon
ensure_compose_command

log "Preparing project files..."
mkdir -p "${ROOT_DIR}/storage/mysql/mysql_data" \
         "${ROOT_DIR}/storage/postgres/postgres_data" \
         "${ROOT_DIR}/storage/sqlite" \
         "${ROOT_DIR}/artifacts/sqlite"

if [ ! -f "${ROOT_DIR}/worker/api/.env" ]; then
  cp "${ROOT_DIR}/worker/api/.env.example" "${ROOT_DIR}/worker/api/.env"
  log "Created worker/api/.env"
fi

if [ ! -f "${ROOT_DIR}/worker/crawler/.env" ]; then
  cp "${ROOT_DIR}/worker/crawler/.env.example" "${ROOT_DIR}/worker/crawler/.env"
  log "Created worker/crawler/.env"
fi

log "Checking nix develop environment..."
nix develop --command bash -lc "echo nix-develop-ok"

if [ "$SKIP_ONBOARD" = true ]; then
  cat <<EOF

Setup completed.
Next command:
  ./scripts/onboard.sh --profile ${PROFILE}
EOF
  exit 0
fi

log "Running onboard profile: ${PROFILE}"
cd "${ROOT_DIR}"
./scripts/onboard.sh --profile "${PROFILE}"
