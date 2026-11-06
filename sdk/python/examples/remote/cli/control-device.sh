#!/bin/sh
# Dokime — SPDX-License-Identifier: Apache-2.0
set -eu

# Connection Parameters
BACKEND_IDENTIFIER="lab"

# Select The Imported Profile Once; Each Call Below Shows Its Remote Command.
set -- --backend "$BACKEND_IDENTIFIER"

trap 'printf "Stopped Waiting; Read Device State Before Resending An Unconfirmed Command\n" >&2; exit 130' INT TERM

# Discover The Operation IDs Offered By The Installed Plugin.
dokime operation list "$@"

# Read The Manual Device Before Changing Power.
dokime operation call obc.device.read --json '{}' --output "before.json" "$@"

# Enable The Manual Device.
dokime operation call obc.device.set_power --json '{"enabled":true}' --output "power-on.json" "$@"

# Read Back The Powered State: 230 V RMS.
dokime operation call obc.device.read --json '{}' --output "running.json" "$@"

# Disable The Manual Device.
dokime operation call obc.device.set_power --json '{"enabled":false}' --output "power-off.json" "$@"

# Read Back The Powered-Off State: Zero Volts.
dokime operation call obc.device.read --json '{}' --output "after.json" "$@"
