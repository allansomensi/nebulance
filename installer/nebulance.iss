; Nebulance installer (Inno Setup 6 — https://jrsoftware.org/isinfo.php)
;
; How to build:
;   1. On Windows, build the game:   cargo build --release
;   2. Open this file in Inno Setup and click "Compile" (or run: iscc installer\nebulance.iss)
;   3. The installer is written to dist\Nebulance-0.1.0-setup.exe
;
; If you built the game another way (e.g. cross-compiling), adjust ExePath below.

#define AppName "Nebulance"
#define AppVersion "0.1.0"
#define AppPublisher "Allan Somensi"
#ifndef ExePath
  #define ExePath "..\target\release\nebulance.exe"
#endif

[Setup]
AppId={{95E2EF4F-1850-4D15-B469-7B7BA438C8A8}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir=..\dist
OutputBaseFilename=Nebulance-{#AppVersion}-setup
SetupIconFile=..\assets\nebulance.ico
UninstallDisplayIcon={app}\nebulance.exe
UninstallDisplayName={#AppName}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#ExePath}"; DestDir: "{app}"; DestName: "nebulance.exe"; Flags: ignoreversion
Source: "..\README.txt"; DestDir: "{app}"; Flags: ignoreversion isreadme; Languages: english
Source: "..\README.txt"; DestDir: "{app}"; Flags: ignoreversion; Languages: brazilianportuguese
Source: "..\LEIAME.txt"; DestDir: "{app}"; Flags: ignoreversion isreadme; Languages: brazilianportuguese
Source: "..\LEIAME.txt"; DestDir: "{app}"; Flags: ignoreversion; Languages: english
Source: "..\assets\fonts\OFL-*.txt"; DestDir: "{app}\licenses"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\nebulance.exe"; WorkingDir: "{app}"
Name: "{group}\Readme"; Filename: "{app}\README.txt"
Name: "{group}\Leia-me"; Filename: "{app}\LEIAME.txt"
Name: "{group}\{cm:UninstallProgram,{#AppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\nebulance.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\nebulance.exe"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
