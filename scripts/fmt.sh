#!/usr/bin/env bash

set -euo pipefail

# we're using unstable config so we need nightly compiler
# pass --check to check format on CI
cargo +nightly fmt --all -- "$@"
