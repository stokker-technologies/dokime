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

if %EXECUTION_COUNT% LSS 1 exit /b 2

rem Materialize Cases Before Constructing Independent Attempts
dokime plan materialize --template "%TEMPLATE%" --parameters-json "{\"sample_count\":%SAMPLE_COUNT%}" --id-only %CONNECTION% > "%OUTPUT_DIRECTORY%\cases.id"
if errorlevel 1 goto Failed
set "Index=0"
for /l %%A in (1,1,%EXECUTION_COUNT%) do (
    for /f "usebackq delims=" %%C in ("%OUTPUT_DIRECTORY%\cases.id") do (
        call :PlannedDestination
        if errorlevel 1 goto Failed
    )
)
if "%Index%"=="0" goto Failed
call :Preflight "%OUTPUT_DIRECTORY%\cycle.zip"
if errorlevel 1 goto Failed

dokime cycle create --label "%CYCLE_LABEL%" --id-only %CONNECTION% > "%OUTPUT_DIRECTORY%\cycle.id"
if errorlevel 1 goto Failed
set /p "CycleId=" < "%OUTPUT_DIRECTORY%\cycle.id"
if not defined CycleId goto Failed
echo Cycle: %CycleId%
set RECOVERY=--cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery"

rem Append Every Attempt Without Starting
type nul > "%OUTPUT_DIRECTORY%\executions.id"
for /l %%A in (1,1,%EXECUTION_COUNT%) do (
    for /f "usebackq delims=" %%C in ("%OUTPUT_DIRECTORY%\cases.id") do (
        call :Append %%C
        if errorlevel 1 goto Failed
    )
)

rem Run Sequentially And Drain Requested Listeners Before Export
set "Observation="
set "FilterRemainder=%RECORD_TYPES%"

:RecordFilters
if not defined FilterRemainder goto StreamFilters
set "NextFilters="
for /f "tokens=2,*" %%F in ("x %FilterRemainder%") do (
    call :RecordFilter %%F
    set "NextFilters=%%G"
)
set "FilterRemainder=%NextFilters%"
goto RecordFilters

:StreamFilters
set "FilterRemainder=%STREAMS%"

:NextStreamFilter
if not defined FilterRemainder goto RunExecutions
set "NextFilters="
for /f "tokens=2,*" %%F in ("x %FilterRemainder%") do (
    call :StreamFilter %%F
    set "NextFilters=%%G"
)
set "FilterRemainder=%NextFilters%"
goto NextStreamFilter

:RunExecutions
set "Index=0"
for /f "usebackq delims=" %%E in ("%OUTPUT_DIRECTORY%\executions.id") do (
    call :Run %%E
    if errorlevel 2 goto Failed
)

rem Seal The Container And Save Its Complete Bundle
dokime cycle seal "%CycleId%" %CONNECTION%
if errorlevel 1 goto Failed
dokime cycle export "%CycleId%" --output "%OUTPUT_DIRECTORY%\cycle.zip" %Export% %CONNECTION%
if errorlevel 1 goto Failed
exit /b %Result%

:Preflight
if "%OVERWRITE%"=="true" exit /b 0
if exist "%~1" (
    echo Evidence File Exists: %~1 1>&2
    exit /b 2
)
exit /b 0

:PlannedDestination
set /a Index+=1
call :Preflight "%OUTPUT_DIRECTORY%\execution-%Index%.zip"
exit /b %ERRORLEVEL%

:Append
dokime cycle append "%CycleId%" --case-id "%1" --id-only %RECOVERY% %CONNECTION% > "%OUTPUT_DIRECTORY%\selected.id"
if errorlevel 1 exit /b 3
set "ExecutionId="
set /p "ExecutionId=" < "%OUTPUT_DIRECTORY%\selected.id"
if not defined ExecutionId exit /b 3
echo Execution: %ExecutionId%
rem A Trailing Digit Before ">>" Would Name A Handle; Parentheses Keep It In The Text
(echo %ExecutionId%)>> "%OUTPUT_DIRECTORY%\executions.id"
exit /b 0

:RecordFilter
set Observation=%Observation% --record-type %1
exit /b 0

:StreamFilter
set Observation=%Observation% --stream %1
exit /b 0

:Run
set "ExecutionId=%1"
set /a Index+=1
dokime execution run "%ExecutionId%" --records --samples --wait-seconds %WAIT_SECONDS% %Observation% --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 2 exit /b 3
if errorlevel 1 set "Result=1"
dokime execution export "%ExecutionId%" --output "%OUTPUT_DIRECTORY%\execution-%Index%.zip" %Export% %RECOVERY% %CONNECTION%
if errorlevel 1 exit /b 3
exit /b 0


:Failed
set "Status=%ERRORLEVEL%"
if "%Status%"=="0" set "Status=3"
echo Workflow Incomplete; Inspect Cycle: %CycleId% Execution: %ExecutionId% Before Resubmitting 1>&2
exit /b %Status%
