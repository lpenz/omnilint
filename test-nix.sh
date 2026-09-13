#!/usr/bin/env bash

set -euo pipefail

system="${1:-x86_64-linux}"

nix flake check --no-build

nix build ".#packages.${system}.default"

cfg=$(nix build --no-link --print-out-paths \
    ".#packages.${system}.omnilint-config")
OMNILINT_CONFIG=$cfg

cat "$OMNILINT_CONFIG"

./result/bin/omnilint inventory

nix develop -c cargo test --features test-linter-tools
