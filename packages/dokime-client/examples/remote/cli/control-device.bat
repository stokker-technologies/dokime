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

rem Reserve Reply Files Before Connecting; Manual Control Is Not Execution Evidence
if defined RESPONSE_DIRECTORY (
    if exist "%RESPONSE_DIRECTORY%" (
        echo Response Directory Exists; Existing Data Is Preserved 1>&2
        exit /b 2
    )
    mkdir "%RESPONSE_DIRECTORY%"
    if errorlevel 1 goto Failed
)

rem Discover The Operation IDs Offered By The Installed Plugin
dokime operation list %CONNECTION%
if errorlevel 1 goto Failed

rem Read The Manual Device Before Changing Power
if defined RESPONSE_DIRECTORY (
    dokime operation call obc.device.read --json "{}" --output "%RESPONSE_DIRECTORY%\before.json" %CONNECTION%
) else (
    dokime operation call obc.device.read --json "{}" %CONNECTION%
)
if errorlevel 1 goto Failed

rem Enable The Manual Device
if defined RESPONSE_DIRECTORY (
    dokime operation call obc.device.set_power --json "{\"enabled\":true}" --output "%RESPONSE_DIRECTORY%\power-on.json" %CONNECTION%
) else (
    dokime operation call obc.device.set_power --json "{\"enabled\":true}" %CONNECTION%
)
if errorlevel 1 goto Failed

rem Read Back The Powered State: 230 V RMS
if defined RESPONSE_DIRECTORY (
    dokime operation call obc.device.read --json "{}" --output "%RESPONSE_DIRECTORY%\running.json" %CONNECTION%
) else (
    dokime operation call obc.device.read --json "{}" %CONNECTION%
)
if errorlevel 1 goto Failed

rem Disable The Manual Device
if defined RESPONSE_DIRECTORY (
    dokime operation call obc.device.set_power --json "{\"enabled\":false}" --output "%RESPONSE_DIRECTORY%\power-off.json" %CONNECTION%
) else (
    dokime operation call obc.device.set_power --json "{\"enabled\":false}" %CONNECTION%
)
if errorlevel 1 goto Failed

rem Read Back The Powered-Off State: Zero Volts
if defined RESPONSE_DIRECTORY (
    dokime operation call obc.device.read --json "{}" --output "%RESPONSE_DIRECTORY%\after.json" %CONNECTION%
) else (
    dokime operation call obc.device.read --json "{}" %CONNECTION%
)
if errorlevel 1 goto Failed

exit /b 0

:Failed
echo Device Workflow Incomplete; Read Device State Before Resending. No Command Was Retried 1>&2
exit /b 3
