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
call :Preflight "%OUTPUT_DIRECTORY%\execution.zip"
if errorlevel 1 goto Failed
call :Preflight "%OUTPUT_DIRECTORY%\cycle.zip"
if errorlevel 1 goto Failed

rem Materialize The Confirmation Case And Append Its Attempt
dokime plan materialize --template "%TEMPLATE%" --parameters-json "{\"require_confirmation\":true}" --id-only %CONNECTION% > "%OUTPUT_DIRECTORY%\cases.id"
if errorlevel 1 goto Failed
set "CaseId="
set "CaseCount=0"
for /f "usebackq delims=" %%C in ("%OUTPUT_DIRECTORY%\cases.id") do call :SelectCase %%C
if not "%CaseCount%"=="1" goto Failed

dokime cycle create --label "%CYCLE_LABEL%" --id-only %CONNECTION% > "%OUTPUT_DIRECTORY%\cycle.id"
if errorlevel 1 goto Failed
set /p "CycleId=" < "%OUTPUT_DIRECTORY%\cycle.id"
if not defined CycleId goto Failed
echo Cycle: %CycleId%
set RECOVERY=--cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery"

dokime cycle append "%CycleId%" --case-id "%CaseId%" --id-only %RECOVERY% %CONNECTION% > "%OUTPUT_DIRECTORY%\execution.id"
if errorlevel 1 goto Failed
set /p "ExecutionId=" < "%OUTPUT_DIRECTORY%\execution.id"
if not defined ExecutionId goto Failed
echo Execution: %ExecutionId%

rem Start Without Blocking So The Script Can Answer Its Prompt
dokime execution start "%ExecutionId%" %RECOVERY% %CONNECTION%
if errorlevel 1 goto Failed
dokime execution prompt next "%ExecutionId%" --wait-seconds %WAIT_SECONDS% --id-only --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION% > "%OUTPUT_DIRECTORY%\prompt.id"
if errorlevel 1 goto Failed
set "PromptId="
set /p "PromptId=" < "%OUTPUT_DIRECTORY%\prompt.id"
if not defined PromptId goto Failed
echo Prompt: %PromptId% Execution: %ExecutionId%
dokime execution prompt show "%ExecutionId%" "%PromptId%" %RECOVERY% %CONNECTION%
if errorlevel 1 goto Failed
dokime execution prompt answer "%ExecutionId%" "%PromptId%" --action "%ANSWER%" %RECOVERY% %CONNECTION%
if errorlevel 1 goto Failed
dokime execution wait "%ExecutionId%" --wait-seconds %WAIT_SECONDS% --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%OUTPUT_DIRECTORY%\recovery" %CONNECTION%
if errorlevel 2 goto Failed
if errorlevel 1 set "Result=1"
dokime execution export "%ExecutionId%" --output "%OUTPUT_DIRECTORY%\execution.zip" %Export% %RECOVERY% %CONNECTION%
if errorlevel 1 goto Failed

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

:SelectCase
set "CaseId=%1"
set /a CaseCount+=1
exit /b 0


:Failed
set "Status=%ERRORLEVEL%"
if "%Status%"=="0" set "Status=3"
echo Workflow Incomplete; Inspect Cycle: %CycleId% Execution: %ExecutionId% Before Resubmitting 1>&2
exit /b %Status%
