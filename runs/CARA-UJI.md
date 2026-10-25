# Cara Uji Spike di VM Windows

> Salin SATU file `drape-spikes-full.zip` (±272 MB, self-contained — TIDAK butuh
> instal .NET runtime) ke VM, ekstrak, lalu jalankan SEKALI:
>
> ```
> powershell -ExecutionPolicy Bypass -File Run-Spikes.ps1
> ```
>
> Skrip menjalankan bagian otomatis (P0-05 timing, P0-04 hide/show/restore+kill),
> memandu bagian interaktif (P0-03/06/07), dan menulis `spike-results.txt`.
> Kirim file itu kembali. Checkpoint VM dulu. Catat `winver`, DPI, monitor.
>
> Isi ZIP: 5 folder spike (`p0-03` s/d `p0-07`, exe self-contained + RUNBOOK),
> `Run-Spikes.ps1`, dan file ini sebagai README. Semua `.exe` Release.

## Urutan

### 1. P0-03 — Window host (`p0-03-desktop-host-vm.zip`, ~179 MB)

```
p0-03-desktop-host.exe C
```

Jendela biru muncul. Isi tabel di RUNBOOK (di dalam ZIP): wallpaper-atas/bawah-app,
Win+D, peek, fokus, Alt+Tab, Explorer restart (manual via Task Manager), DPI 100/150/200%.

### 2. P0-04 — Icons (`p0-04-desktop-icons-vm.zip`, ~6 MB) — HATI-HATI, menyembunyikan icon

```
p0-04-desktop-icons.exe read
p0-04-desktop-icons.exe hide     → icon hilang?
p0-04-desktop-icons.exe read     → hidden=true?
p0-04-desktop-icons.exe hide     → "already hidden" (tidak flip!)
p0-04-desktop-icons.exe show     → icon kembali?
```

Kill test: `hide`, lalu `taskkill /F /IM p0-04-desktop-icons.exe`, lalu `--restore` → icon kembali?

### 3. P0-05 — Icons speed (`p0-05-shell-icons-vm.zip`, ~6 MB)

Siapkan 500 file campuran di Desktop dulu (atau gunakan `--count 500`).

```
p0-05-shell-icons.exe --count 500
```

Catat: elapsed ms, ok/failed. Cek file OneDrive placeholder TIDAK ter-download.
Jawab: overlay hilang? Bitmap Avalonia bisa dibuat di luar UI thread?

### 4. P0-06 — Blur (`p0-06-blur-bench-vm.zip`, ~179 MB)

```
p0-06-blur-bench.exe all
```

20 jendela terbuka. Idle 60 detik → catat CPU/GPU. Seret satu jendela → catat puncak.
Ulangi per varian (`transparent`, `blur`, `acrylic`, `solid`). Cek high contrast + Win10 vs Win11.

### 5. P0-07 — Mouse hook (`p0-07-mouse-hook-vm.zip`, ~6 MB)

```
p0-07-mouse-hook.exe
```

Double-click desktop kosong vs icon, klik kanan (menu muncul?), right-drag, lalu Esc.
Catat tabel event + latensi avg/p99/max (target avg < 1 ms) + reaksi antivirus.

## Syarat runtime

ZIP framework-dependent: butuh .NET 10 runtime di VM. Jika belum ada, instal dari
https://dotnet.microsoft.com/download/dotnet/10.0 (Desktop Runtime, ~100 MB, sekali saja).
Jika tidak boleh instal, beri tahu saya — saya publish ulang sebagai self-contained.

## Yang dikirim kembali ke saya

Untuk tiap spike: tabel terisi + `winver` + DPI + monitor + catatan. Teks biasa cukup:

```
P0-03: Win11 24H2 build 26100, 150%, 1 monitor. A: ... B: ... C: ...
P0-04: hide OK, show OK, no-flip OK, kill-restore OK, explorer-restart OK.
P0-05: 500 item, 812 ms, no hydration, overlay hilang, bitmap-thread: ...
P0-06: acrylic idle 1.2% CPU ... dst.
P0-07: avg 0.04 ms, p99 0.3 ms, AV diam, menu OK.
```
