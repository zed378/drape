# P0-03…P0-07 Spike Harness Batch — Siap Uji di VM

**Date:** 2026-10-25
**Task:** P0-03, P0-04, P0-05, P0-06, P0-07 (harness + spec + runbook; hasil VM menunggu owner)
**Author:** code agent
**Spec refs:** [`docs/PLATFORM/00-WINDOWS-INTEGRATION.md`](../../docs/PLATFORM/00-WINDOWS-INTEGRATION.md) §1–2, §3, §4, §7, §9 · ADR-006, ADR-007

---

## Hook

Semua 5 spike kini punya harness yang kompilasi + spec + runbook. Hasil empiris (tabel per strategi ×
build, 6 kriteria ADR-007) menunggu owner menjalankan di VM. P0-09 (finalisasi ADR) terblokir sampai
hasil itu masuk — dan itu benar: ADR yang diterima tanpa data adalah asumsi yang menyamar.

---

## What Changed

- `MEMORY/specs/P0-03-desktop-host.md`, `P0-04-desktop-icons.md`, `P0-07-mouse-hook.md` — spec ditulis
  sebelum kode, sesuai kartu.
- `spikes/p0-03-desktop-host/` — Avalonia window uji + strategi A (SetParent ke host hasil discovery),
  B (HWND_BOTTOM + NOACTIVATE), C (A lalu fallback B). Discovery: Progman → 0x052C → enum WorkerW.
- `spikes/p0-04-desktop-icons/` — `read`/`hide`/`show`/`--restore` via `WM_COMMAND 0x7402` ke
  SHELLDLL_DefView, baca state via `IsWindowVisible(SysListView32)`. Menolak toggle saat state tak
  terbaca atau sudah sesuai.
- `spikes/p0-05-shell-icons/` — enumerasi Desktop + Public Desktop (Known Folder, skip
  `desktop.ini`/hidden/system), `IShellItemImageFactory::GetImage` per item, timing 500 item.
- `spikes/p0-06-blur-bench/` — 20 jendela (5× transparent/blur/acrylic/solid) untuk idle + drag CPU/GPU.
- `spikes/p0-07-mouse-hook/` — `WH_MOUSE_LL` di thread STA, callback minimal (timestamp, klasifikasi
  kasar, enqueue, forward), latensi end-to-end avg/p99/max, klasifikasi desktop-vs-other via
  `WindowFromPoint` + nama kelas.
- Tiap spike punya `RUNBOOK.md` dengan prosedur VM langkah-demi-langkah dan tabel hasil kosong.

---

## Why

Kartu spike menuntut kode di `spikes/` dan hasil tertulis sebagai ADR. Harness adalah separuh pertama;
separuh kedua (hasil VM) adalah kerja owner karena butuh VM Windows + interaksi manual (Win+D, restart
Explorer, DPI, monitor). Menulis harness dulu berarti owner bisa langsung uji tanpa menunggu kode.

---

## What Surprised Me

- CsWin32 tidak memproyeksikan `FOLDERID_*` — gunakan GUID literal KnownFolders.h.
- `SHGetKnownFolderPath` mengembalikan `CoTaskMemFreePWSTRSafeHandle`; `DangerousValue` tidak boleh
  disimpan (PInvoke017) — pakai inline dalam ekspresi.
- `IShellItemImageFactory::GetImage` mengembalikan `DeleteObjectHBITMAPSafeHandle` (bukan HBITMAP
  mentah) — `using` + `IsInvalid`.
- `POINT` tidak digenerate ("use System.Drawing.Point"); `HHOOK` adalah SafeHandle (`IsInvalid`,
  bukan `IsNull`); `LPARAM` ke pointer via `(void*)lParam.Value`.
- `GetClassName` → `GetClassNameW`; `SIIGBF` (bukan `SIIGBF_FLAGS`); `HWND_INSERT_AFTER` tidak ada.
- Avalonia 12: `SystemDecorations` obsolete, `ExtendClientAreaChromeHints` bukan properti Window,
  `StartWithClassicDesktopLifetime` mengembalikan int, `SetupWithoutStarting` mengembalikan builder,
  `Application` tidak punya `Shutdown()`.

