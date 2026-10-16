; Built by scripts/ci/package_windows.sh with -DVERSION -DSRC -DOUT
!include "MUI2.nsh"
Name "DroidKraft"
OutFile "${OUT}"
InstallDir "$PROGRAMFILES64\DroidKraft"
RequestExecutionLevel admin
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"
Section "Install"
  SetOutPath "$INSTDIR"
  File /r "${SRC}\*.*"
  CreateDirectory "$SMPROGRAMS\DroidKraft"
  CreateShortcut "$SMPROGRAMS\DroidKraft\DroidKraft.lnk" "$INSTDIR\droidkraft.exe" "" "$INSTDIR\droidkraft.ico"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\DroidKraft" "DisplayName" "DroidKraft"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\DroidKraft" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\DroidKraft" "UninstallString" "$INSTDIR\uninstall.exe"
SectionEnd
Section "Uninstall"
  RMDir /r "$INSTDIR"
  RMDir /r "$SMPROGRAMS\DroidKraft"
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\DroidKraft"
SectionEnd
