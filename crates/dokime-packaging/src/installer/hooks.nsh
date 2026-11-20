; Backticks delimit NSIS strings; $$ passes a literal dollar to PowerShell.
!macro NSIS_HOOK_PREINSTALL
  Push $0
  SetRegView 32
  ReadRegStr $0 HKLM "${UNINSTKEY}" "UninstallString"
  ${If} $0 == ""
    ReadRegStr $0 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTNAME}" "UninstallString"
  ${EndIf}
  ${If} $0 == ""
    SetRegView 64
    ReadRegStr $0 HKLM "${UNINSTKEY}" "UninstallString"
    ${If} $0 == ""
      ReadRegStr $0 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTNAME}" "UninstallString"
    ${EndIf}
  ${EndIf}
  ${If} $0 != ""
    DetailPrint "Uninstall the all-users copy of ${PRODUCTNAME} from Windows Settings, then run this installer again. An administrator must remove that copy."
    MessageBox MB_OK|MB_ICONSTOP "Uninstall the all-users copy of ${PRODUCTNAME} from Windows Settings, then run this installer again. An administrator must remove that copy." /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
  SetRegView 64
  SetShellVarContext current
  Pop $0
!macroend

!macro DOKIME_PATH_SCOPE
  StrCpy $0 "User"
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
