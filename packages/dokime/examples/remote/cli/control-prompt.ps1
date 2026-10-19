#Requires -Version 7.3
# Dokime - SPDX-License-Identifier: Apache-2.0
# Pass Connection Flags Such As --endpoint, --ca, --key-file Or --env-file.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandArgumentPassing = 'Standard'
$PSNativeCommandUseErrorActionPreference = $false

$Connection = @($args)
$Dokime = if ($env:DOKIME_EXECUTABLE) { $env:DOKIME_EXECUTABLE } else { 'dokime' }
$Directory = if ($env:DOKIME_OUTPUT_DIRECTORY) { $env:DOKIME_OUTPUT_DIRECTORY } else { 'prompt-evidence' }
$CycleId = ''
$ExecutionId = ''
$PromptId = ''
$RequestKey = ''
$Completed = $false
$Result = 0

function Invoke-Dokime {
    param([string[]]$Command, [switch]$AllowSavedFailure)
    $ConnectionArguments = $script:Connection
    & $script:Dokime @Command @ConnectionArguments
    $Status = $LASTEXITCODE
    if ($AllowSavedFailure -and $Status -eq 1) {
        $script:Result = 1
        return
    }
    if ($Status -ne 0) {
        throw "Dokime Returned Exit Code $Status; No Answer Or Submission Was Retried"
    }
}

try {
    if (Test-Path -LiteralPath $Directory) {
        throw 'Choose A New Output Directory; Existing Evidence Is Preserved'
    }
    New-Item -ItemType Directory -Path $Directory | Out-Null
    $Recovery = Join-Path $Directory 'recovery'
    New-Item -ItemType Directory -Path $Recovery | Out-Null

    $Created = @(Invoke-Dokime -Command @('cycle', 'create', '--label', 'Remote Confirmation Demonstration', '--id-only'))
    if ($Created.Count -ne 1) { throw 'Cycle Creation Did Not Return One Identity; Inspect The Host Before Creating Another' }
    $CycleId = [guid]::Parse([string]$Created[0]).ToString()
    [Console]::Out.WriteLine("Cycle: $CycleId")
    $Created = @(Invoke-Dokime -Command @('cycle', 'append', $CycleId, '--template', 'obc.checks.pulse',
        '--parameters-json', '{"require_confirmation":true}', '--id-only'))
    if ($Created.Count -ne 1) { throw "Append Did Not Return One Execution; Inspect Cycle $CycleId Before Appending Again" }
    $ExecutionId = [guid]::Parse([string]$Created[0]).ToString()
    $RequestKey = "remote-$ExecutionId"
    [Console]::Out.WriteLine("Execution: $ExecutionId Request Key: $RequestKey Mode: automatic")
    Invoke-Dokime -Command @('execution', 'run', $ExecutionId, '--request-key', $RequestKey)

    $Next = @('execution', 'prompt', 'next', $ExecutionId, '--wait-seconds', '60', '--id-only',
        '--cancel-on-interrupt', '--owned-cycle', $CycleId, '--recovery-directory', $Recovery)
    $Pending = @(Invoke-Dokime -Command $Next)
    if ($Pending.Count -ne 1) { throw "Prompt Discovery Did Not Return One Identity; Inspect Execution $ExecutionId" }
    $PromptId = [guid]::Parse([string]$Pending[0]).ToString()
    [Console]::Out.WriteLine("Prompt: $PromptId Execution: $ExecutionId")
    Invoke-Dokime -Command @('execution', 'prompt', 'show', $ExecutionId, $PromptId)
    Invoke-Dokime -Command @('execution', 'prompt', 'answer', $ExecutionId, $PromptId, '--action', 'yes')

    $Wait = @('execution', 'wait', $ExecutionId, '--wait-seconds', '120', '--cancel-on-interrupt',
        '--owned-cycle', $CycleId, '--recovery-directory', $Recovery)
    Invoke-Dokime -Command $Wait -AllowSavedFailure
    $Destination = Join-Path $Directory 'execution.zip'
    Invoke-Dokime -Command @('execution', 'export', $ExecutionId, '--output', $Destination)
    Invoke-Dokime -Command @('cycle', 'seal', $CycleId)
    $Destination = Join-Path $Directory 'cycle.zip'
    Invoke-Dokime -Command @('cycle', 'export', $CycleId, '--output', $Destination)
    $Completed = $true
}
catch {
    [Console]::Error.WriteLine($_.Exception.Message)
    $Result = 3
}
finally {
    if (-not $Completed) {
        [Console]::Error.WriteLine("Recovery Cycle: $CycleId Execution: $ExecutionId Prompt: $PromptId Request Key: $RequestKey")
        [Console]::Error.WriteLine('A Failed Connection Does Not Confirm Cancellation Or An Answer. Inspect The Original Identities Before Resubmitting')
    }
}
exit $Result
