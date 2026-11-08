; Backticks delimit NSIS strings; $$ passes a literal dollar to PowerShell.
!macro DOKIME_PATH_SCOPE
  StrCpy $0 "User"
  ${If} $DokimeMachineMode = 1
    StrCpy $0 "Machine"
  ${EndIf}
  System::Call 'Kernel32::SetEnvironmentVariable(t "DOKIME_PATH_SCOPE", t r0) i .r0'
  ${If} $0 == 0
    Abort "Could not select the command-line installation scope."
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro DOKIME_PATH_SCOPE
  ${IfNot} ${FileExists} "$INSTDIR\dokime.exe"
    Abort "${PRODUCTNAME} CLI binary was not found at $INSTDIR\dokime.exe"
  ${EndIf}
  DetailPrint "Installing ${PRODUCTNAME} CLI command path"
  System::Call 'Kernel32::SetEnvironmentVariable(t "DOKIME_INSTALL_DIR", t "$INSTDIR") i .r0'
  ${If} $0 == 0
    Abort "Could not pass the installation directory to command-line setup."
  ${EndIf}
  ClearErrors
  ExecWait `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$$ErrorActionPreference='Stop'; $$dir=$$env:DOKIME_INSTALL_DIR; $$scope=$$env:DOKIME_PATH_SCOPE; $$path=[Environment]::GetEnvironmentVariable('Path', $$scope); $$parts=@($$path -split ';' | Where-Object { $$_ }); if ($$parts -notcontains $$dir) { [Environment]::SetEnvironmentVariable('Path', (($$parts + $$dir) -join ';'), $$scope) }"` $0
  ${If} ${Errors}
  ${OrIf} $0 != 0
    DetailPrint "Could not add ${PRODUCTNAME} to PATH (PowerShell exit: $0)"
    Abort "Could not add ${PRODUCTNAME} to PATH. Run the installer again to complete command-line setup."
  ${EndIf}
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro DOKIME_PATH_SCOPE
  DetailPrint "Removing ${PRODUCTNAME} CLI command path"
  System::Call 'Kernel32::SetEnvironmentVariable(t "DOKIME_INSTALL_DIR", t "$INSTDIR") i .r0'
  ${If} $0 == 0
    Abort "Could not pass the installation directory to command-line setup."
  ${EndIf}
  ClearErrors
  ExecWait `powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command "$$ErrorActionPreference='Stop'; $$dir=$$env:DOKIME_INSTALL_DIR; $$scope=$$env:DOKIME_PATH_SCOPE; $$path=[Environment]::GetEnvironmentVariable('Path', $$scope); $$parts=@($$path -split ';' | Where-Object { $$_ -and $$_ -ne $$dir }); [Environment]::SetEnvironmentVariable('Path', ($$parts -join ';'), $$scope)"` $0
  ${If} ${Errors}
  ${OrIf} $0 != 0
    DetailPrint "Could not remove ${PRODUCTNAME} from PATH (PowerShell exit: $0)"
    Abort "Could not remove ${PRODUCTNAME} from PATH. Run the uninstaller again."
  ${EndIf}
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
!macroend
