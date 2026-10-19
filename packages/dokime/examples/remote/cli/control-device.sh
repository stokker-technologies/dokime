#!/bin/sh
# Dokime — SPDX-License-Identifier: Apache-2.0
set -eu

# Connection Parameters
ENDPOINT="https://127.0.0.1:8766"
CA_CERTIFICATE="server.pem"
KEY_FILE="remote.key"
ENV_FILE=""

# Response Files
RESPONSE_DIRECTORY=""

# Connect With The Explicit Parameters
cli() {
    set -- "$@" --endpoint "$ENDPOINT"
    if [ -n "$CA_CERTIFICATE" ]; then set -- "$@" --ca "$CA_CERTIFICATE"; fi
    if [ -n "$KEY_FILE" ]; then set -- "$@" --key-file "$KEY_FILE"; fi
    if [ -n "$ENV_FILE" ]; then set -- "$@" --env-file "$ENV_FILE"; fi
    dokime "$@"
}

# Reserve All Response Files Before Device Work
if [ -n "$RESPONSE_DIRECTORY" ]; then mkdir "$RESPONSE_DIRECTORY"; fi
trap 'printf "Stopped Waiting; Read Device State Before Resending An Unconfirmed Command\n" >&2; exit 130' INT TERM

# Discover Device Operations
cli operation list

# Read, Power On And Observe
call() {
    if [ -n "$RESPONSE_DIRECTORY" ]; then
        cli operation call "$1" --json "$2" --output "$RESPONSE_DIRECTORY/$3.json"
    else
        cli operation call "$1" --json "$2"
    fi
}
call obc.device.read '{}' before
call obc.device.set_power '{"enabled":true}' power-on
call obc.device.read '{}' running

# Power Off And Verify
call obc.device.set_power '{"enabled":false}' power-off
call obc.device.read '{}' after
