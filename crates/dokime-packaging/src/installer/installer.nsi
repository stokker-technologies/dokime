; Adapted From the Removed Organization Installer and Tauri's NSIS Template.
; Copyright 2019-2024 Tauri Programme Within The Commons Conservancy.
; SPDX-License-Identifier: Apache-2.0 OR MIT
Unicode true
ManifestDPIAware true
ManifestDPIAwareness PerMonitorV2
RequestExecutionLevel user
SetCompressor /SOLID lzma
!include MUI2.nsh
!include FileFunc.nsh
!include LogicLib.nsh
!include x64.nsh
!include "Win\RestartManager.nsh"
!include "utils.nsh"
!addplugindir "{{additional_plugins_path}}"
!define PRODUCTNAME "{{product_name}}"
!define BUNDLEID "{{bundle_id}}"
!define MAINBINARYNAME "{{main_binary_name}}"
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${BUNDLEID}"
!define MUI_ICON "{{installer_icon}}"
!define MUI_UNICON "{{installer_icon}}"
!define MUI_WELCOMEFINISHPAGE_BITMAP "{{sidebar_image}}"
!include "{{installer_hooks}}"
Name "${PRODUCTNAME}"
OutFile "{{out_file}}"
InstallDir "$LOCALAPPDATA\${BUNDLEID}"
InstallDirRegKey HKCU "${UNINSTKEY}" "InstallLocation"
VIProductVersion "{{version_with_build}}"
VIAddVersionKey "ProductName" "${PRODUCTNAME}"
VIAddVersionKey "FileDescription" "${PRODUCTNAME}"
VIAddVersionKey "FileVersion" "{{version}}"
VIAddVersionKey "ProductVersion" "{{version}}"
VIAddVersionKey "LegalCopyright" "{{copyright}}"
Var UpdateMode
Var PassiveMode
Var NoShortcutMode
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!insertmacro MUI_PAGE_WELCOME
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!insertmacro MUI_PAGE_LICENSE "{{license}}"
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
{{#each language_files}}
!include "{{this}}"
{{/each}}

Function SkipIfPassive
  ${If} $PassiveMode = 1
    Abort
  ${EndIf}
FunctionEnd

Function .onInit
  SetShellVarContext current
  SetRegView 64
  ; Refuse Machine Scope Rather Than Elevating or Selecting Another Installation.
  ClearErrors
  ${GetOptions} $CMDLINE "/ALLUSERS" $0
  ${IfNot} ${Errors}
    SetErrorLevel 87
    Quit
  ${EndIf}
  ClearErrors
  ${GetOptions} $CMDLINE "/UPDATE" $0
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $CMDLINE "/P" $0
  ${IfNot} ${Errors}
    StrCpy $PassiveMode 1
  ${EndIf}
  ClearErrors
  ${GetOptions} $CMDLINE "/NS" $0
  ${IfNot} ${Errors}
    StrCpy $NoShortcutMode 1
  ${EndIf}
FunctionEnd

Section "WebView2"
  ; Read Both Runtime Scopes; Bootstrap Missing Runtimes as the Current User.
  ReadRegStr $0 HKCU "Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${If} $0 == ""
    ReadRegStr $0 HKLM "Software\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${EndIf}
  ${If} $0 == ""
    InitPluginsDir
    {{#if webview2_bootstrapper_path}}
    File "/oname=$PLUGINSDIR\webview2.exe" "{{webview2_bootstrapper_path}}"
    {{else}}
    NSISdl::download "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$PLUGINSDIR\webview2.exe"
    Pop $0
    ${If} $0 != "success"
      Abort "Install the WebView2 Runtime for This User and Run Setup Again."
    ${EndIf}
    {{/if}}
    ExecWait '"$PLUGINSDIR\webview2.exe" /silent /install' $0
    ${If} $0 != 0
      Abort "Install the WebView2 Runtime for This User and Run Setup Again."
    ${EndIf}
  ${EndIf}
SectionEnd

Section "Install"
  !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  SetOutPath "$INSTDIR"
  ; Remove Only the Previous Installer's Owned Files Before Applying a Replacement Pack.
  ${If} ${FileExists} "$INSTDIR\uninstall.exe"
    ExecWait '"$INSTDIR\uninstall.exe" /S /UPDATE _?=$INSTDIR' $0
    ${If} $0 != 0
      Abort "Close the Application and Run Setup Again to Replace the Previous Installation."
    ${EndIf}
  ${EndIf}
  File "{{main_binary_path}}"
  {{#each resources_dirs}}
    CreateDirectory "$INSTDIR\\{{this}}"
  {{/each}}
  {{#each resources}}
    File /a "/oname={{this.[1]}}" "{{no-escape @key}}"
  {{/each}}
  {{#each binaries}}
    File /a "/oname={{this}}" "{{no-escape @key}}"
  {{/each}}
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayName" "${PRODUCTNAME}"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayVersion" "{{version}}"
  WriteRegStr HKCU "${UNINSTKEY}" "Publisher" "{{manufacturer}}"
  WriteRegStr HKCU "${UNINSTKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "UninstallString" '$\"$INSTDIR\uninstall.exe$\"'
  WriteRegStr HKCU "${UNINSTKEY}" "QuietUninstallString" '$\"$INSTDIR\uninstall.exe$\" /S'
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayIcon" "$INSTDIR\${MAINBINARYNAME}.exe"
  WriteRegDWORD HKCU "${UNINSTKEY}" "EstimatedSize" {{estimated_size}}
  WriteRegDWORD HKCU "${UNINSTKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTKEY}" "NoRepair" 1
  ${If} $NoShortcutMode != 1
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  ${EndIf}
  !insertmacro NSIS_HOOK_POSTINSTALL
SectionEnd

Function .onInstSuccess
  ClearErrors
  ${GetOptions} $CMDLINE "/R" $0
  ${IfNot} ${Errors}
    Exec '"$INSTDIR\${MAINBINARYNAME}.exe"'
  ${EndIf}
FunctionEnd

Function un.onInit
  SetShellVarContext current
  SetRegView 64
  ClearErrors
  ${GetOptions} $CMDLINE "/UPDATE" $0
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}
FunctionEnd

Section "Uninstall"
  !insertmacro CheckIfAppIsRunning "$INSTDIR\${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  !insertmacro NSIS_HOOK_PREUNINSTALL
  Delete "$INSTDIR\${MAINBINARYNAME}.exe"
  {{#each resources}}
    Delete "$INSTDIR\\{{this.[1]}}"
  {{/each}}
  {{#each binaries}}
    Delete "$INSTDIR\\{{this}}"
  {{/each}}
  {{#each resources_ancestors}}
    RMDir "$INSTDIR\\{{this}}"
  {{/each}}
  Delete "$INSTDIR\uninstall.exe"
  ${If} $UpdateMode != 1
    Delete "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    Delete "$DESKTOP\${PRODUCTNAME}.lnk"
  ${EndIf}
  DeleteRegKey HKCU "${UNINSTKEY}"
  RMDir "$INSTDIR"
SectionEnd
