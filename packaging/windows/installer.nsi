; DroidKraft TUI installer. Built by scripts/ci/package_windows.nu with -DVERSION -DSRC -DOUT
!include "MUI2.nsh"
!include "StrFunc.nsh"
${StrStr}
${UnStrRep}

Name "DroidKraft (terminal)"
OutFile "${OUT}"
InstallDir "$PROGRAMFILES64\DroidKraft"
RequestExecutionLevel admin
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

!define ENVKEY "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"
!define UNKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\DroidKraft"

Section "Install"
  SetOutPath "$INSTDIR"
  File "${SRC}\droidkraft-tui.exe"
  File "${SRC}\LICENSE"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  ; add to the machine PATH (once)
  ReadRegStr $0 HKLM "${ENVKEY}" "Path"
  ${StrStr} $1 "$0" "$INSTDIR"
  StrCmp $1 "" 0 +3
    WriteRegExpandStr HKLM "${ENVKEY}" "Path" "$0;$INSTDIR"
    SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  WriteRegStr HKLM "${UNKEY}" "DisplayName" "DroidKraft (terminal)"
  WriteRegStr HKLM "${UNKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${UNKEY}" "Publisher" "sorinirimies"
  WriteRegStr HKLM "${UNKEY}" "UninstallString" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  ReadRegStr $0 HKLM "${ENVKEY}" "Path"
  ${UnStrRep} $0 "$0" ";$INSTDIR" ""
  WriteRegExpandStr HKLM "${ENVKEY}" "Path" "$0"
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
  RMDir /r "$INSTDIR"
  DeleteRegKey HKLM "${UNKEY}"
SectionEnd
