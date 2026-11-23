#!/bin/bash
set -euo pipefail

cd -- "$(dirname -- "$0")/.."
mode=${1:-all}
case "$mode" in
  all|source|images) ;;
  *) echo "Usage: $0 [source|images]" >&2; exit 2 ;;
esac

require_tools() {
  local tool
  for tool in "$@"; do
    if ! command -v "$tool" >/dev/null; then
      echo "Security scan requires $tool." >&2
      exit 2
    fi
  done
}

require_tools osv-scanner jq

scratch=$(mktemp -d "${TMPDIR:-/tmp}/postal-osv.XXXXXXXX")
trap 'rm -rf -- "$scratch"' EXIT
printf '\n' > "$scratch/image.toml"
printf '{}\n' > "$scratch/trivy.yaml"

scan() {
  local inventory="$1" report="$scratch/report.json" status=0
  shift
  : > "$report" || return 2
  osv-scanner scan "$@" --all-vulns --all-packages --format json --output-file "$report" || status=$?

  if [[ "$status" != 0 && "$status" != 1 ]]; then
    echo "OSV-Scanner could not complete the scan (exit $status)." >&2
    return 2
  fi
  if ! jq -e '.results | type == "array"' "$report" >/dev/null; then
    echo "OSV-Scanner returned an invalid report." >&2
    return 2
  fi
  if [[ "$inventory" == required ]] && ! jq -e '.results | any(.packages | length > 0)' "$report" >/dev/null; then
    echo "OSV-Scanner returned no package inventory." >&2
    return 2
  fi

  jq -r '.results[] | .source.path as $source | .packages[] |
    select((.vulnerabilities | length) > 0) |
    "\($source): \(.package.name)@\(.package.version) \([.vulnerabilities[].id] | join(", "))"' "$report" || return 2

  if [[ "$status" != 0 ]] || jq -e 'any(.results[].packages[]; (.vulnerabilities | length) > 0)' "$report" >/dev/null; then
    echo "Security scan found advisories." >&2
    return 1
  fi
}

scan_runtime() {
  local product="$1" version="$2" report="$scratch/nvd.json"
  if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Expected an exact version for $product." >&2
    return 2
  fi
  echo "Checking $product $version against NVD"
  curl --fail --silent --show-error --max-time 30 --retry 2 --get \
    --data-urlencode "cpeName=cpe:2.3:a:$product:$version:*:*:*:*:*:*:*" \
    --data-urlencode isVulnerable \
    --output "$report" https://services.nvd.nist.gov/rest/json/cves/2.0 || return 2
  jq -e '(.totalResults | type == "number") and (.vulnerabilities | type == "array")' "$report" >/dev/null || return 2
  jq -r '.vulnerabilities[].cve.id' "$report" || return 2
  jq -e '.totalResults == 0' "$report" >/dev/null
}

scan_source() {
  require_tools curl
  local manifest go_version
  go_version=$(awk '$1 == "toolchain" && $2 ~ /^go[0-9]+\.[0-9]+\.[0-9]+$/ {print substr($2, 3)}' launcher/go.mod)
  if [[ ! "$go_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Expected an exact toolchain version in launcher/go.mod." >&2
    return 2
  fi
  printf 'ScanGoModVersion = true\nGoVersionOverride = "%s"\n' "$go_version" > "$scratch/source.toml"

  local status=0
  for manifest in worker/Cargo.lock frontend/yarn.lock launcher/go.mod; do
    scan required source --config "$scratch/source.toml" -L "$manifest" || status=1
  done
  local node_version rust_version rust_versions
  node_version=$(cat frontend/.node-version)
  scan_runtime nodejs:node.js "$node_version" || status=1

  rust_versions=$(
    {
      sed -n 's/.*rust-bin.stable."\([0-9.]*\)".*/\1/p' flake.nix
      sed -n 's/^ARG RUST_VERSION=\([0-9.]*\)$/\1/p' worker/api/Dockerfile worker/crawler/Dockerfile
    } | sort -u
  )
  while IFS= read -r rust_version; do
    scan_runtime rust-lang:rust "$rust_version" || status=1
  done <<< "$rust_versions"
  return "$status"
}

scan_archive() {
  local archive="$1" report="$scratch/trivy.json" status=0

  # OSV may inventory application artifacts while skipping the image's OS packages.
  trivy --config "$scratch/trivy.yaml" image --input "$archive" \
    --scanners vuln --pkg-types os,library --ignorefile '' --ignore-unfixed=false --list-all-pkgs \
    --severity UNKNOWN,LOW,MEDIUM,HIGH,CRITICAL \
    --format json --output "$report" --no-progress || return 2
  if ! jq -e 'any(.Results[]?; .Class == "os-pkgs" and (.Packages | length > 0))' "$report" >/dev/null; then
    echo "Trivy returned no OS package inventory." >&2
    return 2
  fi
  jq -r '.Results[] | .Target as $target | (.Vulnerabilities // [])[] |
    "\($target): \(.PkgName)@\(.InstalledVersion) \(.VulnerabilityID)"' "$report" || return 2
  if jq -e 'any(.Results[]; (.Vulnerabilities | length) > 0)' "$report" >/dev/null; then
    echo "Trivy found image package advisories." >&2
    status=1
  fi
  if jq -e '.Metadata.OS.Family == "oracle"' "$report" >/dev/null; then
    # OSV does not support Oracle RPMs. Trivy has already verified the RPM inventory.
    scan optional image --config "$scratch/image.toml" --archive "$archive" --allow-no-lockfiles || status=1
  else
    scan required image --config "$scratch/image.toml" --archive "$archive" || status=1
  fi
  return "$status"
}

scan_images() {
  require_tools docker trivy
  local image dockerfile stage status=0 archive="$scratch/image.tar"
  local compose_config="$scratch/compose.json"
  local compose_files=(--profile '*' -f docker-compose.yml -f docker-compose.local.yml)
  local buildx_args=()
  if [[ -n "${POSTAL_OSV_EXPORT_BUILDER:-}" ]]; then
    buildx_args+=(--builder "$POSTAL_OSV_EXPORT_BUILDER")
  fi

  docker compose "${compose_files[@]}" config --format json > "$compose_config"
  if ! jq -e '.services | length > 0 and all(.[]; (.image | type == "string" and length > 0))' "$compose_config" >/dev/null; then
    echo "Docker Compose must declare an image name for each service." >&2
    return 2
  fi

  if jq -e 'any(.services[]; .build != null)' "$compose_config" >/dev/null; then
    docker compose "${compose_files[@]}" build --pull --no-cache
  fi
  jq -r '[.services[] | select(.build == null) | .image] | unique[]' "$compose_config" > "$scratch/pull-images"
  while IFS= read -r image; do
    docker pull "$image"
  done < "$scratch/pull-images"
  jq -r '[.services[].image] | unique[]' "$compose_config" > "$scratch/sorted-images"
  while IFS= read -r image; do
    echo "Scanning Compose image $image"
    docker save --output "$archive" "$image"
    scan_archive "$archive" || status=1
  done < "$scratch/sorted-images"

  for dockerfile in worker/api/Dockerfile worker/crawler/Dockerfile; do
    for stage in rust-base runtime-base builder runtime; do
      echo "Scanning $dockerfile ($stage)"
      docker buildx build "${buildx_args[@]}" --pull --output "type=docker,dest=$archive" \
        --file "$dockerfile" --target "$stage" .
      scan_archive "$archive" || status=1
    done
  done
  return "$status"
}

case "$mode" in
  all) scan_source; scan_images ;;
  source) scan_source ;;
  images) scan_images ;;
esac
