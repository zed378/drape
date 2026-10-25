<#
.SYNOPSIS
    Deploy + uji spike Drape ke VM via SSH. Dijalankan dari mesin dev.
.DESCRIPTION
    1. Salin paket spike ke VM via SCP.
    2. Jalankan bagian otomatis (P0-05 timing, P0-04 read/hide/show/restore)
       via SSH, hasilnya disimpan ke spike-results-auto.txt.
    3. Bagian interaktif (P0-03/06/07) TETAP butuh owner di VM — skrip hanya
       menyiapkan binary-nya dan mencetak instruksi.

    Prasyarat: OpenSSH Server aktif di VM, port 22 terbuka.
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File runs/Deploy-And-Test.ps1 -VmHost 10.1.200.32 -VmUser zawawi
#>
param(
    [string]$VmHost = '10.1.200.32',
    [string]$VmUser = 'zawawi',
    [string]$VmDir = 'C:/DrapeSpikes'
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$OutFile = Join-Path $Root "spike-results-auto-$Stamp.txt"

function Ssh([string]$Command) {
    ssh -o StrictHostKeyChecking=accept-new -o ConnectTimeout=15 "$VmUser@$VmHost" $Command
}

function Scp-To([string]$Local, [string]$Remote) {
    scp -o StrictHostKeyChecking=accept-new -r $Local "$VmUser@${VmHost}:$Remote"
}

function Scp-From([string]$Remote, [string]$Local) {
    scp -o StrictHostKeyChecking=accept-new -r "$VmUser@${VmHost}:$Remote" $Local
}

Write-Host '== 1/4 koneksi SSH ==' -ForegroundColor Cyan
Ssh 'powershell -NoProfile -Command "whoami; [Environment]::OSVersion; (Get-ItemProperty ''HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'').DisplayVersion"'

Write-Host '== 2/4 salin paket ==' -ForegroundColor Cyan
Ssh "powershell -NoProfile -Command `"New-Item -ItemType Directory -Force -Path '$VmDir' | Out-Null`""
foreach ($s in @('p0-03-desktop-host', 'p0-04-desktop-icons', 'p0-05-shell-icons', 'p0-06-blur-bench', 'p0-07-mouse-hook')) {
    $bin = Join-Path $Root "..\spikes\$s\bin\Release\net10.0-windows10.0.19041.0"
    $runbook = Join-Path $Root "..\spikes\$s\RUNBOOK.md"
    Scp-To $bin "$VmDir/$s/"
    Scp-To $runbook "$VmDir/$s/"
}
Scp-To (Join-Path $Root 'Run-Spikes.ps1') "$VmDir/"

Write-Host '== 3/4 uji otomatis via SSH ==' -ForegroundColor Cyan
$auto = @'
$out = @()
$out += "date: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
$out += "os: $([Environment]::OSVersion)"
$out += "build: $((Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion').DisplayVersion)"
$out += "dpi: $((Get-ItemProperty -Path 'HKCU:\Control Panel\Desktop\WindowMetrics' -Name AppliedDPI -ErrorAction SilentlyContinue).AppliedDPI)"
$out += "monitors: $((Get-CimInstance -Namespace root/wmi -ClassName WmiMonitorBasicDisplayParams -ErrorAction SilentlyContinue | Measure-Object).Count)"
$out += "--- P0-05 ---"
$out += (& C:/DrapeSpikes/p0-05-shell-icons/p0-05-shell-icons.exe --count 500 2>&1 | Out-String).Trim()
$out += "--- P0-04 ---"
$out += (& C:/DrapeSpikes/p0-04-desktop-icons/p0-04-desktop-icons.exe read 2>&1 | Out-String).Trim()
$out += (& C:/DrapeSpikes/p0-04-desktop-icons/p0-04-desktop-icons.exe hide 2>&1 | Out-String).Trim()
$out += (& C:/DrapeSpikes/p0-04-desktop-icons/p0-04-desktop-icons.exe read 2>&1 | Out-String).Trim()
$out += (& C:/DrapeSpikes/p0-04-desktop-icons/p0-04-desktop-icons.exe hide 2>&1 | Out-String).Trim()
$out += (& C:/DrapeSpikes/p0-04-desktop-icons/p0-04-desktop-icons.exe --restore 2>&1 | Out-String).Trim()
$out += (& C:/DrapeSpikes/p0-04-desktop-icons/p0-04-desktop-icons.exe show 2>&1 | Out-String).Trim()
$out += (& C:/DrapeSpikes/p0-04-desktop-icons/p0-04-desktop-icons.exe read 2>&1 | Out-String).Trim()
$out -join "`n"
'@
$result = Ssh "powershell -NoProfile -Command `"$auto`""
$result | Tee-Object -FilePath $OutFile | Out-Null
Write-Host $result

Write-Host '== 4/4 selesai ==' -ForegroundColor Green
Write-Host "Hasil otomatis: $OutFile"
Write-Host 'Bagian interaktif (P0-03/06/07) tetap butuh owner di VM via RDP:'
Write-Host '  powershell -ExecutionPolicy Bypass -File C:/DrapeSpikes/Run-Spikes.ps1'
