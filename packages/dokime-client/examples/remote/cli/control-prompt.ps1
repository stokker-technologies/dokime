#Requires -Version 7.3
# Dokime - SPDX-License-Identifier: Apache-2.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandArgumentPassing = 'Standard'
$PSNativeCommandUseErrorActionPreference = $true

# Connection Parameters
$BACKEND_IDENTIFIER = 'lab'

# Case And Execution Parameters
$TEMPLATE = 'obc.checks.pulse'
$CYCLE_LABEL = 'Remote Confirmation Demonstration'
$ANSWER = 'yes'
$WAIT_SECONDS = 60

# Evidence Files
$OUTPUT_DIRECTORY = 'prompt-evidence'
$OVERWRITE = $false

# Connect With The Explicit Parameters
$Connection = @('--backend', $BACKEND_IDENTIFIER)

# Uncaptured Commands Retain Native Ctrl+C Recovery; UUIDs Travel In A File.
$CycleId = ''
$ExecutionId = ''
$Completed = $false
$OperationalFailure = $false
$Export = @()
if ($OVERWRITE) { $Export = @('--overwrite') }

try {
    if ($WAIT_SECONDS -le 0) { throw 'WAIT_SECONDS Must Be Positive' }
    New-Item -ItemType Directory -Force -Path $OUTPUT_DIRECTORY | Out-Null
    $Recovery = Join-Path $OUTPUT_DIRECTORY 'recovery'
    New-Item -ItemType Directory -Force -Path $Recovery | Out-Null
    if ($ANSWER -notin @('yes', 'no')) { throw 'ANSWER Must Be yes Or no' }
    if ((Test-Path -LiteralPath (Join-Path $OUTPUT_DIRECTORY 'execution.zip')) -and -not $OVERWRITE) {
        throw 'Evidence File Exists: execution.zip'
    }
    if ((Test-Path -LiteralPath (Join-Path $OUTPUT_DIRECTORY 'cycle.zip')) -and -not $OVERWRITE) {
        throw 'Evidence File Exists: cycle.zip'
    }

    # Materialize One Immutable Case; Its ID Is Reused By Independent Attempts.
    $Cases = @(dokime plan materialize --template $TEMPLATE --parameters-json '{"require_confirmation":true}' --id-only @Connection)
    if ($Cases.Count -ne 1) { throw 'Expected One Case' }
    $CaseId = [guid]::Parse([string]$Cases[0]).ToString()
    [Console]::Out.WriteLine("Case: $CaseId")
    dokime cycle create --label $CYCLE_LABEL --json @Connection > cycle.json
    $CycleId = [guid]::Parse((Get-Content -Raw cycle.json | ConvertFrom-Json).cycle_id).ToString()
    [Console]::Out.WriteLine("Cycle: $CycleId")

    # Append One Attempt; Its UUID Scopes The Prompt And Evidence.
    dokime cycle append $CycleId --case-id $CaseId --id-only --id-file execution.id --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    $Created = @(Get-Content -LiteralPath execution.id)
    if ($Created.Count -ne 1) { throw 'Append Did Not Return One Execution Identity' }
    $ExecutionId = [guid]::Parse([string]$Created[0]).ToString()
    [Console]::Out.WriteLine("Execution: $ExecutionId")

    # Start Without Blocking, Then Wait For This Execution's Next Prompt.
    dokime execution start $ExecutionId --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    dokime execution prompt next $ExecutionId --wait-seconds $WAIT_SECONDS --id-only --id-file prompt.id --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    $Pending = @(Get-Content -LiteralPath prompt.id)
    if ($Pending.Count -ne 1) { throw 'Prompt Discovery Did Not Return One Identity' }
    $PromptId = [guid]::Parse([string]$Pending[0]).ToString()
    [Console]::Out.WriteLine("Prompt: $PromptId Execution: $ExecutionId")
    dokime execution prompt show $ExecutionId $PromptId --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection --json > prompt.json

    # Answer Once; Wait For The Durable Result Before Downloading.
    dokime execution prompt answer $ExecutionId $PromptId --action $ANSWER --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    dokime execution wait $ExecutionId --wait-seconds $WAIT_SECONDS --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection
    dokime execution export $ExecutionId --output (Join-Path $OUTPUT_DIRECTORY 'execution.zip') @Export --cancel-on-interrupt --owned-cycle $CycleId --recovery-directory $Recovery @Connection

    # Seal The Cycle And Download All Members Into cycle.zip.
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
                dokime execution wait $ExecutionId @Connection
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
