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
role=${DOKIME_CI_ROLE:-check}
case "$role" in
    build|test|coverage) gui=--gui-packages ;;
    check|contracts|catalog|typescript|publish) gui= ;;
    *) printf '%s\n' "Unknown CI role: $role" >&2; exit 1 ;;
esac
mise exec -- python tools/python/src/dokime_tools/ci/bootstrap.py $gui --role "$role"
