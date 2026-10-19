@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
setlocal EnableExtensions DisableDelayedExpansion

rem Connection Parameters
set "ENDPOINT=https://127.0.0.1:8766"
set "CA_CERTIFICATE=server.pem"
set "KEY_FILE=remote.key"
set "ENV_FILE="

rem Response Files
set "RESPONSE_DIRECTORY="

rem Connect With The Explicit Parameters
set CONNECTION=--endpoint "%ENDPOINT%"
if defined CA_CERTIFICATE set CONNECTION=%CONNECTION% --ca "%CA_CERTIFICATE%"
if defined KEY_FILE set CONNECTION=%CONNECTION% --key-file "%KEY_FILE%"
if defined ENV_FILE set CONNECTION=%CONNECTION% --env-file "%ENV_FILE%"

rem Reserve All Response Files Before Device Work
if not defined RESPONSE_DIRECTORY goto Discover
if exist "%RESPONSE_DIRECTORY%" (
    echo Response Directory Exists; Existing Data Is Preserved 1>&2
    exit /b 2
)
mkdir "%RESPONSE_DIRECTORY%"
if errorlevel 1 goto Failed

:Discover
dokime operation list %CONNECTION%
if errorlevel 1 goto Failed

rem Read, Power On, Observe, Power Off And Verify
call :Operation obc.device.read "{}" before
if errorlevel 1 goto Failed
call :Operation obc.device.set_power "{\"enabled\":true}" power-on
if errorlevel 1 goto Failed
call :Operation obc.device.read "{}" running
if errorlevel 1 goto Failed
call :Operation obc.device.set_power "{\"enabled\":false}" power-off
if errorlevel 1 goto Failed
call :Operation obc.device.read "{}" after
if errorlevel 1 goto Failed
exit /b 0

:Operation
if defined RESPONSE_DIRECTORY (
    dokime operation call %1 --json %2 --output "%RESPONSE_DIRECTORY%\%3.json" %CONNECTION%
) else (
    dokime operation call %1 --json %2 %CONNECTION%
)
exit /b %ERRORLEVEL%

:Failed
echo Device Workflow Incomplete; Read Device State Before Resending. No Command Was Retried 1>&2
exit /b 3
