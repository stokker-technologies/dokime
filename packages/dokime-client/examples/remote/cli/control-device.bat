@echo off
rem Dokime - SPDX-License-Identifier: Apache-2.0
setlocal EnableExtensions DisableDelayedExpansion

rem Connection Parameters
set "BACKEND_IDENTIFIER=lab"

rem Connect With The Explicit Parameters
set CONNECTION=--backend "%BACKEND_IDENTIFIER%"

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
