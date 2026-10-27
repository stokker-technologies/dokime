@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
setlocal EnableExtensions DisableDelayedExpansion

rem Connection Parameters
set "BACKEND_IDENTIFIER=lab"

rem Case And Execution Parameters
set "TEMPLATE=obc.checks.pulse"
set "CYCLE_LABEL=Remote Confirmation Demonstration"
set "ANSWER=yes"
set "WAIT_SECONDS=60"

rem Evidence Files
set "OUTPUT_DIRECTORY=prompt-evidence"
set "OVERWRITE=false"

rem Connect With The Explicit Parameters
set CONNECTION=--backend "%BACKEND_IDENTIFIER%"

set "CycleId="
set "ExecutionId="
if %WAIT_SECONDS% LSS 1 exit /b 2
set "Export="
if "%OVERWRITE%"=="true" set "Export=--overwrite"
if not exist "%OUTPUT_DIRECTORY%\recovery" mkdir "%OUTPUT_DIRECTORY%\recovery"
if not "%ANSWER%"=="yes" if not "%ANSWER%"=="no" exit /b 2

rem Preserve Each Evidence Destination Before Device Work
if not "%OVERWRITE%"=="true" if exist "%OUTPUT_DIRECTORY%\execution.zip" (
    echo Evidence File Exists: %OUTPUT_DIRECTORY%\execution.zip 1>&2
    exit /b 2
)
if not "%OVERWRITE%"=="true" if exist "%OUTPUT_DIRECTORY%\cycle.zip" (
    echo Evidence File Exists: %OUTPUT_DIRECTORY%\cycle.zip 1>&2
    exit /b 2
)

rem Materialize One Immutable Case; Each Attempt Reuses Its Case ID
dokime plan materialize --template "%TEMPLATE%" --parameters-json "{\"require_confirmation\":true}" --id-only  %CONNECTION% > "case.id"
set "CaseId="
set "ExtraCase="
(
    set /p "CaseId="
    set /p "ExtraCase="
) < "case.id"
if not defined CaseId goto Failed
if defined ExtraCase goto Failed
echo Case: %CaseId%

dokime cycle create --label "%CYCLE_LABEL%" --json %CONNECTION% > cycle.json
for /f "tokens=2 delims=:{}" %%I in (cycle.json) do set "CycleId=%%~I"
if not defined CycleId goto Failed
echo Cycle: %CycleId%

rem Append One Attempt; Its UUID Scopes The Prompt And Evidence
dokime cycle append "%CycleId%" --case-id "%CaseId%" --id-only --id-file execution.id --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
set "ExecutionId="
set /p "ExecutionId=" < execution.id
if not defined ExecutionId goto Failed
echo Execution: %ExecutionId%

rem Start Without Blocking, Then Wait For This Execution's Next Prompt
dokime execution start "%ExecutionId%" --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
dokime execution prompt next "%ExecutionId%" --wait-seconds %WAIT_SECONDS% --id-only --id-file prompt.id --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
set "PromptId="
set /p "PromptId=" < prompt.id
if not defined PromptId goto Failed
echo Prompt: %PromptId% Execution: %ExecutionId%
dokime execution prompt show "%ExecutionId%" "%PromptId%" --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION% --json > prompt.json

rem Answer Once; Wait For The Durable Result Before Downloading
dokime execution prompt answer "%ExecutionId%" "%PromptId%" --action "%ANSWER%" --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
dokime execution wait "%ExecutionId%" --wait-seconds %WAIT_SECONDS% --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
dokime execution export "%ExecutionId%" --output "%OUTPUT_DIRECTORY%\execution.zip" %Export% --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%

rem Seal The Cycle And Download All Members Into cycle.zip
dokime cycle seal "%CycleId%" %CONNECTION%
dokime cycle export "%CycleId%" --output "%OUTPUT_DIRECTORY%\cycle.zip" %Export% %CONNECTION%
exit /b

:Failed
echo Workflow Incomplete; Inspect Cycle: %CycleId% Execution: %ExecutionId% Before Resubmitting 1>&2
exit /b 3
