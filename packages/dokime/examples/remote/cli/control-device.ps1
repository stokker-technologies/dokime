#Requires -Version 7.3
# Dokime - SPDX-License-Identifier: Apache-2.0
# Pass Connection Flags Such As --endpoint, --ca, --key-file Or --env-file.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandArgumentPassing = 'Standard'
$PSNativeCommandUseErrorActionPreference = $false

$Connection = @($args)
$Dokime = if ($env:DOKIME_EXECUTABLE) { $env:DOKIME_EXECUTABLE } else { 'dokime' }
$Responses = $env:DOKIME_RESPONSE_DIRECTORY
$Completed = $false
$Result = 0

function Invoke-Dokime {
    param([string[]]$Command)
    $ConnectionArguments = $script:Connection
    & $script:Dokime @Command @ConnectionArguments
    if ($LASTEXITCODE -ne 0) {
        throw "Dokime Returned Exit Code $LASTEXITCODE; No Command Was Retried"
    }
}

try {
    if ($Responses) {
        if (Test-Path -LiteralPath $Responses) {
            throw 'Choose A New Response Directory; Existing Data Is Preserved'
        }
        New-Item -ItemType Directory -Path $Responses | Out-Null
    }

    Invoke-Dokime -Command @('operation', 'list')
    $Calls = @(
        @('obc.device.read', '{}', 'before'),
        @('obc.device.set_power', '{"enabled":true}', 'power-on'),
        @('obc.device.read', '{}', 'running'),
        @('obc.device.set_power', '{"enabled":false}', 'power-off'),
        @('obc.device.read', '{}', 'after')
    )
    foreach ($Call in $Calls) {
        $Command = @('operation', 'call', $Call[0], '--json', $Call[1])
        if ($Responses) {
            $Destination = Join-Path $Responses "$($Call[2]).json"
            $Command += @('--output', $Destination)
        }
        Invoke-Dokime -Command $Command
    }
    $Completed = $true
}
catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    $Result = 3
}
finally {
    if (-not $Completed) {
        [Console]::Error.WriteLine('Device Outcome May Be Unconfirmed. Read Device State Before Resending; No Command Was Retried')
    }
}
exit $Result
