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

# Select Secrets Once; Each Call Below Shows Its Remote Command.
set -- --endpoint "$ENDPOINT"
if [ -n "$CA_CERTIFICATE" ]; then set -- "$@" --ca "$CA_CERTIFICATE"; fi
if [ -n "$KEY_FILE" ]; then set -- "$@" --key-file "$KEY_FILE"; fi
if [ -n "$ENV_FILE" ]; then set -- "$@" --env-file "$ENV_FILE"; fi

# Reserve Reply Files Before Connecting; Manual Control Is Not Execution Evidence.
if [ -n "$RESPONSE_DIRECTORY" ]; then mkdir "$RESPONSE_DIRECTORY"; fi
trap 'printf "Stopped Waiting; Read Device State Before Resending An Unconfirmed Command\n" >&2; exit 130' INT TERM

# Discover The Operation IDs Offered By The Installed Plugin.
dokime operation list "$@"

# Read The Manual Device Before Changing Power.
if [ -n "$RESPONSE_DIRECTORY" ]; then
    dokime operation call obc.device.read --json '{}' --output "$RESPONSE_DIRECTORY/before.json" "$@"
else
    dokime operation call obc.device.read --json '{}' "$@"
fi

# Enable The Manual Device.
if [ -n "$RESPONSE_DIRECTORY" ]; then
    dokime operation call obc.device.set_power --json '{"enabled":true}' --output "$RESPONSE_DIRECTORY/power-on.json" "$@"
else
    dokime operation call obc.device.set_power --json '{"enabled":true}' "$@"
fi

# Read Back The Powered State: 230 V RMS.
if [ -n "$RESPONSE_DIRECTORY" ]; then
    dokime operation call obc.device.read --json '{}' --output "$RESPONSE_DIRECTORY/running.json" "$@"
else
    dokime operation call obc.device.read --json '{}' "$@"
fi

# Disable The Manual Device.
if [ -n "$RESPONSE_DIRECTORY" ]; then
    dokime operation call obc.device.set_power --json '{"enabled":false}' --output "$RESPONSE_DIRECTORY/power-off.json" "$@"
else
    dokime operation call obc.device.set_power --json '{"enabled":false}' "$@"
fi

# Read Back The Powered-Off State: Zero Volts.
if [ -n "$RESPONSE_DIRECTORY" ]; then
    dokime operation call obc.device.read --json '{}' --output "$RESPONSE_DIRECTORY/after.json" "$@"
else
    dokime operation call obc.device.read --json '{}' "$@"
fi
