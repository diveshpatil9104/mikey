; Mikey Windows installer (Inno Setup 6). Installs mikey.exe and Mikey Mic, the virtual microphone,
; opens the firewall on private networks for TCP 7653 and UDP 7654, and can start Mikey at sign-in.
;
; Build: ISCC /DMyAppVersion=x.y.z mikey.iss, with mikey.exe built and the Mikey Mic driver files in
; installer\driver (the CI workflow windows-installer.yml does both).

#ifndef MyAppVersion
  #define MyAppVersion "0.1.0"
#endif
#ifndef MikeyExe
  #define MikeyExe "..\target\release\mikey.exe"
#endif
#define MyAppName "Mikey"
#define MyAppPublisher "Mikey Contributors"
#define MyAppURL "https://github.com/diveshpatil9104/mikey"
#define MyAppExeName "mikey.exe"

[Setup]
AppId={{9F3B6E8C-8F74-4C75-A1E2-93D0F8C56A10}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
OutputDir=Output
OutputBaseFilename=Mikey-Setup-{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
MinVersion=10.0
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Messages]
WelcomeLabel2=This installs [name/ver] and Mikey Mic, so your Android phone can be the microphone and webcam in Meet, Zoom, Teams and any other app.%n%nIt's best to close other apps before continuing.
FinishedLabelNoIcons=Mikey is installed. Open Mikey on your phone, connect, and pick Mikey Mic and Mikey Cam in your meeting app.
FinishedLabel=Mikey is installed. Open Mikey on your phone, connect, and pick Mikey Mic and Mikey Cam in your meeting app.
FinishedRestartLabel=Windows needs to restart to finish setting up Mikey Mic. Restart now?

[Tasks]
Name: "autostart"; Description: "Start Mikey when I sign in to Windows"; GroupDescription: "Startup:"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#MikeyExe}"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion
Source: "..\softcam.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "setup-audio-device.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "THIRD-PARTY-NOTICES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "driver\*"; DestDir: "{app}\driver"; Flags: ignoreversion recursesubdirs

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Mikey"; ValueData: """{app}\{#MyAppExeName}"" --autostart"; Tasks: autostart; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "Mikey"; Flags: dontcreatekey uninsdeletevalue

[Run]
Filename: "regsvr32.exe"; Parameters: "/s ""{app}\softcam.dll"""; StatusMsg: "Registering virtual camera..."; Flags: runhidden
Filename: "netsh"; Parameters: "advfirewall firewall add rule name=""Mikey TCP"" dir=in action=allow protocol=TCP localport=7653 profile=any"; StatusMsg: "Letting your phone reach Mikey..."; Flags: runhidden
Filename: "netsh"; Parameters: "advfirewall firewall add rule name=""Mikey UDP Beacon"" dir=in action=allow protocol=UDP localport=7654 profile=any"; StatusMsg: "Letting your phone find Mikey..."; Flags: runhidden
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "taskkill.exe"; Parameters: "/F /IM {#MyAppExeName}"; Flags: runhidden; RunOnceId: "MikeyKill"
Filename: "regsvr32.exe"; Parameters: "/u /s ""{app}\softcam.dll"""; Flags: runhidden; RunOnceId: "MikeySoftcam"
Filename: "netsh"; Parameters: "advfirewall firewall delete rule name=""Mikey TCP"""; Flags: runhidden; RunOnceId: "MikeyFirewallTcp"
Filename: "netsh"; Parameters: "advfirewall firewall delete rule name=""Mikey UDP Beacon"""; Flags: runhidden; RunOnceId: "MikeyFirewallUdp"

[Code]
var
  MicNeedsRestart: Boolean;

// Sets up Mikey Mic after the files are in place. The script restores the user's own default
// speakers and microphone, and exits 3010 when Windows has to restart to finish.
procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
begin
  if CurStep <> ssPostInstall then
    Exit;
  WizardForm.StatusLabel.Caption := 'Setting up Mikey Mic...';
  WizardForm.FilenameLabel.Caption := 'Configuring virtual audio driver and endpoints (this can take up to a minute)...';
  WizardForm.ProgressGauge.Style := npbstMarquee;
  try
    if Exec(ExpandConstant('{sys}\WindowsPowerShell\v1.0\powershell.exe'),
        '-NoProfile -ExecutionPolicy Bypass -File "' + ExpandConstant('{app}\setup-audio-device.ps1') + '" -Silent',
        '', SW_HIDE, ewWaitUntilTerminated, ResultCode) and ((ResultCode = 0) or (ResultCode = 3010)) then
      MicNeedsRestart := ResultCode = 3010
    else
      SuppressibleMsgBox('Mikey is installed, but Mikey Mic couldn''t be set up yet. Restart Windows, open Mikey, and click Setup Mic in its panel.',
        mbInformation, MB_OK, IDOK);
  finally
    WizardForm.ProgressGauge.Style := npbstNormal;
    WizardForm.FilenameLabel.Caption := '';
  end;
end;

function NeedRestart(): Boolean;
begin
  Result := MicNeedsRestart;
end;
