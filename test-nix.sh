#!/usr/bin/env bash

set -euo pipefail

nix flake check --no-build

nix build ".#packages.x86_64-linux.default"

cfg=$(nix build --no-link --print-out-paths \
    ".#packages.x86_64-linux.omnilint-config")
OMNILINT_CONFIG=$cfg

cat "$OMNILINT_CONFIG"

./result/bin/omnilint inventory

nix develop -c cargo test --features test-linter-tools
