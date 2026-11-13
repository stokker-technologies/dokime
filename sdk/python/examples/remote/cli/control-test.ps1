#Requires -Version 7.3
# Dokime - SPDX-License-Identifier: Apache-2.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandArgumentPassing = 'Standard'
$PSNativeCommandUseErrorActionPreference = $true

# Connection Parameters
$PAIRING_FILE = '../lab.pair'

# Case and Execution Parameters
$TEMPLATE = 'example.checks.pulse'
$CYCLE_LABEL = 'Remote Pulse Demonstration'
$SAMPLE_COUNT = 8
$EXECUTION_COUNT = 2
$WAIT_S = 60

# Observation Filters
$RECORD_TYPES = 'example.run.*'
$STREAMS = 'example.pulse'

# Evidence Files
$OUTPUT_DIRECTORY = 'test-evidence'
$OVERWRITE = $false

# Connect with the Explicit Parameters
$Connection = @('--remote', $PAIRING_FILE)

# Uncaptured Commands Retain Native Ctrl+C Recovery; UUIDs Travel in a File.
$CycleId = ''
$ExecutionId = ''
$Completed = $false
$OperationalFailure = $false
$Export = @()
if ($OVERWRITE) { $Export = @('--overwrite') }

try {
    if ($WAIT_S -le 0) { throw 'WAIT_S Must Be Positive' }
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

    # Materialize One Immutable Case; Its ID Is Reused by Independent Attempts.
    $Parameters = @{ sample_count = $SAMPLE_COUNT } | ConvertTo-Json -Compress
    $Cases = @(dokime plan materialize --template $TEMPLATE --parameters-json $Parameters --id-only @Connection)
    if ($Cases.Count -ne 1) { throw 'Expected One Case' }
    $CaseId = [guid]::Parse([string]$Cases[0]).ToString()
    [Console]::Out.WriteLine("Case: $CaseId")
    dokime cycle create --label $CYCLE_LABEL --json @Connection > cycle.json
    $CycleId = [guid]::Parse((Get-Content -Raw cycle.json | ConvertFrom-Json).cycle_id).ToString()
    [Console]::Out.WriteLine("Cycle: $CycleId")

    # Append Both Attempts Without Starting Hardware.
    dokime cycle append $CycleId --case-id $CaseId --id-only --id-file execution.id --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    $Created = @(Get-Content -LiteralPath execution.id)
    if ($Created.Count -ne 1) { throw 'Append Did Not Return One Execution Identity' }
    $FirstId = [guid]::Parse([string]$Created[0]).ToString()
    [Console]::Out.WriteLine("Execution: $FirstId")
    if ($EXECUTION_COUNT -eq 2) {
        dokime cycle append $CycleId --case-id $CaseId --id-only --id-file execution.id --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
        $Created = @(Get-Content -LiteralPath execution.id)
        if ($Created.Count -ne 1) { throw 'Append Did Not Return One Execution Identity' }
        $SecondId = [guid]::Parse([string]$Created[0]).ToString()
        [Console]::Out.WriteLine("Execution: $SecondId")
    }

    # Run Attempt 1; the Command Waits for Its Saved Outcome and Listener Drain.
    $ExecutionId = $FirstId
    dokime execution run $FirstId --records --samples --record-type $RECORD_TYPES --stream $STREAMS --wait-seconds $WAIT_S --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    dokime execution export $FirstId --output (Join-Path $OUTPUT_DIRECTORY 'execution-1.zip') @Export --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection

    if ($EXECUTION_COUNT -eq 2) {
        # Run Attempt 2; the Command Waits for Its Saved Outcome and Listener Drain.
        $ExecutionId = $SecondId
        dokime execution run $SecondId --records --samples --record-type $RECORD_TYPES --stream $STREAMS --wait-seconds $WAIT_S --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
        dokime execution export $SecondId --output (Join-Path $OUTPUT_DIRECTORY 'execution-2.zip') @Export --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    }

    # Seal the Cycle and Download All Members into cycle.zip.
    dokime cycle seal $CycleId @Connection
    dokime cycle export $CycleId --output (Join-Path $OUTPUT_DIRECTORY 'cycle.zip') @Export @Connection
    $Completed = $true
} catch {
    $OperationalFailure = $true
    [Console]::Error.WriteLine("Workflow Incomplete: $_. Inspect Cycle: $CycleId Execution: $ExecutionId Before Resubmitting")
    exit 3
} finally {
    # Recover Only This Script-Owned Cycle If Interrupted Between Calls.
    if (-not $Completed -and -not $OperationalFailure -and $CycleId) {
        try {
            $Confirmed = @(Get-Content -LiteralPath execution.id)
            if (-not $ExecutionId -and $Confirmed.Count -eq 1) {
                $ExecutionId = [guid]::Parse([string]$Confirmed[0]).ToString()
            }
            [Console]::Error.WriteLine("Cancellation Requested For Script-Owned Cycle: $CycleId")
            if ($ExecutionId) {
                dokime execution stop $ExecutionId @Connection
            }
            dokime cycle seal $CycleId @Connection
            if ($ExecutionId) {
                $Destination = Join-Path $Recovery "execution-$ExecutionId.zip"
                if (-not (Test-Path -LiteralPath $Destination)) {
                    dokime execution export $ExecutionId --output $Destination @Connection
                }
            }
            $Destination = Join-Path $Recovery "cycle-$CycleId.zip"
            if (-not (Test-Path -LiteralPath $Destination)) {
                dokime cycle export $CycleId --output $Destination @Connection
            }
        } catch {
            [Console]::Error.WriteLine("Recovery Was Not Confirmed: $_. Retain Cycle: $CycleId Execution: $ExecutionId")
        }
        exit 130
    }
}
