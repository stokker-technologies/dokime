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
set "CYCLE_LABEL=Remote Pulse Demonstration"
set "SAMPLE_COUNT=8"
set "EXECUTION_COUNT=2"
set "WAIT_SECONDS=60"

rem Observation Filters
set RECORD_TYPES="obc.run.*"
set STREAMS="obc.pulse"

rem Evidence Files
set "OUTPUT_DIRECTORY=test-evidence"
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
if not "%EXECUTION_COUNT%"=="1" if not "%EXECUTION_COUNT%"=="2" exit /b 2

rem Preserve Each Evidence Destination Before Device Work
if not "%OVERWRITE%"=="true" if exist "%OUTPUT_DIRECTORY%\execution-1.zip" (
    echo Evidence File Exists: %OUTPUT_DIRECTORY%\execution-1.zip 1>&2
    exit /b 2
)
if "%EXECUTION_COUNT%"=="2" if not "%OVERWRITE%"=="true" if exist "%OUTPUT_DIRECTORY%\execution-2.zip" (
    echo Evidence File Exists: %OUTPUT_DIRECTORY%\execution-2.zip 1>&2
    exit /b 2
)
if not "%OVERWRITE%"=="true" if exist "%OUTPUT_DIRECTORY%\cycle.zip" (
    echo Evidence File Exists: %OUTPUT_DIRECTORY%\cycle.zip 1>&2
    exit /b 2
)

rem Materialize One Immutable Case; Each Attempt Reuses Its Case ID
dokime plan materialize --template "%TEMPLATE%" --parameters-json "{\"sample_count\":%SAMPLE_COUNT%}" --id-only ^
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

rem Append Both Attempts Without Starting Hardware
dokime cycle append "%CycleId%" --case-id "%CaseId%" --id-only --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION% > "%OUTPUT_DIRECTORY%\execution-1.id"
if errorlevel 1 goto Failed
set "FirstId="
set /p "FirstId=" < "%OUTPUT_DIRECTORY%\execution-1.id"
if not defined FirstId goto Failed
echo Execution: %FirstId%
if "%EXECUTION_COUNT%"=="1" goto RunFirst
dokime cycle append "%CycleId%" --case-id "%CaseId%" --id-only --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION% > "%OUTPUT_DIRECTORY%\execution-2.id"
if errorlevel 1 goto Failed
set "SecondId="
set /p "SecondId=" < "%OUTPUT_DIRECTORY%\execution-2.id"
if not defined SecondId goto Failed
echo Execution: %SecondId%

:RunFirst
rem Run Attempt 1; Wait For The Saved Outcome And Listener Drain
set "ExecutionId=%FirstId%"
dokime execution run "%FirstId%" --records --samples --record-type %RECORD_TYPES% --stream %STREAMS% ^
    --wait-seconds %WAIT_SECONDS% --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 2 goto Failed
if errorlevel 1 set "Result=1"
dokime execution export "%FirstId%" --output "%OUTPUT_DIRECTORY%\execution-1.zip" %Export% --cancel-on-interrupt ^
    --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 1 goto Failed

if "%EXECUTION_COUNT%"=="1" goto SealCycle
rem Run Attempt 2; Wait For The Saved Outcome And Listener Drain
set "ExecutionId=%SecondId%"
dokime execution run "%SecondId%" --records --samples --record-type %RECORD_TYPES% --stream %STREAMS% ^
    --wait-seconds %WAIT_SECONDS% --cancel-on-interrupt --owned-cycle "%CycleId%" ^
    --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 2 goto Failed
if errorlevel 1 set "Result=1"
dokime execution export "%SecondId%" --output "%OUTPUT_DIRECTORY%\execution-2.zip" %Export% --cancel-on-interrupt ^
    --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 1 goto Failed


:SealCycle
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
