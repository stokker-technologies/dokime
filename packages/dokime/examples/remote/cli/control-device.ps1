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

function Invoke-Dokime {
    param([string[]]$Command, [switch]$AllowSavedFailure)
    dokime @Command @script:Connection
    $Status = $LASTEXITCODE
    if ($AllowSavedFailure -and $Status -eq 1) { $script:Result = 1; return }
    if ($Status -ne 0) { throw "Dokime Returned Exit Code $Status; No Mutation Was Retried" }
}

# Reserve All Response Files Before Device Work
if ($RESPONSE_DIRECTORY) {
    if (Test-Path -LiteralPath $RESPONSE_DIRECTORY) { throw 'Response Directory Exists; Existing Data Is Preserved' }
    New-Item -ItemType Directory -Path $RESPONSE_DIRECTORY | Out-Null
}

# Discover Device Operations
Invoke-Dokime -Command @('operation', 'list')

# Read, Power On, Observe, Power Off And Verify
try {
    $Calls = @(
        @('obc.device.read', '{}', 'before'),
        @('obc.device.set_power', '{"enabled":true}', 'power-on'),
        @('obc.device.read', '{}', 'running'),
        @('obc.device.set_power', '{"enabled":false}', 'power-off'),
        @('obc.device.read', '{}', 'after')
    )
    foreach ($Call in $Calls) {
        $Command = @('operation', 'call', $Call[0], '--json', $Call[1])
        if ($RESPONSE_DIRECTORY) { $Command += @('--output', (Join-Path $RESPONSE_DIRECTORY "$($Call[2]).json")) }
        Invoke-Dokime -Command $Command
    }
} catch {
    [Console]::Error.WriteLine("Device Workflow Incomplete: $_. Read Device State Before Resending")
    exit 3
}
