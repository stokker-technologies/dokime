@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
setlocal EnableExtensions DisableDelayedExpansion

rem Connection Parameters
set "ENDPOINT=https://127.0.0.1:8766"
set "CA_CERTIFICATE=server.pem"
set "KEY_FILE=remote.key"
set "ENV_FILE="

rem Connect With The Explicit Parameters
set CONNECTION=--endpoint "%ENDPOINT%"
if defined CA_CERTIFICATE set CONNECTION=%CONNECTION% --ca "%CA_CERTIFICATE%"
if defined KEY_FILE set CONNECTION=%CONNECTION% --key-file "%KEY_FILE%"
if defined ENV_FILE set CONNECTION=%CONNECTION% --env-file "%ENV_FILE%"

rem Discover The Operation IDs Offered By The Installed Plugin
dokime operation list %CONNECTION%

rem Read The Manual Device Before Changing Power
dokime operation call obc.device.read --json "{}" --output "before.json" %CONNECTION%

rem Enable The Manual Device
dokime operation call obc.device.set_power --json "{\"enabled\":true}" --output "power-on.json" %CONNECTION%

rem Read Back The Powered State: 230 V RMS
dokime operation call obc.device.read --json "{}" --output "running.json" %CONNECTION%

rem Disable The Manual Device
dokime operation call obc.device.set_power --json "{\"enabled\":false}" --output "power-off.json" %CONNECTION%

rem Read Back The Powered-Off State: Zero Volts
dokime operation call obc.device.read --json "{}" --output "after.json" %CONNECTION%
