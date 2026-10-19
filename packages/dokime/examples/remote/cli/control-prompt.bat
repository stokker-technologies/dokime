@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
rem Pass Connection Flags Such As --endpoint, --ca, --key-file Or --env-file.
setlocal EnableExtensions DisableDelayedExpansion
set "Directory=%DOKIME_OUTPUT_DIRECTORY%"
if not defined Directory set "Directory=prompt-evidence"
set "CycleId="
set "ExecutionId="
set "PromptId="
set "RequestKey="
set "Result=0"

if exist "%Directory%" goto DirectoryExists
mkdir "%Directory%\recovery"
if errorlevel 1 goto Failed

dokime cycle create --label "Remote Confirmation Demonstration" --id-only %* > "%Directory%\cycle.id"
if errorlevel 1 goto Failed
set /p "CycleId=" < "%Directory%\cycle.id"
if not defined CycleId goto Failed
echo Cycle: %CycleId%

dokime cycle append "%CycleId%" --template obc.checks.pulse --parameters-json "{\"require_confirmation\":true}" --id-only %* > "%Directory%\execution.id"
if errorlevel 1 goto Failed
set /p "ExecutionId=" < "%Directory%\execution.id"
if not defined ExecutionId goto Failed
set "RequestKey=remote-%ExecutionId%"
echo Execution: %ExecutionId% Request Key: %RequestKey% Mode: automatic
dokime execution run "%ExecutionId%" --request-key "%RequestKey%" %*
if errorlevel 1 goto Failed

dokime execution prompt next "%ExecutionId%" --wait-seconds 60 --id-only --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%Directory%\recovery" %* > "%Directory%\prompt.id"
if errorlevel 1 goto Failed
set /p "PromptId=" < "%Directory%\prompt.id"
if not defined PromptId goto Failed
echo Prompt: %PromptId% Execution: %ExecutionId%
dokime execution prompt show "%ExecutionId%" "%PromptId%" %*
if errorlevel 1 goto Failed
dokime execution prompt answer "%ExecutionId%" "%PromptId%" --action yes %*
if errorlevel 1 goto Failed

dokime execution wait "%ExecutionId%" --wait-seconds 120 --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%Directory%\recovery" %*
if errorlevel 2 goto Failed
if errorlevel 1 set "Result=1"
dokime execution export "%ExecutionId%" --output "%Directory%\execution.zip" %*
if errorlevel 1 goto Failed
dokime cycle seal "%CycleId%" %*
if errorlevel 1 goto Failed
dokime cycle export "%CycleId%" --output "%Directory%\cycle.zip" %*
if errorlevel 1 goto Failed
exit /b %Result%

:DirectoryExists
echo Choose A New Output Directory; Existing Evidence Is Preserved 1>&2
exit /b 2

:Failed
set "Status=%ERRORLEVEL%"
if "%Status%"=="0" set "Status=3"
echo Recovery Cycle: %CycleId% Execution: %ExecutionId% Prompt: %PromptId% Request Key: %RequestKey% 1>&2
echo A Failed Connection Does Not Confirm Cancellation Or An Answer. Inspect The Original Identities Before Resubmitting 1>&2
exit /b %Status%
