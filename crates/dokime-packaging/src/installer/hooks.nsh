; Backticks delimit NSIS strings; $$ passes a literal dollar to PowerShell.
!macro NSIS_HOOK_POSTINSTALL
  ${IfNot} ${FileExists} "$INSTDIR\dokime.exe"
    Abort "Dokime CLI binary was not found at $INSTDIR\dokime.exe"
  ${EndIf}
  DetailPrint "Installing Dokime CLI command path"
  System::Call 'Kernel32::SetEnvironmentVariable(t "DOKIME_INSTALL_DIR", t "$INSTDIR") i .r0'
  ${If} $0 == 0
    Abort "Could not pass the installation directory to command-line setup."
  ${EndIf}
  ClearErrors
  ExecWait `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$$ErrorActionPreference='Stop'; $$dir=$$env:DOKIME_INSTALL_DIR; $$path=[Environment]::GetEnvironmentVariable('Path', 'User'); $$parts=@($$path -split ';' | Where-Object { $$_ }); if ($$parts -notcontains $$dir) { [Environment]::SetEnvironmentVariable('Path', (($$parts + $$dir) -join ';'), 'User') }"` $0
  ${If} ${Errors}
  ${OrIf} $0 != 0
    DetailPrint "Could not add Dokime to the user PATH (PowerShell exit: $0)"
    Abort "Could not add Dokime to PATH. Run the installer again to complete command-line setup."
  ${EndIf}
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Removing Dokime CLI command path"
  System::Call 'Kernel32::SetEnvironmentVariable(t "DOKIME_INSTALL_DIR", t "$INSTDIR") i .r0'
  ${If} $0 == 0
    Abort "Could not pass the installation directory to command-line setup."
  ${EndIf}
  ClearErrors
  ExecWait `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$$ErrorActionPreference='Stop'; $$dir=$$env:DOKIME_INSTALL_DIR; $$path=[Environment]::GetEnvironmentVariable('Path', 'User'); $$parts=@($$path -split ';' | Where-Object { $$_ -and $$_ -ne $$dir }); [Environment]::SetEnvironmentVariable('Path', ($$parts -join ';'), 'User')"` $0
  ${If} ${Errors}
  ${OrIf} $0 != 0
    DetailPrint "Could not remove Dokime from the user PATH (PowerShell exit: $0)"
    Abort "Could not remove Dokime from PATH. Run the uninstaller again."
  ${EndIf}
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend
