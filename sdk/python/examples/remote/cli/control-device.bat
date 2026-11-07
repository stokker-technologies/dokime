@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
setlocal EnableExtensions DisableDelayedExpansion

rem Connection Parameters
set "BACKEND_IDENTIFIER=lab"

rem Connect with the Explicit Parameters
set CONNECTION=--backend "%BACKEND_IDENTIFIER%"

rem Discover the Operation IDs Offered by the Installed Plugin
dokime operation list %CONNECTION%

rem Read the Manual Device Before Changing Power
dokime operation call obc.device.read --json "{}" --output "before.json" %CONNECTION%

rem Enable the Manual Device
dokime operation call obc.device.set_power --json "{\"enabled\":true}" --output "power-on.json" %CONNECTION%

rem Read Back the Powered State: 230 V RMS
dokime operation call obc.device.read --json "{}" --output "running.json" %CONNECTION%

rem Disable the Manual Device
dokime operation call obc.device.set_power --json "{\"enabled\":false}" --output "power-off.json" %CONNECTION%

rem Read Back the Powered-Off State: Zero Volts
dokime operation call obc.device.read --json "{}" --output "after.json" %CONNECTION%
