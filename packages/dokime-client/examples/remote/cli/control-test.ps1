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
$RECORD_TYPES = 'obc.run.*'
$STREAMS = 'obc.pulse'

# Evidence Files
$OUTPUT_DIRECTORY = 'test-evidence'
$OVERWRITE = $false

# Connect With The Explicit Parameters
$Connection = @('--endpoint', $ENDPOINT)
if ($CA_CERTIFICATE) { $Connection += @('--ca', $CA_CERTIFICATE) }
if ($KEY_FILE) { $Connection += @('--key-file', $KEY_FILE) }
if ($ENV_FILE) { $Connection += @('--env-file', $ENV_FILE) }

# Uncaptured Commands Retain Native Ctrl+C Recovery; UUIDs Travel In A File.
$IdFile = [System.IO.Path]::GetTempFileName()
$CycleId = ''
$ExecutionId = ''
$Result = 0
$Completed = $false
$OperationalFailure = $false
$Export = @()
if ($OVERWRITE) { $Export = @('--overwrite') }

try {
    if ($WAIT_SECONDS -le 0) { throw 'WAIT_SECONDS Must Be Positive' }
    New-Item -ItemType Directory -Force -Path $OUTPUT_DIRECTORY | Out-Null
    $Recovery = Join-Path $OUTPUT_DIRECTORY 'recovery'
    New-Item -ItemType Directory -Force -Path $Recovery | Out-Null
    if ($EXECUTION_COUNT -notin @(1, 2)) { throw 'EXECUTION_COUNT Must Be 1 Or 2' }
    if ((Test-Path -LiteralPath (Join-Path $OUTPUT_DIRECTORY 'execution-1.zip')) -and -not $OVERWRITE) {
        throw 'Evidence File Exists: execution-1.zip'
    }
    if ($EXECUTION_COUNT -eq 2 -and (Test-Path -LiteralPath (Join-Path $OUTPUT_DIRECTORY 'execution-2.zip')) -and -not $OVERWRITE) {
        throw 'Evidence File Exists: execution-2.zip'
    }
    if ((Test-Path -LiteralPath (Join-Path $OUTPUT_DIRECTORY 'cycle.zip')) -and -not $OVERWRITE) {
        throw 'Evidence File Exists: cycle.zip'
    }

    # Materialize One Immutable Case; Its ID Is Reused By Independent Attempts.
    $Parameters = @{ sample_count = $SAMPLE_COUNT } | ConvertTo-Json -Compress
    $Cases = @(dokime plan materialize --template $TEMPLATE --parameters-json $Parameters --id-only @Connection)
    if ($LASTEXITCODE -ne 0) { throw "Materialize Returned Exit Code $LASTEXITCODE" }
    if ($Cases.Count -ne 1) { throw 'Expected One Case' }
    $CaseId = [guid]::Parse([string]$Cases[0]).ToString()
    [Console]::Out.WriteLine("Case: $CaseId")
    $Created = @(dokime cycle create --label $CYCLE_LABEL --id-only @Connection)
    if ($LASTEXITCODE -ne 0) { throw "Create Returned Exit Code $LASTEXITCODE" }
    if ($Created.Count -ne 1) { throw 'Cycle Creation Did Not Return One Identity' }
    $CycleId = [guid]::Parse([string]$Created[0]).ToString()
    [Console]::Out.WriteLine("Cycle: $CycleId")

    # Append Both Attempts Without Starting Hardware.
    dokime cycle append $CycleId --case-id $CaseId --id-only --id-file $IdFile --cancel-on-interrupt --owned-cycle $CycleId `
        --recovery-directory $Recovery @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }
    $Created = @(Get-Content -LiteralPath $IdFile)
    if ($Created.Count -ne 1) { throw 'Append Did Not Return One Execution Identity' }
    $FirstId = [guid]::Parse([string]$Created[0]).ToString()
    [Console]::Out.WriteLine("Execution: $FirstId")
    if ($EXECUTION_COUNT -eq 2) {
        dokime cycle append $CycleId --case-id $CaseId --id-only --id-file $IdFile --cancel-on-interrupt --owned-cycle $CycleId `
            --recovery-directory $Recovery @Connection
        if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }
        $Created = @(Get-Content -LiteralPath $IdFile)
        if ($Created.Count -ne 1) { throw 'Append Did Not Return One Execution Identity' }
        $SecondId = [guid]::Parse([string]$Created[0]).ToString()
        [Console]::Out.WriteLine("Execution: $SecondId")
    }

    # Run Attempt 1; The Command Waits For Its Saved Outcome And Listener Drain.
    $ExecutionId = $FirstId
    dokime execution run $FirstId --records --samples --record-type $RECORD_TYPES --stream $STREAMS --wait-seconds $WAIT_SECONDS `
        --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    if ($LASTEXITCODE -gt 1 -or $LASTEXITCODE -lt 0) { throw "Dokime Returned Exit Code $LASTEXITCODE" }
    if ($LASTEXITCODE -eq 1) { $Result = 1 }
    dokime execution export $FirstId --output (Join-Path $OUTPUT_DIRECTORY 'execution-1.zip') @Export --cancel-on-interrupt `
        --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }

    if ($EXECUTION_COUNT -eq 2) {
        # Run Attempt 2; The Command Waits For Its Saved Outcome And Listener Drain.
        $ExecutionId = $SecondId
        dokime execution run $SecondId --records --samples --record-type $RECORD_TYPES --stream $STREAMS `
            --wait-seconds $WAIT_SECONDS --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
        if ($LASTEXITCODE -gt 1 -or $LASTEXITCODE -lt 0) { throw "Dokime Returned Exit Code $LASTEXITCODE" }
        if ($LASTEXITCODE -eq 1) { $Result = 1 }
        dokime execution export $SecondId --output (Join-Path $OUTPUT_DIRECTORY 'execution-2.zip') @Export --cancel-on-interrupt `
            --owned-cycle $CycleId --recovery-directory $Recovery @Connection
        if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }
    }

    # Seal The Cycle And Download All Members Into cycle.zip.
    dokime cycle seal $CycleId @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }
    dokime cycle export $CycleId --output (Join-Path $OUTPUT_DIRECTORY 'cycle.zip') @Export @Connection
    if ($LASTEXITCODE -ne 0) { throw "Dokime Returned Exit Code $LASTEXITCODE; No Mutation Was Retried" }
    $Completed = $true
} catch {
    $OperationalFailure = $true
    [Console]::Error.WriteLine("Workflow Incomplete: $_. Inspect Cycle: $CycleId Execution: $ExecutionId Before Resubmitting")
    exit 3
} finally {
    # Recover Only This Script-Owned Cycle If Interrupted Between Calls.
    if (-not $Completed -and -not $OperationalFailure -and $CycleId) {
        try {
            $Confirmed = @(Get-Content -LiteralPath $IdFile)
            if (-not $ExecutionId -and $Confirmed.Count -eq 1) {
                $ExecutionId = [guid]::Parse([string]$Confirmed[0]).ToString()
            }
            [Console]::Error.WriteLine("Cancellation Requested For Script-Owned Cycle: $CycleId")
            if ($ExecutionId) {
                dokime execution stop $ExecutionId @Connection
                if ($LASTEXITCODE -ne 0) {
                    [Console]::Error.WriteLine('Stop Was Not Confirmed; Resolving Through The Owned Cycle Seal')
                }
            }
            dokime cycle seal $CycleId @Connection
            if ($LASTEXITCODE -ne 0) { throw "Seal Returned Exit Code $LASTEXITCODE" }
            if ($ExecutionId) {
                dokime execution wait $ExecutionId @Connection
                if ($LASTEXITCODE -gt 1 -or $LASTEXITCODE -lt 0) { throw "Wait Returned Exit Code $LASTEXITCODE" }
                $Destination = Join-Path $Recovery "execution-$ExecutionId.zip"
                if (-not (Test-Path -LiteralPath $Destination)) {
                    dokime execution export $ExecutionId --output $Destination @Connection
                    if ($LASTEXITCODE -ne 0) { throw "Export Returned Exit Code $LASTEXITCODE" }
                }
            }
            $Destination = Join-Path $Recovery "cycle-$CycleId.zip"
            if (-not (Test-Path -LiteralPath $Destination)) {
                dokime cycle export $CycleId --output $Destination @Connection
                if ($LASTEXITCODE -ne 0) { throw "Export Returned Exit Code $LASTEXITCODE" }
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
