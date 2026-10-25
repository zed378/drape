# P0-07 Runbook — jalankan di VM Windows

## Persiapan

1. Checkpoint VM. Catat `winver`, DPI.
2. Salin folder `bin/Release/net10.0-windows10.0.19041.0/` ke VM.

## Prosedur

```
p0-07-mouse-hook.exe
```

1. Double-click area kosong desktop → harus muncul `L-dblclk desktop`.
2. Double-click sebuah icon → harus muncul `L-dblclk other` (bukan desktop).
3. Klik kanan biasa → menu Windows tetap muncul; event `R-down other`/`R-up` tercatat.
4. Right-drag di desktop kosong → tidak merusak menu; event tercatat.
5. Tekan Esc → statistik latensi tercetak.

## Yang dicatat

- Tabel event (copy dari output).
- Latensi: avg, p99, max (target avg < 1 ms).
- Apakah ada peringatan antivirus saat menjalankan.
- Apakah klik kanan tetap memunculkan menu Windows.

## Hasil yang dikirim kembali

Output lengkap + `winver` + catatan AV.
