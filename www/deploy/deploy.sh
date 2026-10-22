#!/usr/bin/env bash
# Build the Leptos site and rsync it to the nginx server.
# Usage: DEPLOY_HOST=user@kreaker.net ./deploy/deploy.sh
set -euo pipefail

: "${DEPLOY_HOST:?set DEPLOY_HOST, e.g. user@kreaker.net}"
DEPLOY_PATH="${DEPLOY_PATH:-/var/www/kree}"

cd "$(dirname "$0")/.."

trunk build --release
rsync -avz --delete dist/ "${DEPLOY_HOST}:${DEPLOY_PATH}/"
echo "Deployed to ${DEPLOY_HOST}:${DEPLOY_PATH}"
