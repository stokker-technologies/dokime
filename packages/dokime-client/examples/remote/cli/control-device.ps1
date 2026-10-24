#Requires -Version 7.3
# Dokime - SPDX-License-Identifier: Apache-2.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandArgumentPassing = 'Standard'
$PSNativeCommandUseErrorActionPreference = $false

# Connection Parameters
$ENDPOINT = 'https://127.0.0.1:8766'
$CA_CERTIFICATE = 'server.pem'
$KEY_FILE = 'remote.key'
$ENV_FILE = ''

# Response Files
$RESPONSE_DIRECTORY = ''

# Connect With The Explicit Parameters
$Connection = @('--endpoint', $ENDPOINT)
if ($CA_CERTIFICATE) { $Connection += @('--ca', $CA_CERTIFICATE) }
if ($KEY_FILE) { $Connection += @('--key-file', $KEY_FILE) }
if ($ENV_FILE) { $Connection += @('--env-file', $ENV_FILE) }

# Reserve Reply Files Before Connecting; Manual Control Is Not Execution Evidence.
if ($RESPONSE_DIRECTORY) {
    if (Test-Path -LiteralPath $RESPONSE_DIRECTORY) { throw 'Response Directory Exists; Existing Data Is Preserved' }
    New-Item -ItemType Directory -Path $RESPONSE_DIRECTORY | Out-Null
}

try {
    # Discover The Operation IDs Offered By The Installed Plugin.
    dokime operation list @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }

    # Read The Manual Device Before Changing Power.
    $Output = @()
    if ($RESPONSE_DIRECTORY) { $Output = @('--output', (Join-Path $RESPONSE_DIRECTORY 'before.json')) }
    dokime operation call obc.device.read --json '{}' @Output @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }

    # Enable The Manual Device.
    $Output = @()
    if ($RESPONSE_DIRECTORY) { $Output = @('--output', (Join-Path $RESPONSE_DIRECTORY 'power-on.json')) }
    dokime operation call obc.device.set_power --json '{"enabled":true}' @Output @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }

    # Read Back The Powered State: 230 V RMS.
    $Output = @()
    if ($RESPONSE_DIRECTORY) { $Output = @('--output', (Join-Path $RESPONSE_DIRECTORY 'running.json')) }
    dokime operation call obc.device.read --json '{}' @Output @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }

    # Disable The Manual Device.
    $Output = @()
    if ($RESPONSE_DIRECTORY) { $Output = @('--output', (Join-Path $RESPONSE_DIRECTORY 'power-off.json')) }
    dokime operation call obc.device.set_power --json '{"enabled":false}' @Output @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }

    # Read Back The Powered-Off State: Zero Volts.
    $Output = @()
    if ($RESPONSE_DIRECTORY) { $Output = @('--output', (Join-Path $RESPONSE_DIRECTORY 'after.json')) }
    dokime operation call obc.device.read --json '{}' @Output @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }

} catch {
    [Console]::Error.WriteLine("Device Workflow Incomplete: $_. Read Device State Before Resending")
    exit 3
}
