; Dokime
; SPDX-License-Identifier: Apache-2.0

; Reject an Existing Machine-Wide Installation
!macro NSIS_HOOK_PREINSTALL
  Push $0
  SetRegView 32
  ReadRegStr $0 HKLM "${UNINSTKEY}" "UninstallString"
  ${If} $0 == ""
    SetRegView 64
    ReadRegStr $0 HKLM "${UNINSTKEY}" "UninstallString"
  ${EndIf}
  ${If} $0 != ""
    DetailPrint "Uninstall the all-users copy of ${PRODUCTNAME} from Windows Settings, then run this installer again. An administrator must remove that copy."
    MessageBox MB_OK|MB_ICONSTOP "Uninstall the all-users copy of ${PRODUCTNAME} from Windows Settings, then run this installer again. An administrator must remove that copy." /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
  !insertmacro SetContext
  Pop $0
!macroend
