#Requires -Version 7.3
# Dokime - SPDX-License-Identifier: Apache-2.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandArgumentPassing = 'Standard'
$PSNativeCommandUseErrorActionPreference = $true

# Connection Parameters
$BACKEND_IDENTIFIER = 'lab'

# Connect with the Explicit Parameters
$Connection = @('--backend', $BACKEND_IDENTIFIER)

# Discover the Operation IDs Offered by the Installed Plugin.
dokime operation list @Connection

# Read the Manual Device Before Changing Power.
dokime operation call obc.device.read --json '{}' --output 'before.json' @Connection

# Enable the Manual Device.
dokime operation call obc.device.set_power --json '{"enabled":true}' --output 'power-on.json' @Connection

# Read Back the Powered State: 230 V RMS.
dokime operation call obc.device.read --json '{}' --output 'running.json' @Connection

# Disable the Manual Device.
dokime operation call obc.device.set_power --json '{"enabled":false}' --output 'power-off.json' @Connection

# Read Back the Powered-Off State: Zero Volts.
dokime operation call obc.device.read --json '{}' --output 'after.json' @Connection
