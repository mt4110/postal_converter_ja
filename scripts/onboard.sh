#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROFILE="dev"
ACTION="start"
LOG_DIR="/tmp/postal_converter_ja_onboard"
PID_FILE="${LOG_DIR}/pids.env"

API_PID=""
FRONTEND_PID=""
COMPOSE_DRIVER=""

usage() {
  cat <<'EOF'
Usage:
  ./scripts/onboard.sh [--profile dev|demo|sqlite-release]
  ./scripts/onboard.sh --stop

Profiles:
  dev            Start PostgreSQL + Redis + API + Frontend (no crawler auto-run)
  demo           Start PostgreSQL + Redis + one-shot crawler + API + Frontend
  sqlite-release Start PostgreSQL + one-shot crawler + SQLite artifact packaging
EOF
}

log() {
  printf '[onboard] %s\n' "$1"
}

require_command() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing command: ${cmd}"
    exit 1
  fi
}

init_compose_driver() {
  local compose_version_output

  if compose_version_output="$(docker compose version 2>&1)" \
    && ! printf '%s' "${compose_version_output}" | grep -q "^Docker version " \
    && docker compose up --help >/dev/null 2>&1; then
    COMPOSE_DRIVER="docker compose"
    return 0
  fi

  if command -v docker-compose >/dev/null 2>&1; then
    COMPOSE_DRIVER="docker-compose"
    return 0
  fi

  echo "Missing Docker Compose command."
  echo "Tried: docker compose, docker-compose"
  exit 1
}

compose_cmd() {
  case "${COMPOSE_DRIVER}" in
    "docker compose")
      docker compose "$@"
      ;;
    "docker-compose")
      docker-compose "$@"
      ;;
    *)
      echo "Compose runner is not initialized."
      exit 1
      ;;
  esac
}

source_nix_env() {
  local nix_profile_script="/nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh"
  local nix_profile_bin="/nix/var/nix/profiles/default/bin"

  if [ -f "${nix_profile_script}" ]; then
    # shellcheck disable=SC1091
    source "${nix_profile_script}"
  fi

  if ! command -v nix >/dev/null 2>&1 && [ -x "${nix_profile_bin}/nix" ]; then
    export PATH="${nix_profile_bin}:$PATH"
  fi
}

run_nix() {
  local cmd="$1"
  nix develop --command bash -lc "$cmd"
}

ensure_env_files() {
  if [ ! -f "${ROOT_DIR}/worker/api/.env" ]; then
    cp "${ROOT_DIR}/worker/api/.env.example" "${ROOT_DIR}/worker/api/.env"
    log "Created worker/api/.env from template."
  fi
  if [ ! -f "${ROOT_DIR}/worker/crawler/.env" ]; then
    cp "${ROOT_DIR}/worker/crawler/.env.example" "${ROOT_DIR}/worker/crawler/.env"
    log "Created worker/crawler/.env from template."
  fi
}

wait_for_url() {
  local url="$1"
  local label="$2"
  local max_retries="${3:-60}"
  local sleep_seconds="${4:-1}"

  for _ in $(seq 1 "$max_retries"); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      log "${label} is ready: ${url}"
      return 0
    fi
    sleep "$sleep_seconds"
  done
  echo "${label} did not become ready: ${url}"
  return 1
}

wait_for_postgres() {
  for _ in $(seq 1 60); do
    if docker exec postgres_container pg_isready -U postgres -d zip_code_db >/dev/null 2>&1; then
      log "PostgreSQL is ready."
      return 0
    fi
    sleep 2
  done
  echo "PostgreSQL readiness check failed."
  return 1
}

assert_port_free() {
  local port="$1"
  local label="$2"

  if ! command -v lsof >/dev/null 2>&1; then
    return 0
  fi

  if lsof -nP -iTCP:"${port}" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "${label} could not start because port ${port} is already in use."
    lsof -nP -iTCP:"${port}" -sTCP:LISTEN || true
    return 1
  fi
}

start_api() {
  if curl -fsS http://127.0.0.1:3202/health >/dev/null 2>&1; then
    log "API already running on 127.0.0.1:3202."
    return 0
  fi
  assert_port_free "3202" "API"

  log "Starting API..."
  nohup nix develop --command bash -lc \
    "cd worker/api && DATABASE_TYPE=postgres POSTGRES_DATABASE_URL=postgres://postgres:postgres_password@127.0.0.1:3205/zip_code_db REDIS_URL=redis://127.0.0.1:3206 cargo run --release --bin api" \
    >"${LOG_DIR}/api.log" 2>&1 &
  API_PID="$!"

  wait_for_url "http://127.0.0.1:3202/health" "API health" 90 1
  wait_for_url "http://127.0.0.1:3202/docs" "Swagger UI" 90 1
}

