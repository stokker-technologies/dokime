#Requires -Version 7.3
# Dokime - SPDX-License-Identifier: Apache-2.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandArgumentPassing = 'Standard'
$PSNativeCommandUseErrorActionPreference = $true

# Connection Parameters
$ENDPOINT = 'https://127.0.0.1:8766'
$CA_CERTIFICATE = 'server.pem'
$KEY_FILE = 'remote.key'
$ENV_FILE = ''

# Connect With The Explicit Parameters
$Connection = @('--endpoint', $ENDPOINT)
if ($CA_CERTIFICATE) { $Connection += @('--ca', $CA_CERTIFICATE) }
if ($KEY_FILE) { $Connection += @('--key-file', $KEY_FILE) }
if ($ENV_FILE) { $Connection += @('--env-file', $ENV_FILE) }

# Discover The Operation IDs Offered By The Installed Plugin.
dokime operation list @Connection

# Read The Manual Device Before Changing Power.
dokime operation call obc.device.read --json '{}' --output 'before.json' @Connection

# Enable The Manual Device.
dokime operation call obc.device.set_power --json '{"enabled":true}' --output 'power-on.json' @Connection

# Read Back The Powered State: 230 V RMS.
dokime operation call obc.device.read --json '{}' --output 'running.json' @Connection

# Disable The Manual Device.
dokime operation call obc.device.set_power --json '{"enabled":false}' --output 'power-off.json' @Connection

# Read Back The Powered-Off State: Zero Volts.
dokime operation call obc.device.read --json '{}' --output 'after.json' @Connection
