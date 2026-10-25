# P0-06 Runbook — jalankan di VM Windows

## Persiapan

1. Checkpoint VM. Catat `winver`, GPU, DPI, monitor.
2. Salin folder `bin/Release/net10.0-windows10.0.19041.0/` ke VM.

## Prosedur

```
p0-06-blur-bench.exe all
```

20 jendela terbuka (5 transparent, 5 blur, 5 acrylic, 5 solid).

1. **Idle**: diamkan 60 detik. Catat CPU% dan GPU% proses di Task Manager → Details.
2. **Drag**: seret satu jendela acrylic selama 10 detik. Catat CPU/GPU puncak.
3. **Per varian**: tutup semua, jalankan `p0-06-blur-bench.exe transparent` (lalu `blur`, `acrylic`, `solid`), idle 60 detik tiap varian, catat.
4. **High contrast**: nyalakan High Contrast, catat apakah blur mati dan outline muncul.
5. **Win10 vs Win11**: ulangi di kedua OS; catat rounded corner (Win11 DWM vs Win10 clip).

## Tabel hasil

| Varian | Idle CPU% | Idle GPU% | Drag CPU% | Keterangan |
|---|---|---|---|---|
| transparent | | | | |
| blur | | | | |
| acrylic | | | | |
| solid | | | | |

## Keputusan yang diharapkan

- Varian mana yang jadi default, mana yang fallback.
- Apakah `IBlurEffect` dengan 3 implementasi layak, atau solid-colour saja.
- Anggaran idle-CPU untuk 20 Drape (target PR-5: < 0.5% rata-rata).
