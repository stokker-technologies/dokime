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

# Case And Execution Parameters
$TEMPLATE = 'obc.checks.pulse'
$CYCLE_LABEL = 'Remote Pulse Demonstration'
$SAMPLE_COUNT = 8
$EXECUTION_COUNT = 2
$WAIT_SECONDS = 60

# Observation Filters
$RECORD_TYPES = @('obc.run.*')
$STREAMS = @('obc.pulse')

# Evidence Files
$OUTPUT_DIRECTORY = 'test-evidence'
$OVERWRITE = $false

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

# PowerShell Kills A Captured Native Command On Ctrl+C, Losing Its Recovery,
# So Recovery-Armed Commands Run Uncaptured And Hand Identities Over A File
$IdFile = [System.IO.Path]::GetTempFileName()
function Read-Ids {
    if (-not (Test-Path -LiteralPath $script:IdFile)) { return @() }
    @(Get-Content -LiteralPath $script:IdFile | Where-Object { $_ })
}

$CycleId = ''
$ExecutionId = ''
$Result = 0
$Completed = $false
$OperationalFailure = $false

# Preserve Evidence Before Starting Any Device Work
function Assert-Destination([string]$Path) {
    if ((Test-Path -LiteralPath $Path) -and -not $OVERWRITE) { throw "Evidence File Exists: $Path" }
}
$Export = @()
if ($OVERWRITE) { $Export = @('--overwrite') }

try {
    if ($WAIT_SECONDS -le 0) { throw 'WAIT_SECONDS Must Be Positive' }
    New-Item -ItemType Directory -Force -Path $OUTPUT_DIRECTORY | Out-Null
    $Recovery = Join-Path $OUTPUT_DIRECTORY 'recovery'
    New-Item -ItemType Directory -Force -Path $Recovery | Out-Null

    if ($EXECUTION_COUNT -lt 1) { throw 'EXECUTION_COUNT Must Be Positive' }

    # Materialize Cases Before Constructing Independent Attempts
    $Parameters = @{ sample_count = $SAMPLE_COUNT } | ConvertTo-Json -Compress
    $Cases = @(Invoke-Dokime -Command @('plan', 'materialize', '--template', $TEMPLATE,
        '--parameters-json', $Parameters, '--id-only'))
    if ($Cases.Count -eq 0) { throw 'No Cases Were Materialized' }
    foreach ($Index in 1..($Cases.Count * $EXECUTION_COUNT)) {
        Assert-Destination (Join-Path $OUTPUT_DIRECTORY "execution-$Index.zip")
    }
    Assert-Destination (Join-Path $OUTPUT_DIRECTORY 'cycle.zip')

    $Created = @(Invoke-Dokime -Command @('cycle', 'create', '--label', $CYCLE_LABEL, '--id-only'))
    if ($Created.Count -ne 1) { throw 'Cycle Creation Did Not Return One Identity' }
    $CycleId = [guid]::Parse([string]$Created[0]).ToString()
    [Console]::Out.WriteLine("Cycle: $CycleId")
    $RecoveryArguments = @('--cancel-on-interrupt', '--owned-cycle', $CycleId, '--recovery-directory', $Recovery)

    # Append Every Attempt Without Starting
    $Executions = @()
    foreach ($Attempt in 1..$EXECUTION_COUNT) {
        foreach ($CaseId in $Cases) {
            Invoke-Dokime -Command (@('cycle', 'append', $CycleId, '--case-id', [string]$CaseId, '--id-only', '--id-file', $IdFile) + $RecoveryArguments)
            $Created = @(Read-Ids)
            if ($Created.Count -ne 1) { throw 'Append Did Not Return One Execution Identity' }
            $ExecutionId = [guid]::Parse([string]$Created[0]).ToString()
            [Console]::Out.WriteLine("Execution: $ExecutionId")
            $Executions += $ExecutionId
        }
    }

    # Run Sequentially And Drain Requested Listeners Before Export
    $Index = 0
    foreach ($ExecutionId in $Executions) {
        $Index++
        $Run = @('execution', 'run', $ExecutionId, '--records', '--samples', '--wait-seconds', "$WAIT_SECONDS",
            '--cancel-on-interrupt', '--owned-cycle', $CycleId, '--recovery-directory', $Recovery)
        foreach ($RecordType in $RECORD_TYPES) { $Run += @('--record-type', $RecordType) }
        foreach ($Stream in $STREAMS) { $Run += @('--stream', $Stream) }
        Invoke-Dokime -Command $Run -AllowSavedFailure
        Invoke-Dokime -Command (@('execution', 'export', $ExecutionId, '--output', (Join-Path $OUTPUT_DIRECTORY "execution-$Index.zip")) + $Export + $RecoveryArguments)
    }

    # Seal The Container And Save Its Complete Bundle
    Invoke-Dokime -Command @('cycle', 'seal', $CycleId)
    Invoke-Dokime -Command (@('cycle', 'export', $CycleId, '--output', (Join-Path $OUTPUT_DIRECTORY 'cycle.zip')) + $Export)
    $Completed = $true
} catch {
    $OperationalFailure = $true
    [Console]::Error.WriteLine("Workflow Incomplete: $_. Inspect Cycle: $CycleId Execution: $ExecutionId Before Resubmitting")
    exit 3
} finally {
    # Cover Shell Interruption Between Native Commands
    if (-not $Completed -and -not $OperationalFailure -and $CycleId) {
        try {
            # An Append Confirmed Just Before Ctrl+C Left Its Identity In The File
            $Confirmed = @(Read-Ids)
            if (-not $ExecutionId -and $Confirmed.Count -eq 1) {
                $ExecutionId = [guid]::Parse([string]$Confirmed[0]).ToString()
            }
            [Console]::Error.WriteLine("Cancellation Requested For Script-Owned Cycle: $CycleId")
            if ($ExecutionId) {
                try { Invoke-Dokime -Command @('execution', 'stop', $ExecutionId) }
                catch { [Console]::Error.WriteLine('Stop Was Not Confirmed; Resolving Through The Owned Cycle Seal') }
            }
            Invoke-Dokime -Command @('cycle', 'seal', $CycleId)
            if ($ExecutionId) {
                Invoke-Dokime -Command @('execution', 'wait', $ExecutionId) -AllowSavedFailure
                $Destination = Join-Path $Recovery "execution-$ExecutionId.zip"
                if (-not (Test-Path -LiteralPath $Destination)) {
                    Invoke-Dokime -Command @('execution', 'export', $ExecutionId, '--output', $Destination)
                }
            }
            $Destination = Join-Path $Recovery "cycle-$CycleId.zip"
            if (-not (Test-Path -LiteralPath $Destination)) {
                Invoke-Dokime -Command @('cycle', 'export', $CycleId, '--output', $Destination)
            }
        } catch {
            [Console]::Error.WriteLine("Recovery Was Not Confirmed: $_. Retain Cycle: $CycleId Execution: $ExecutionId")
        }
        Remove-Item -LiteralPath $IdFile -ErrorAction SilentlyContinue
        exit 130
    }
    Remove-Item -LiteralPath $IdFile -ErrorAction SilentlyContinue
}
exit $Result
