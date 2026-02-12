#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "${ROOT_DIR}" ]]; then
  echo "error: not inside git repository" >&2
  exit 1
fi

ALIAS_LINE="alias gr='${ROOT_DIR}/scripts/gr'"

if [[ "${1:-}" == "--write-zshrc" ]]; then
  ZSHRC_PATH="${HOME}/.zshrc"
  if rg -q "alias gr='${ROOT_DIR}/scripts/gr'" "${ZSHRC_PATH}" 2>/dev/null; then
    echo "gr alias already exists in ${ZSHRC_PATH}"
    exit 0
  fi
  {
    echo
    echo "# Postal Converter JA PR helper"
    echo "${ALIAS_LINE}"
  } >> "${ZSHRC_PATH}"
  echo "added gr alias to ${ZSHRC_PATH}"
  echo "run: source ${ZSHRC_PATH}"
  exit 0
fi

echo "Run this command to enable gr in current shell:"
echo "${ALIAS_LINE}"
