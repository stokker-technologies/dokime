@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
setlocal EnableExtensions DisableDelayedExpansion

rem Connection Parameters
set "ENDPOINT=https://127.0.0.1:8766"
set "CA_CERTIFICATE=server.pem"
set "KEY_FILE=remote.key"
set "ENV_FILE="

rem Case And Execution Parameters
set "TEMPLATE=obc.checks.pulse"
set "CYCLE_LABEL=Remote Confirmation Demonstration"
set "ANSWER=yes"
set "WAIT_SECONDS=60"

rem Evidence Files
set "OUTPUT_DIRECTORY=prompt-evidence"
set "OVERWRITE=false"

rem Connect With The Explicit Parameters
set CONNECTION=--endpoint "%ENDPOINT%"
if defined CA_CERTIFICATE set CONNECTION=%CONNECTION% --ca "%CA_CERTIFICATE%"
if defined KEY_FILE set CONNECTION=%CONNECTION% --key-file "%KEY_FILE%"
if defined ENV_FILE set CONNECTION=%CONNECTION% --env-file "%ENV_FILE%"

set "CycleId="
set "ExecutionId="
set "Result=0"
if %WAIT_SECONDS% LSS 1 exit /b 2
set "Export="
if "%OVERWRITE%"=="true" set "Export=--overwrite"
if not exist "%OUTPUT_DIRECTORY%\recovery" mkdir "%OUTPUT_DIRECTORY%\recovery"
if errorlevel 1 goto Failed
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
dokime plan materialize --template "%TEMPLATE%" --parameters-json "{\"require_confirmation\":true}" --id-only ^
    %CONNECTION% > "%OUTPUT_DIRECTORY%\cases.id"
if errorlevel 1 goto Failed
set "CaseId="
set "ExtraCase="
(
    set /p "CaseId="
    set /p "ExtraCase="
) < "%OUTPUT_DIRECTORY%\cases.id"
if not defined CaseId goto Failed
if defined ExtraCase goto Failed
echo Case: %CaseId%

dokime cycle create --label "%CYCLE_LABEL%" --id-only %CONNECTION% > "%OUTPUT_DIRECTORY%\cycle.id"
if errorlevel 1 goto Failed
set /p "CycleId=" < "%OUTPUT_DIRECTORY%\cycle.id"
if not defined CycleId goto Failed
echo Cycle: %CycleId%

rem Append One Attempt; Its UUID Scopes The Prompt And Evidence
dokime cycle append "%CycleId%" --case-id "%CaseId%" --id-only --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION% > "%OUTPUT_DIRECTORY%\execution.id"
if errorlevel 1 goto Failed
set "ExecutionId="
set /p "ExecutionId=" < "%OUTPUT_DIRECTORY%\execution.id"
if not defined ExecutionId goto Failed
echo Execution: %ExecutionId%

rem Start Without Blocking, Then Wait For This Execution's Next Prompt
dokime execution start "%ExecutionId%" --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 1 goto Failed
dokime execution prompt next "%ExecutionId%" --wait-seconds %WAIT_SECONDS% --id-only --cancel-on-interrupt ^
    --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION% > "%OUTPUT_DIRECTORY%\prompt.id"
if errorlevel 1 goto Failed
set "PromptId="
set /p "PromptId=" < "%OUTPUT_DIRECTORY%\prompt.id"
if not defined PromptId goto Failed
echo Prompt: %PromptId% Execution: %ExecutionId%
dokime execution prompt show "%ExecutionId%" "%PromptId%" --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 1 goto Failed

rem Answer Once; Wait For The Durable Result Before Downloading
dokime execution prompt answer "%ExecutionId%" "%PromptId%" --action "%ANSWER%" --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 1 goto Failed
dokime execution wait "%ExecutionId%" --wait-seconds %WAIT_SECONDS% --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 2 goto Failed
if errorlevel 1 set "Result=1"
dokime execution export "%ExecutionId%" --output "%OUTPUT_DIRECTORY%\execution.zip" %Export% --cancel-on-interrupt ^
    --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 1 goto Failed

rem Seal The Cycle And Download All Members Into cycle.zip
dokime cycle seal "%CycleId%" %CONNECTION%
if errorlevel 1 goto Failed
dokime cycle export "%CycleId%" --output "%OUTPUT_DIRECTORY%\cycle.zip" %Export% %CONNECTION%
if errorlevel 1 goto Failed
exit /b %Result%

:Failed
set "Status=%ERRORLEVEL%"
if "%Status%"=="0" set "Status=3"
echo Workflow Incomplete; Inspect Cycle: %CycleId% Execution: %ExecutionId% Before Resubmitting 1>&2
exit /b %Status%
