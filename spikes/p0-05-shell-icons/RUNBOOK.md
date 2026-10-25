# P0-05 Runbook — jalankan di VM Windows

## Persiapan

1. Siapkan folder uji di Desktop VM: 500 file campuran (`.txt`, `.png`, `.lnk`, beberapa folder).
   Jika Desktop asli sparse, gunakan `--count 500` untuk mengulang item yang ada.
2. Jika ada OneDrive Files On-Demand: tandai satu file sebagai placeholder (cloud-only).
3. Catat `winver`, DPI, monitor.

## Jalankan

```
p0-05-shell-icons.exe --count 500
```

Catat: jumlah item, ok/failed, elapsed ms.

## Yang diamati manual

1. **Kecepatan**: elapsed untuk 500 item (target < 1 detik, icon async).
2. **Hidrasi**: file placeholder OneDrive TIDAK boleh ter-download (cek status cloud tetap).
3. **Overlay**: apakah shortcut arrow / status cloud hilang (keterbatasan IShellItemImageFactory)?
4. **Kualitas**: icon 48px tajam atau pecah?

## Pertanyaan yang harus dijawab

- Bisakah Avalonia bitmap dibuat di luar UI thread, atau harus di `Dispatcher.UIThread`?
  (Jawab dengan test/dokumen, bukan opini — cek `Avalonia.Media.Imaging.Bitmap` thread affinity.)

## Hasil yang dikirim kembali

Elapsed ms + jumlah item + status hidrasi + status overlay + jawaban bitmap thread.
