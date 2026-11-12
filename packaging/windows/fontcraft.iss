; FontCraft installer for Windows x64 and Windows on ARM (ARM64) (Inno Setup 6).
; Build:  iscc /DArch=x64   /DExeDir=..\..\target\x86_64-pc-windows-msvc\release  packaging\windows\fontcraft.iss
;         iscc /DArch=arm64 /DExeDir=..\..\target\aarch64-pc-windows-msvc\release packaging\windows\fontcraft.iss
;         (compilar-windows.bat does this). Without /D values: x64, from target\release.
; Result: dist\FontCraft-<version>-windows-<arch>-setup.exe

#define AppVersion GetEnv("FONTCRAFT_VERSION")
#if AppVersion == ""
  #define AppVersion "0.1.0"
#endif
#ifndef Arch
  #define Arch "x64"
#endif
#ifndef ExeDir
  #define ExeDir "..\..\target\release"
#endif

[Setup]
AppId={{6C1E2B7A-4D0F-4E5B-9F3A-FC0FC0FC0001}
AppName=FontCraft
AppVersion={#AppVersion}
AppPublisher=FontCraft
DefaultDirName={autopf}\FontCraft
DefaultGroupName=FontCraft
DisableProgramGroupPage=yes
; Install for everyone (asks for administrator) or only for this user.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
#if Arch == "arm64"
; Windows on ARM only (the program is built for ARM64).
; Windows 10 22H2 (2022) or newer, and Windows 11: systems from the last five years.
MinVersion=10.0.19045
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#else
MinVersion=10.0.19045
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#endif
OutputDir=..\..\dist
OutputBaseFilename=FontCraft-{#AppVersion}-windows-{#Arch}-setup
SetupIconFile=..\..\assets\icons\fontcraft.ico
UninstallDisplayIcon={app}\fontcraft.exe
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
ChangesAssociations=yes

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "italian"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "portuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#ExeDir}\fontcraft.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\icons\fontcraft.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\icons\fontcraft-library.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\FontCraft"; Filename: "{app}\fontcraft.exe"
Name: "{group}\{cm:UninstallProgram,FontCraft}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\FontCraft"; Filename: "{app}\fontcraft.exe"; Tasks: desktopicon

[Registry]
; FontCraft libraries (.fcl) open with FontCraft and show its document icon.
Root: HKA; Subkey: "Software\Classes\.fcl"; ValueType: string; ValueName: ""; ValueData: "FontCraft.Library"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\FontCraft.Library"; ValueType: string; ValueName: ""; ValueData: "FontCraft Library"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\FontCraft.Library\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\fontcraft-library.ico"
Root: HKA; Subkey: "Software\Classes\FontCraft.Library\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\fontcraft.exe"" ""%1"""
; Remove the "start with Windows" entry FontCraft may have added.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "FontCraft"; Flags: uninsdeletevalue dontcreatekey

[Run]
Filename: "{app}\fontcraft.exe"; Description: "{cm:LaunchProgram,FontCraft}"; Flags: nowait postinstall skipifsilent