start_frontend() {
  if curl -fsS http://127.0.0.1:3203 >/dev/null 2>&1; then
    log "Frontend already running on 127.0.0.1:3203."
    return 0
  fi
  assert_port_free "3203" "Frontend"

  log "Starting Frontend..."
  nohup nix develop --command bash -lc \
    "cd frontend && if [ ! -d node_modules ]; then yarn install --frozen-lockfile || yarn install; fi && yarn dev" \
    >"${LOG_DIR}/frontend.log" 2>&1 &
  FRONTEND_PID="$!"

  wait_for_url "http://127.0.0.1:3203" "Frontend" 120 1
}

run_crawler_once() {
  log "Running crawler once (latest Japan Post data import)..."
  run_nix "cd worker/crawler && ZIP_CODE_URL=https://www.post.japanpost.jp/zipcode/dl/kogaki/zip/ken_all.zip CRAWLER_INTERVAL_SECONDS=1 CRAWLER_RUN_ONCE=true DATABASE_TYPE=postgres POSTGRES_DATABASE_URL=postgres://postgres:postgres_password@127.0.0.1:3205/zip_code_db REDIS_URL=redis://127.0.0.1:3206 cargo run --release --bin crawler"
}

write_pid_file() {
  cat >"${PID_FILE}" <<EOF
API_PID=${API_PID}
FRONTEND_PID=${FRONTEND_PID}
EOF
}

stop_started_processes() {
  if [ -f "${PID_FILE}" ]; then
    # shellcheck disable=SC1090
    source "${PID_FILE}"
    for pid in "${API_PID:-}" "${FRONTEND_PID:-}"; do
      if [ -n "${pid}" ] && kill -0 "${pid}" >/dev/null 2>&1; then
        kill "${pid}" || true
      fi
    done
    rm -f "${PID_FILE}"
  fi
}

print_next_actions() {
  cat <<EOF

Onboarding completed (profile: ${PROFILE})

API:
  http://127.0.0.1:3202
Swagger:
  http://127.0.0.1:3202/docs
Frontend:
  http://127.0.0.1:3203

Logs:
  ${LOG_DIR}/api.log
  ${LOG_DIR}/frontend.log

Stop command:
  ./scripts/onboard.sh --stop
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --profile)
      if [ "${2:-}" = "" ]; then
        echo "--profile requires a value."
        usage
        exit 1
      fi
      PROFILE="$2"
      shift 2
      ;;
    --stop)
      ACTION="stop"
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

if [ "${ACTION}" = "start" ]; then
  case "${PROFILE}" in
    dev|demo|sqlite-release) ;;
    *)
      echo "Invalid profile: ${PROFILE}"
      usage
      exit 1
      ;;
  esac
fi

mkdir -p "${LOG_DIR}"
cd "${ROOT_DIR}"

source_nix_env
require_command docker
init_compose_driver

if [ "${ACTION}" = "stop" ]; then
  log "Stopping managed local processes..."
  stop_started_processes
  compose_cmd down >/dev/null 2>&1 || true
  log "Stopped. Docker services are down."
  exit 0
fi

require_command nix
require_command curl

ensure_env_files

if [ "${PROFILE}" = "dev" ]; then
  log "Profile=dev: starting PostgreSQL + Redis + API + Frontend"
  compose_cmd up -d postgres redis
  wait_for_postgres
  start_api
  start_frontend
  write_pid_file
  print_next_actions
  exit 0
fi

if [ "${PROFILE}" = "demo" ]; then
  log "Profile=demo: starting PostgreSQL + Redis + one-shot crawler + API + Frontend"
  compose_cmd up -d postgres redis
  wait_for_postgres
  run_crawler_once
  start_api
  start_frontend
  write_pid_file
  print_next_actions
  exit 0
fi

if [ "${PROFILE}" = "sqlite-release" ]; then
  VERSION_LABEL="$(date -u +%Y%m%d)"
  log "Profile=sqlite-release: starting PostgreSQL + one-shot crawler + packaging"
  compose_cmd up -d postgres
  wait_for_postgres
  run_crawler_once
  run_nix "./scripts/package_sqlite_release.sh '${VERSION_LABEL}'"
  compose_cmd down >/dev/null 2>&1 || true

  cat <<EOF

SQLite release artifact generated:
  ${ROOT_DIR}/artifacts/sqlite/postal_codes-${VERSION_LABEL}.sqlite3
  ${ROOT_DIR}/artifacts/sqlite/checksums-${VERSION_LABEL}.txt
  ${ROOT_DIR}/artifacts/sqlite/manifest-${VERSION_LABEL}.txt
EOF
  exit 0
fi
