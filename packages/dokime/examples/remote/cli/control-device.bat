@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
rem Pass Connection Flags Such As --endpoint, --ca, --key-file Or --env-file.
setlocal EnableExtensions DisableDelayedExpansion
set "Responses=%DOKIME_RESPONSE_DIRECTORY%"

if not defined Responses goto Discover
if exist "%Responses%" goto DirectoryExists
mkdir "%Responses%"
if errorlevel 1 goto Failed

:Discover
dokime operation list %*
if errorlevel 1 goto Failed

if defined Responses (
    dokime operation call obc.device.read --json "{}" --output "%Responses%\before.json" %*
) else (
    dokime operation call obc.device.read --json "{}" %*
)
if errorlevel 1 goto Failed

if defined Responses (
    dokime operation call obc.device.set_power --json "{\"enabled\":true}" --output "%Responses%\power-on.json" %*
) else (
    dokime operation call obc.device.set_power --json "{\"enabled\":true}" %*
)
if errorlevel 1 goto Failed

if defined Responses (
    dokime operation call obc.device.read --json "{}" --output "%Responses%\running.json" %*
) else (
    dokime operation call obc.device.read --json "{}" %*
)
if errorlevel 1 goto Failed

if defined Responses (
    dokime operation call obc.device.set_power --json "{\"enabled\":false}" --output "%Responses%\power-off.json" %*
) else (
    dokime operation call obc.device.set_power --json "{\"enabled\":false}" %*
)
if errorlevel 1 goto Failed

if defined Responses (
    dokime operation call obc.device.read --json "{}" --output "%Responses%\after.json" %*
) else (
    dokime operation call obc.device.read --json "{}" %*
)
if errorlevel 1 goto Failed
exit /b 0

:DirectoryExists
echo Choose A New Response Directory; Existing Data Is Preserved 1>&2
exit /b 2

:Failed
set "Status=%ERRORLEVEL%"
if "%Status%"=="0" set "Status=3"
echo Device Workflow Incomplete; No Command Was Retried 1>&2
echo A Dispatched Command May Have Acted. Read Device State Before Resending 1>&2
exit /b %Status%