---

## Tests

- `dotnet build Drape.slnx -c Release -warnaserror` — 0 warnings.
- `dotnet format Drape.slnx --verify-no-changes` — clean.
- `dotnet test Drape.slnx -c Release` — **98 passed, 0 failed**.
- Tiap spike: `dotnet build spikes/<name> -c Release` — 0 warnings. Perilaku di VM: menunggu owner.

---

## Platform Matrix

| Windows build | DPI scale | Monitors | Result |
|---|---|---|---|
| Dev machine (build only) | — | — | Semua harness kompilasi |
| VM Windows 10/11 | 100/150/200%, mixed | 1–2 | Menunggu owner |

---

## Definition of Done

- [x] Spec ditulis untuk P0-03, P0-04, P0-07 (P0-05/P0-06: Spec required = no)
- [x] Harness kompilasi untuk P0-03…P0-07
- [x] Runbook dengan tabel hasil kosong untuk tiap spike
- [ ] Hasil VM dari owner (tabel terisi + winver + DPI + monitor)
- [ ] P0-09: ADR-006/007 final + update docs (terblokir sampai hasil masuk)

**Waived:** none.

---

## Desktop Safety

- Spike P0-04 menyembunyikan icon — hanya di VM dengan checkpoint, oleh owner yang mengawasi.
- Tidak ada spike yang menyentuh file user, registry persisten, atau explorer.exe host.
- Hook P0-07 tidak menyimpan atau me-log koordinat.

---

## What to Watch

- Hasil VM menentukan ADR-007 (strategi host) dan ADR-006 (teknik icon). Jangan implementasi
  produksi P1-05/P2-03 sebelum ADR final.
- Jika strategi A gagal di semua build, fallback B menjadi default dan degraded mode harus didesain
  ulang sebagai mode utama.

---

## What Was Not Determined

- Tabel strategi × build (P0-03), biaya blur (P0-06), latensi hook (P0-07). Menunggu owner
  menjalankan bagian interaktif.

## Hasil Parsial VM (2026-10-25, Win11 26H2 build 26300, DPI 100)

- **P0-05**: 5 item di Desktop, diulang jadi 500: ok=500 failed=0, **40812 ms (~82 ms/item)**.
  Konsisten dengan host (~108 ms/item). Target 1 detik/500 item MISSED ~40x. Produksi WAJIB
  async + cache + lazy-load (P1-08); angka ini menjadi baseline optimasi.
- **P0-04**: read hidden=False, hide OK, read hidden=True, hide kedua "already hidden, no toggle
  sent" (TIDAK flip — benar), --restore OK, show OK. Teknik `WM_COMMAND 0x7402` + baca
  `IsWindowVisible(SysListView32)` TERBUKTI di 26H2. Kill test belum (bug skrip: proses `read`
  keluar sendiri sebelum di-kill — skrip diperbaiki, kill test jadi manual).
- Skrip berhenti di error Stop-Process (tanpa try/catch) sehingga P0-07/03/06 belum jalan.
  Skrip diperbaiki: tiap bagian try/catch + lanjut, kill test jadi instruksi manual.

## VM Access (2026-10-25)

- VM `10.1.200.32`: ping diblokir firewall, RDP 3389 OPEN, WinRM 5985/5986 + SMB 445 + SSH 22 TIMEOUT.
  Eksekusi remote tidak mungkin; otomatisasi via "agent polling" (`runs/Run-Spikes.ps1`): owner
  jalankan sekali di VM, skrip menguji otomatis (P0-05, P0-04) + memandu interaktif (P0-03/06/07),
  menulis `spike-results.txt`. Sintaks PS 5.1 OK (em-dash harus dihindari — PSParser 5.1 gagal).
- Paket: `runs/drape-spikes-full.zip` (376 MB, semua exe + runbook + agent) + 5 ZIP per spike +
  `runs/spike-agent.zip` (skrip + panduan, 4 KB). ZIP di-ignore git; `CARA-UJI.md` dicommit.
