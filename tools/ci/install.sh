#!/bin/sh
# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

# Prepare Native Linux CI Tools.

# === Imports === #

# === Constants === #

# === Functions === #

# === Entry Point === #
set -eu
. tools/ci/install-mise.sh
if [ -n "${GITHUB_PATH:-}" ]; then
    printf '%s\n' "$PWD/.cache/bootstrap/bin" >> "$GITHUB_PATH"
fi
mise trust mise.toml
mise install --locked python uv
mise run --skip-tools ci:bootstrap
mise install --locked
git diff --exit-code -- mise.lock
mise exec -- rustc --version
mise exec -- cargo --version
mise exec -- python --version
mise exec -- uv --version
mise exec -- node --version
mise exec -- pnpm --version

mise run sync
