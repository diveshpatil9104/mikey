# Mikey Mic setup: installs the virtual microphone driver if it's missing, names it "Mikey Mic",
# and leaves the user's own default speakers and microphone exactly as they were.
#
# Run by the installer (-Silent) and by the panel's Setup Mic button. Needs administrator rights.
# The driver files come from -DriverDir, by default the "driver" folder next to this script.
# Exit codes: 0 ready, 3010 ready after Windows restarts, 1 failed.

param (
    [switch]$Silent = $false,
    [string]$DriverDir = ""
)

$ErrorActionPreference = "Stop"
if (-not $DriverDir) { $DriverDir = Join-Path $PSScriptRoot "driver" }

function Say([string]$text, [string]$color = "Cyan") {
    if (-not $Silent) { Write-Host "[mikey] $text" -ForegroundColor $color }
}

function Finish([int]$code) {
    if (-not $Silent) { Read-Host "Press Enter to close" | Out-Null }
    exit $code
}

$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    if ($Silent) { exit 1 }
    Say "Setting up Mikey Mic needs administrator rights. Asking Windows..." "Yellow"
    $elevated = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -DriverDir `"$DriverDir`""
    try {
        $proc = Start-Process powershell.exe -ArgumentList $elevated -Verb RunAs -PassThru -Wait
        exit $proc.ExitCode
    } catch {
        Say "Windows didn't allow it, so Mikey Mic wasn't set up." "Yellow"
        exit 1
    }
}

# Mikey Mic runs on this virtual cable driver. Its device and endpoint names as Windows reports them.
$DriverDevice = "VB-Audio Virtual Cable"
$MikeyAudio = "Mikey Audio"
$EndpointName = "{a45c254e-df1c-4efd-8020-67d146a850e0},2"   # PKEY_Device_DeviceDesc: "CABLE Output"
$AdapterName  = "{b3f8fa53-0004-438e-9003-51a46e139bfc},6"   # PKEY_DeviceInterface_FriendlyName: the part in brackets

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

[ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
class MMDeviceEnumerator {}

[ComImport, Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDeviceEnumerator {
    int EnumAudioEndpoints();
    [PreserveSig] int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice device);
}

[ComImport, Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDevice {
    int Activate();
    int OpenPropertyStore();
    [PreserveSig] int GetId([MarshalAs(UnmanagedType.LPWStr)] out string id);
}

[ComImport, Guid("F8679F50-850A-41CF-9C72-430F290290C8"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IPolicyConfig {
    int GetMixFormat(); int GetDeviceFormat(); int ResetDeviceFormat(); int SetDeviceFormat();
    int GetProcessingPeriod(); int SetProcessingPeriod(); int GetShareMode(); int SetShareMode();
    int GetPropertyValue(); int SetPropertyValue();
    [PreserveSig] int SetDefaultEndpoint([MarshalAs(UnmanagedType.LPWStr)] string id, int role);
}

[ComImport, Guid("870AF99C-171D-4F9E-AF0D-E63DF40C2BC9")]
class PolicyConfigClient {}

public static class MikeyDefaults {
    // flow 0 = speakers, 1 = microphones; role 0 = console, 1 = multimedia, 2 = communications
    public static string Get(int flow, int role) {
        IMMDevice device;
        var enumerator = (IMMDeviceEnumerator)new MMDeviceEnumerator();
        if (enumerator.GetDefaultAudioEndpoint(flow, role, out device) != 0 || device == null) return null;
        string id;
        return device.GetId(out id) == 0 ? id : null;
    }
    public static void Set(string id, int role) {
        ((IPolicyConfig)new PolicyConfigClient()).SetDefaultEndpoint(id, role);
    }
}
'@

# 1. Remember the user's default speakers and microphone, for every role.
$saved = @()
foreach ($flow in 0, 1) {
    foreach ($role in 0, 1, 2) {
        $id = [MikeyDefaults]::Get($flow, $role)
        if ($id) { $saved += [pscustomobject]@{ Id = $id; Role = $role } }
    }
}

# Installing the driver can make it the default device; put the user's own ones back.
function Restore-Defaults {
    foreach ($d in $saved) {
        try { [void][MikeyDefaults]::Set($d.Id, $d.Role) } catch {}
    }
}

# 2. Install the driver if it isn't there.
function Test-Driver {
    $dev = Get-PnpDevice -Class Media -ErrorAction SilentlyContinue | Where-Object { $_.FriendlyName -eq $DriverDevice }
    return [bool]($dev | Where-Object { $_.Status -eq "OK" })
}

$restartNeeded = $false
if (Test-Driver) {
    Say "Mikey Mic's driver is already installed." "Green"
} else {
    $setup = Join-Path $DriverDir "VBCABLE_Setup_x64.exe"
    if (-not (Test-Path $setup)) {
        Say "Mikey Mic's driver files are missing. Reinstall Mikey to set it up." "Yellow"
        Finish 1
    }
    Say "Installing Mikey Mic. This can take a minute..."
    $proc = Start-Process -FilePath $setup -ArgumentList "-i", "-h" -WorkingDirectory $DriverDir -PassThru
    if (-not $proc.WaitForExit(180000)) {
        Restore-Defaults
        Say "Installing Mikey Mic took too long." "Yellow"
        Finish 1
    }
    $restartNeeded = $true
}

# 3. Name it Mikey Mic. Its endpoints appear shortly after the driver installs.
function Rename-Endpoints([string]$flow, [string]$name) {
    $root = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\$flow"
    $found = 0
    Get-ChildItem -Path $root -ErrorAction SilentlyContinue | ForEach-Object {
        $props = Join-Path $_.PSPath "Properties"
        $values = Get-ItemProperty -Path $props -ErrorAction SilentlyContinue
        if ($values -and ($values.$AdapterName -eq $DriverDevice -or $values.$AdapterName -eq $MikeyAudio)) {
            Set-ItemProperty -Path $props -Name $EndpointName -Value $name
            Set-ItemProperty -Path $props -Name $AdapterName -Value $MikeyAudio
            $found++
        }
    }
    return $found
}

try {
    $deadline = (Get-Date).AddSeconds(30)
    do {
        $mics = Rename-Endpoints "Capture" "Mikey Mic"
        if ($mics -gt 0) { break }
        Start-Sleep -Seconds 2
    } while ((Get-Date) -lt $deadline)
    [void](Rename-Endpoints "Render" "Mikey Mic Bridge")
} catch {
    Restore-Defaults
    Say "Couldn't name the microphone Mikey Mic: $($_.Exception.Message)" "Yellow"
    Finish 1
}

if ($mics -eq 0) {
    Restore-Defaults
    if ($restartNeeded) {
        Say "Mikey Mic is installed. Restart Windows, then click Setup Mic in Mikey's panel to finish." "Yellow"
        Finish 3010
    }
    Say "Mikey Mic's driver is installed, but Windows hasn't created the microphone yet. Restart Windows and try again." "Yellow"
    Finish 1
}

# 4. Apply the names, then put the user's defaults back.
try {
    Restart-Service -Name "Audiosrv" -Force
    Start-Sleep -Seconds 2
} catch {
    Say "The new name shows after Windows restarts." "Yellow"
}
Restore-Defaults

Say "Mikey Mic is ready. Pick it as the microphone in Meet, Zoom or Teams." "Green"
if ($restartNeeded) { Finish 3010 }
Finish 0
