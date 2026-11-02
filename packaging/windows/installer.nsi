; Generic NSIS installer. Build with (see packaging/windows/build_installer.sh):
;   makensis -DAPP_NAME=.. -DSLUG=.. -DVERSION=1.2.3 -DROOT=C:\repo \
;            -DBINDIR=target\release -DCLI_EXE=x.exe [-DGUI_EXE=y.exe]
Unicode true
!include "MUI2.nsh"

Name "${APP_NAME} ${VERSION}"
OutFile "${ROOT}\dist\${SLUG}-v${VERSION}-windows-x86_64-setup.exe"
InstallDir "$PROGRAMFILES64\${APP_NAME}"
InstallDirRegKey HKLM "Software\${APP_NAME}" "InstallDir"
RequestExecutionLevel admin

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetRegView 64
  SetOutPath "$INSTDIR"
  File "${ROOT}\${BINDIR}\${CLI_EXE}"
  !ifdef GUI_EXE
    File "${ROOT}\${BINDIR}\${GUI_EXE}"
    CreateShortcut "$SMPROGRAMS\${APP_NAME}.lnk" "$INSTDIR\${GUI_EXE}"
  !endif
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\${APP_NAME}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "NoModify" 1
SectionEnd

Section "Uninstall"
  SetRegView 64
  Delete "$INSTDIR\${CLI_EXE}"
  !ifdef GUI_EXE
    Delete "$INSTDIR\${GUI_EXE}"
    Delete "$SMPROGRAMS\${APP_NAME}.lnk"
  !endif
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "Software\${APP_NAME}"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"
SectionEnd
