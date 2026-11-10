#!/bin/sh
# Dokime — SPDX-License-Identifier: Apache-2.0
set -eu

# Connection Parameters
PAIRING_FILE="../lab.pair"

# Select The Pairing File; Each Call Below Shows Its Remote Command.
set -- --remote "$PAIRING_FILE"

trap 'printf "Stopped Waiting; Read Device State Before Resending An Unconfirmed Command\n" >&2; exit 130' INT TERM

# Discover the Operation IDs Offered by the Installed Plugin.
dokime operation list "$@"

# Read the Manual Device Before Changing Power.
dokime operation call obc.device.read --json '{}' --output "before.json" "$@"

# Enable the Manual Device.
dokime operation call obc.device.set_power --json '{"enabled":true}' --output "power-on.json" "$@"

# Read Back the Powered State: 230 V RMS.
dokime operation call obc.device.read --json '{}' --output "running.json" "$@"

# Disable the Manual Device.
dokime operation call obc.device.set_power --json '{"enabled":false}' --output "power-off.json" "$@"

# Read Back the Powered-Off State: Zero Volts.
dokime operation call obc.device.read --json '{}' --output "after.json" "$@"
