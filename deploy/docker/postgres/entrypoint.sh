#!/bin/sh
set -eu

if [ "${1:-}" = postgres ] &&
    [ -s "$PGDATA/PG_VERSION" ] &&
    [ ! -f "$PGDATA/.postal-alpine-volume" ]; then
    printf '%s\n' 'This PostgreSQL volume was not initialized by the Alpine image. Restore a logical backup into a new volume before starting it. See CONTRIBUTING.md.' >&2
    exit 1
fi

exec /usr/local/bin/docker-entrypoint.sh "$@"
