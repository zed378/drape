# P0-03 Runbook — jalankan di VM Windows

## Persiapan

1. Ambil checkpoint VM sebelum mulai.
2. Catat: Windows build (`winver`), DPI, jumlah monitor + DPI tiap monitor.
3. Build di mesin dev: `dotnet build spikes/p0-03-desktop-host -c Release`.
4. Salin folder `spikes/p0-03-desktop-host/bin/Release/net10.0-windows10.0.19041.0/` ke VM.

## Jalankan

Di VM, untuk tiap strategi (`A`, `B`, `C`):

```
p0-03-desktop-host.exe C
```

Jendela uji berwarna biru transparan akan muncul. Untuk tiap strategi, isi tabel di bawah:

| Kriteria (ADR-007) | A | B | C |
|---|---|---|---|
| 1. Di atas wallpaper, di bawah app windows | | | |
| 2. Tetap terlihat setelah Win+D dan peek | | | |
| 3. Tidak mencuri fokus; tidak ada di taskbar/Alt+Tab | | | |
| 4. Pulih setelah Explorer restart (manual, di VM) | | | |
| 5. Transparan/blur benar pada 100/150/200% DPI | | | |
| 6. Idle CPU/GPU wajar | | | |

## Cara uji tiap kriteria

- **Win+D**: tekan Win+D dua kali; jendela harus tetap terlihat.
- **Peek**: arahkan ke pojok "Show desktop"; jendela harus tetap terlihat.
- **Fokus**: saat jendela muncul, fokus harus tetap di app sebelumnya. Alt+Tab tidak boleh menampilkan "P0-03 spike".
- **Explorer restart**: buka Task Manager → restart Windows Explorer (jangan kill via script). Tunggu desktop kembali, cek jendela masih di layer yang benar.
- **DPI**: ubah skala display ke 150% dan 200%, logout/login, cek transparansi dan posisi.
- **Monitor**: jika VM mendukung 2 monitor beda DPI, pindahkan jendela dan cek.

## Setelah selesai

Kirim kembali: tabel terisi + `winver` + DPI + jumlah monitor. Tulis ke record P0-03.
