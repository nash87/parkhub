; ParkHub NSIS Installer Script
; Build with: makensis parkhub.nsi

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "nsDialogs.nsh"

; General
Name "ParkHub"
OutFile "ParkHub-Setup.exe"
InstallDir "$PROGRAMFILES\ParkHub"
InstallDirRegKey HKLM "Software\ParkHub" "InstallDir"
RequestExecutionLevel admin

; Version info
!define VERSION "1.0.0"
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "ParkHub"
VIAddVersionKey "FileDescription" "ParkHub Parking Server Installer"
VIAddVersionKey "LegalCopyright" "MIT License"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"

; Variables
Var InstallService
Var CreateShortcut
Var AddFirewall

; Interface
!define MUI_ABORTWARNING
!define MUI_ICON "..\..\assets\icon.ico"
!define MUI_UNICON "..\..\assets\icon.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
Page custom OptionsPage OptionsPageLeave
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Language
!insertmacro MUI_LANGUAGE "English"

; Options page
Function OptionsPage
    nsDialogs::Create 1018
    Pop $0

    ${NSD_CreateCheckbox} 0 0u 100% 12u "Install as Windows Service (auto-start)"
    Pop $InstallService
    ${NSD_Check} $InstallService

    ${NSD_CreateCheckbox} 0 20u 100% 12u "Create Desktop shortcut"
    Pop $CreateShortcut
    ${NSD_Check} $CreateShortcut

    ${NSD_CreateCheckbox} 0 40u 100% 12u "Add firewall rule (port 7878)"
    Pop $AddFirewall
    ${NSD_Check} $AddFirewall

    nsDialogs::Show
FunctionEnd

Function OptionsPageLeave
    ${NSD_GetState} $InstallService $InstallService
    ${NSD_GetState} $CreateShortcut $CreateShortcut
    ${NSD_GetState} $AddFirewall $AddFirewall
FunctionEnd

; Installer
Section "ParkHub Server" SecMain
    SectionIn RO
    SetOutPath "$INSTDIR"

    ; Copy files
    File "parkhub-server.exe"

    ; Create data directory
    CreateDirectory "$LOCALAPPDATA\ParkHub\data"

    ; Write uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Registry
    WriteRegStr HKLM "Software\ParkHub" "InstallDir" "$INSTDIR"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ParkHub" "DisplayName" "ParkHub"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ParkHub" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ParkHub" "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ParkHub" "Publisher" "ParkHub"

    ; Start Menu
    CreateDirectory "$SMPROGRAMS\ParkHub"
    CreateShortcut "$SMPROGRAMS\ParkHub\ParkHub Server.lnk" "$INSTDIR\parkhub-server.exe"
    CreateShortcut "$SMPROGRAMS\ParkHub\Uninstall.lnk" "$INSTDIR\Uninstall.exe"

    ; Optional: Install as service
    ${If} $InstallService == ${BST_CHECKED}
        nsExec::ExecToLog '"$INSTDIR\parkhub-server.exe" install'
        nsExec::ExecToLog 'sc start ParkHub'
    ${EndIf}

    ; Optional: Desktop shortcut
    ${If} $CreateShortcut == ${BST_CHECKED}
        CreateShortcut "$DESKTOP\ParkHub Server.lnk" "$INSTDIR\parkhub-server.exe"
    ${EndIf}

    ; Optional: Firewall rule
    ${If} $AddFirewall == ${BST_CHECKED}
        nsExec::ExecToLog 'netsh advfirewall firewall add rule name="ParkHub Server" dir=in action=allow protocol=TCP localport=7878'
    ${EndIf}

    ; Add to PATH
    EnVar::AddValue "PATH" "$INSTDIR"
SectionEnd

; Uninstaller
Section "Uninstall"
    ; Stop and remove service
    nsExec::ExecToLog 'sc stop ParkHub'
    nsExec::ExecToLog '"$INSTDIR\parkhub-server.exe" uninstall'

    ; Remove firewall rule
    nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="ParkHub Server"'

    ; Remove files
    Delete "$INSTDIR\parkhub-server.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"

    ; Remove shortcuts
    Delete "$DESKTOP\ParkHub Server.lnk"
    Delete "$SMPROGRAMS\ParkHub\ParkHub Server.lnk"
    Delete "$SMPROGRAMS\ParkHub\Uninstall.lnk"
    RMDir "$SMPROGRAMS\ParkHub"

    ; Remove from PATH
    EnVar::DeleteValue "PATH" "$INSTDIR"

    ; Registry cleanup
    DeleteRegKey HKLM "Software\ParkHub"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\ParkHub"
SectionEnd
