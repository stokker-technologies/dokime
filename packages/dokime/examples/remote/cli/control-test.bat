@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
rem Pass Connection Flags Such As --endpoint, --ca, --key-file Or --env-file.
setlocal EnableExtensions DisableDelayedExpansion
set "Directory=%DOKIME_OUTPUT_DIRECTORY%"
if not defined Directory set "Directory=test-evidence"
set "CycleId="
set "ExecutionId="
set "RequestKey="
set "Result=0"

if exist "%Directory%" goto DirectoryExists
mkdir "%Directory%\recovery"
if errorlevel 1 goto Failed

dokime cycle create --label "Remote Pulse Demonstration" --id-only %* > "%Directory%\cycle.id"
if errorlevel 1 goto Failed
set /p "CycleId=" < "%Directory%\cycle.id"
if not defined CycleId goto Failed
echo Cycle: %CycleId%

rem First Execution; The Native CLI Owns Ctrl-C Recovery While Waiting.
dokime cycle append "%CycleId%" --template obc.checks.pulse --id-only %* > "%Directory%\execution-1.id"
if errorlevel 1 goto Failed
set /p "ExecutionId=" < "%Directory%\execution-1.id"
if not defined ExecutionId goto Failed
set "RequestKey=remote-%ExecutionId%"
echo Execution: %ExecutionId% Request Key: %RequestKey% Mode: automatic
dokime execution run "%ExecutionId%" --request-key "%RequestKey%" %*
if errorlevel 1 goto Failed
dokime execution wait "%ExecutionId%" --wait-seconds 120 --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%Directory%\recovery" %*
if errorlevel 2 goto Failed
if errorlevel 1 set "Result=1"
dokime execution export "%ExecutionId%" --output "%Directory%\execution-1.zip" %*
if errorlevel 1 goto Failed

rem Append Another Execution To The Same Still-Active Cycle.
set "ExecutionId="
dokime cycle append "%CycleId%" --template obc.checks.pulse --id-only %* > "%Directory%\execution-2.id"
if errorlevel 1 goto Failed
set /p "ExecutionId=" < "%Directory%\execution-2.id"
if not defined ExecutionId goto Failed
set "RequestKey=remote-%ExecutionId%"
echo Execution: %ExecutionId% Request Key: %RequestKey% Mode: automatic
dokime execution run "%ExecutionId%" --request-key "%RequestKey%" %*
if errorlevel 1 goto Failed
dokime execution wait "%ExecutionId%" --wait-seconds 120 --cancel-on-interrupt --owned-cycle "%CycleId%" --recovery-directory "%Directory%\recovery" %*
if errorlevel 2 goto Failed
if errorlevel 1 set "Result=1"
dokime execution export "%ExecutionId%" --output "%Directory%\execution-2.zip" %*
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
echo Recovery Cycle: %CycleId% Execution: %ExecutionId% Request Key: %RequestKey% 1>&2
echo A Failed Connection Does Not Confirm Cancellation. Inspect The Original Identities Before Resubmitting 1>&2
exit /b %Status%
