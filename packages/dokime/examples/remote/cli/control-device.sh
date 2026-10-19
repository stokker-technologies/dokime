#!/bin/sh
# Dokime — SPDX-License-Identifier: Apache-2.0
# Pass Connection Flags Such As --endpoint, --ca, --key-file Or --env-file.
set -eu

dokime=${DOKIME_EXECUTABLE:-dokime}
responses=${DOKIME_RESPONSE_DIRECTORY:-}

if [ -n "$responses" ]; then
    mkdir -p "$responses"
    for name in before power-on running power-off after; do
        destination=$responses/$name.json
        if [ -e "$destination" ] || [ -L "$destination" ]; then
            printf 'Response File Exists: %s; Choose An Empty Response Directory\n' "$destination" >&2
            exit 2
        fi
    done
fi

trap 'printf "Stopped Waiting; A Dispatched Command May Have Taken Effect. Read Device State Before Resending\n" >&2; exit 130' INT TERM

# Keep File Policy In One Place; Every Invocation Still Uses The Native CLI.
call() {
    operation=$1
    request=$2
    filename=$3
    shift 3
    if [ -n "$responses" ]; then
        "$dokime" operation call "$operation" --json "$request" --output "$responses/$filename.json" "$@"
    else
        "$dokime" operation call "$operation" --json "$request" "$@"
    fi
}

"$dokime" operation list "$@"
call obc.device.read '{}' before "$@"
call obc.device.set_power '{"enabled":true}' power-on "$@"
call obc.device.read '{}' running "$@"
call obc.device.set_power '{"enabled":false}' power-off "$@"
call obc.device.read '{}' after "$@"
