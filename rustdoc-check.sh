#!/bin/bash

set -eu -o pipefail

cd "$(git rev-parse --show-toplevel)"

# Workspace members are represented in a topological sort order that means all packages will run
# after their dependencies.
members=(
    "dymium-term --lib"
    "dymium-term --bin verify-caps"
)

for m in "${members[@]}"; do
    cargo +nightly rustdoc -p $m -- -D rustdoc::all -A unknown_lints --document-private-items -Z unstable-options --check
done
