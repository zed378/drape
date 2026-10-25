<#
.SYNOPSIS
    Agen uji spike Drape - dijalankan SEKALI di VM Windows.
.DESCRIPTION
    Menjalankan bagian otomatis dari tiap spike, memandu bagian interaktif,
    dan menulis satu file hasil (spike-results.txt) untuk dikirim kembali.

    Otomatis penuh : P0-05 (timing icon), P0-04 (hide/show/restore + kill test).
    Semi-otomatis  : P0-03 (buka window, owner cek Win+D/fokus/Alt+Tab), P0-06
                      (buka 20 window, owner catat CPU/GPU), P0-07 (owner
                      gerakkan mouse, skrip ukur latensi).

    Prasyarat: .NET 10 Desktop Runtime. Checkpoint VM dulu.
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File Run-Spikes.ps1
#>
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$OutFile = Join-Path $Root 'spike-results.txt'
$Stamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'

function Write-Result([string]$Line) {
    $Line | Tee-Object -FilePath $OutFile -Append | Out-Null
    Write-Host $Line
}

function Get-WinVer {
    try { (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion').DisplayVersion }
    catch { (Get-CimInstance Win32_OperatingSystem).BuildNumber }
}

function Get-Dpi {
    try {
        $key = 'HKCU:\Control Panel\Desktop\WindowMetrics'
        $applied = (Get-ItemProperty -Path $key -Name AppliedDPI -ErrorAction Stop).AppliedDPI
        return [int]($applied / 96 * 100)
    } catch {
        return 'unknown'
    }
}

function Get-Monitors {
    (Get-CimInstance -Namespace root\wmi -ClassName WmiMonitorBasicDisplayParams -ErrorAction SilentlyContinue |
        Measure-Object).Count
}

# --- header ---
if (Test-Path $OutFile) { Remove-Item $OutFile }
Write-Result "=== Drape spike results ==="
Write-Result "date: $Stamp"
Write-Result "os: $([Environment]::OSVersion) build: $(Get-WinVer)"
Write-Result "dpi: $(Get-Dpi) monitors: $(Get-Monitors)"
Write-Result ""

# --- P0-05: full auto ---
Write-Result "--- P0-05 shell icons (auto) ---"
$p05 = Join-Path $Root 'p0-05-shell-icons\p0-05-shell-icons.exe'
if (Test-Path $p05) {
    $o = & $p05 --count 500 2>&1 | Out-String
    Write-Result $o.Trim()
} else { Write-Result 'P0-05 SKIP: exe not found' }
Write-Result ''

# --- P0-04: full auto (hide/show/restore + kill test) ---
Write-Result '--- P0-04 desktop icons (auto) ---'
$p04dir = Join-Path $Root 'p0-04-desktop-icons'
$p04 = Join-Path $p04dir 'p0-04-desktop-icons.exe'
if (Test-Path $p04) {
    Write-Result (& $p04 read 2>&1 | Out-String).Trim()
    Write-Result (& $p04 hide 2>&1 | Out-String).Trim()
    Write-Result (& $p04 read 2>&1 | Out-String).Trim()
    Write-Result (& $p04 hide 2>&1 | Out-String).Trim()  # must NOT flip
    # kill test: hide, kill a dummy copy, restore
    Write-Result (& $p04 hide 2>&1 | Out-String).Trim()
    $proc = Start-Process -FilePath $p04 -ArgumentList 'read' -PassThru
    Start-Sleep -Seconds 1
    Stop-Process -Id $proc.Id -Force
    Write-Result (& $p04 --restore 2>&1 | Out-String).Trim()
    Write-Result (& $p04 show 2>&1 | Out-String).Trim()
    Write-Result (& $p04 read 2>&1 | Out-String).Trim()
} else { Write-Result 'P0-04 SKIP: exe not found' }
Write-Result ''

# --- P0-07: semi-auto (needs human mouse) ---
Write-Result '--- P0-07 mouse hook (semi-auto) ---'
$p07 = Join-Path $Root 'p0-07-mouse-hook\p0-07-mouse-hook.exe'
if (Test-Path $p07) {
    Write-Host ''
    Write-Host '>>> P0-07: jendela hook akan dibuka 25 detik.' -ForegroundColor Yellow
    Write-Host '>>> Double-click desktop kosong, double-click sebuah icon,' -ForegroundColor Yellow
    Write-Host '>>> klik kanan (menu harus muncul), right-drag, lalu tunggu selesai.' -ForegroundColor Yellow
    Write-Host '>>> Tekan ENTER untuk mulai...' -ForegroundColor Yellow
    Read-Host | Out-Null
    $job = Start-Job -ScriptBlock {
        param($exe)
        $p = Start-Process -FilePath $exe -PassThru -WindowStyle Normal
        Start-Sleep -Seconds 25
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    } -ArgumentList $p07
    Wait-Job $job | Out-Null
    Receive-Job $job
    Remove-Job $job
    Write-Result '(lihat output hook di atas; salin angka latensi ke bawah ini)'
} else { Write-Result 'P0-07 SKIP: exe not found' }
Write-Result ''

# --- P0-03: semi-auto (needs human eyes) ---
Write-Result '--- P0-03 desktop host (semi-auto) ---'
$p03 = Join-Path $Root 'p0-03-desktop-host\p0-03-desktop-host.exe'
if (Test-Path $p03) {
    Write-Host ''
    Write-Host '>>> P0-03: jendela biru akan dibuka (strategi C).' -ForegroundColor Yellow
    Write-Host '>>> Cek: di atas wallpaper? Win+D 2x tetap terlihat? Peek?' -ForegroundColor Yellow
    Write-Host '>>> Fokus tidak pindah? Alt+Tab tidak ada "P0-03 spike"?' -ForegroundColor Yellow
    Write-Host '>>> Lalu restart Explorer via Task Manager, cek jendela pulih.' -ForegroundColor Yellow
    Write-Host '>>> Tekan ENTER untuk buka, tutup manual setelah selesai...' -ForegroundColor Yellow
    Read-Host | Out-Null
    Start-Process -FilePath $p03 -ArgumentList 'C'
    Write-Host '>>> Ketik hasil (A/B/C lolos-gagal per kriteria), ENTER kosong untuk selesai:' -ForegroundColor Yellow
    $notes = Read-Host 'P0-03 notes'
    Write-Result "owner notes: $notes"
} else { Write-Result 'P0-03 SKIP: exe not found' }
Write-Result ''

# --- P0-06: semi-auto (needs Task Manager) ---
Write-Result '--- P0-06 blur bench (semi-auto) ---'
$p06 = Join-Path $Root 'p0-06-blur-bench\p0-06-blur-bench.exe'
if (Test-Path $p06) {
    Write-Host ''
    Write-Host '>>> P0-06: 20 jendela akan dibuka. Buka Task Manager → Details.' -ForegroundColor Yellow
    Write-Host '>>> Idle 60 detik, catat CPU/GPU. Seret satu jendela, catat puncak.' -ForegroundColor Yellow
    Write-Host '>>> Tekan ENTER untuk buka...' -ForegroundColor Yellow
    Read-Host | Out-Null
    Start-Process -FilePath $p06 -ArgumentList 'all'
    Write-Host '>>> Ketik angka (idle CPU/GPU, drag puncak per varian), ENTER kosong untuk selesai:' -ForegroundColor Yellow
    $notes = Read-Host 'P0-06 notes'
    Write-Result "owner notes: $notes"
} else { Write-Result 'P0-06 SKIP: exe not found' }
Write-Result ''
Write-Result '=== selesai. Kirim file spike-results.txt ini kembali. ==='
Write-Host ''
Write-Host "Hasil tersimpan di: $OutFile" -ForegroundColor Green
