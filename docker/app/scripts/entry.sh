#!/usr/bin/env ash

set -euo pipefail

wait_db() {
  local retries=5
  while [[ "$retries" -gt "0" ]]; do
    set +e
    tools/diesel migration list \
      --database-url "$PG_DB_URL" \
      >&2
    local status="$?"
    set -e

    if [[ "$status" -eq "0" ]]; then
      echo "Database available." >&2
      return
    fi

    retries=$(( retries - 1 ))
    echo "Database is not available. Sleeping..." >&2
    sleep 10s
  done

  exit 1
}

main() {
  export SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
  export SSL_CERT_DIR=/etc/ssl/certs

  echo "Waiting for DB" >&2
  wait_db

  echo "Running migrations" >&2
  tools/diesel setup \
    --database-url "$PG_DB_URL" \
    >&2
  tools/diesel migration run \
    --database-url "$PG_DB_URL" \
    >&2

  echo "Running service" >&2
  ST_LOG=prod \
    ST_INDEXER_URL="$ST_INDEXER_URL" \
    ST_IMPORT_URL="$ST_IMPORT_URL" \
    ST_SCRAPER_URL="$ST_SCRAPER_URL" \
    PG_DB_URL="$PG_DB_URL" \
    exec ./satelit-scheduler
}

main "$@"
