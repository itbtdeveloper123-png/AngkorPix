; Inno Setup Script for AngkorPix
; Native Pro Image Editor & Photoshop Alternative

#define MyAppName "AngkorPix"
#define MyAppVersion "0.17.0"
#define MyAppPublisher "ArtPixels / IT BT"
#define MyAppURL "https://github.com/itbtdeveloper123-png/AngkorPix"
#define MyAppExeName "angkorpix.exe"

[Setup]
AppId={{C8A53B2F-93E1-42D0-9154-A61427D6F39A}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\target\installer
OutputBaseFilename=AngkorPix-Setup-v{#MyAppVersion}
SetupIconFile=..\..\assets\app-icon\photocraft.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes
DisableProgramGroupPage=auto

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "assoc_angkorpix"; Description: "Associate .angkorpix files (AngkorPix Native Project)"; GroupDescription: "File associations:"
Name: "assoc_pcraft"; Description: "Associate .pcraft files (Legacy Project Format)"; GroupDescription: "File associations:"; Flags: unchecked
Name: "assoc_psd"; Description: "Associate .psd files (Photoshop Documents)"; GroupDescription: "File associations:"; Flags: unchecked

[Files]
Source: "..\..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\scripts\*"; DestDir: "{app}\scripts"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\..\models\*"; DestDir: "{app}\models"; Flags: ignoreversion recursesubdirs createallsubdirs; Flags: skipifsourcedoesntexist
Source: "..\..\setup-ai-directml.cmd"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\{#MyAppExeName}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon; IconFilename: "{app}\{#MyAppExeName}"

[Registry]
; .angkorpix association (Primary native format)
Root: HKA; Subkey: "Software\Classes\.angkorpix"; ValueType: string; ValueName: ""; ValueData: "AngkorPix.Project"; Flags: uninsdeletevalue; Tasks: assoc_angkorpix
Root: HKA; Subkey: "Software\Classes\AngkorPix.Project"; ValueType: string; ValueName: ""; ValueData: "AngkorPix Native Project"; Flags: uninsdeletekey; Tasks: assoc_angkorpix
Root: HKA; Subkey: "Software\Classes\AngkorPix.Project\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: assoc_angkorpix
Root: HKA; Subkey: "Software\Classes\AngkorPix.Project\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: assoc_angkorpix

; .pcraft association (Legacy native format)
Root: HKA; Subkey: "Software\Classes\.pcraft"; ValueType: string; ValueName: ""; ValueData: "AngkorPix.LegacyProject"; Flags: uninsdeletevalue; Tasks: assoc_pcraft
Root: HKA; Subkey: "Software\Classes\AngkorPix.LegacyProject"; ValueType: string; ValueName: ""; ValueData: "PhotoCraft Project"; Flags: uninsdeletekey; Tasks: assoc_pcraft
Root: HKA; Subkey: "Software\Classes\AngkorPix.LegacyProject\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: assoc_pcraft
Root: HKA; Subkey: "Software\Classes\AngkorPix.LegacyProject\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: assoc_pcraft

; .psd association
Root: HKA; Subkey: "Software\Classes\.psd"; ValueType: string; ValueName: ""; ValueData: "AngkorPix.Psd"; Flags: uninsdeletevalue; Tasks: assoc_psd
Root: HKA; Subkey: "Software\Classes\AngkorPix.Psd"; ValueType: string; ValueName: ""; ValueData: "Photoshop Image Document"; Flags: uninsdeletekey; Tasks: assoc_psd
Root: HKA; Subkey: "Software\Classes\AngkorPix.Psd\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyAppExeName},0"; Tasks: assoc_psd
Root: HKA; Subkey: "Software\Classes\AngkorPix.Psd\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: assoc_psd

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
