# P0-04 Runbook — jalankan di VM Windows SAJA

> Historical C#/.NET/Avalonia baseline, frozen 2026-11-25. Commands, task states and measurements below describe legacy work only; they do not prove Rust/Slint behaviour. Rust replacement work uses fresh R0/R1 IDs in [Rust migration tasks](../../TASKS/RUST-MIGRATION.md). No code was ported or removed by the documentation update.


> Jangan jalankan di host. Checkpoint VM dulu.

## Persiapan

1. Checkpoint VM. Catat `winver`, DPI, monitor.
2. Salin folder `bin/Release/net10.0-windows10.0.19041.0/` ke VM.

## Prosedur

```
p0-04-desktop-icons.exe read     → catat: hidden=false (awal)
p0-04-desktop-icons.exe hide     → ikon hilang? catat
p0-04-desktop-icons.exe read     → hidden=true?
p0-04-desktop-icons.exe hide     → "already hidden; no toggle sent" (tidak flip!)
p0-04-desktop-icons.exe show     → ikon kembali? catat
```

## Kill test (recovery)

```
p0-04-desktop-icons.exe hide
taskkill /F /IM p0-04-desktop-icons.exe
p0-04-desktop-icons.exe --restore   → ikon kembali? catat
```

## Explorer restart

```
p0-04-desktop-icons.exe hide
Task Manager → restart Windows Explorer (manual)
p0-04-desktop-icons.exe read     → masih hidden=true? (tidak flip)
p0-04-desktop-icons.exe show     → ikon kembali?
```

## Alternatif SW_HIDE (opsional, jika WM_COMMAND gagal)

Sembunyikan via Spy++ (`ShowWindow(SW_HIDE)` pada SysListView32), restart Explorer manual,
catat apakah Explorer menampilkannya lagi.

## Hasil yang dikirim kembali

`winver` + tabel hide/show/read + hasil kill test + hasil Explorer restart.
