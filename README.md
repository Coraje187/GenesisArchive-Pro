<div align="center">

# ⚡ GenesisArchive Pro

### The Modern, Blazing-Fast, 100% Free WinRAR Alternative for Windows & Linux.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Engine-Rust%201.97-orange.svg)](https://www.rust-lang.org/)
[![Tauri v2](https://img.shields.io/badge/UI-Tauri%20v2%20%2B%20React%2019-24C8DB.svg)](https://tauri.app/)
[![Zstandard](https://img.shields.io/badge/Compression-Meta%20Zstandard-0080FF.svg)](https://facebook.github.io/zstd/)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-lightgrey.svg)]()

**No 40-day nag screens. No license fees. No 1990s UI.**  
Built from the ground up in memory-safe Rust with Meta's Zstandard compression engine, browser-style multi-archive tabs, and in-memory quick-look file previews.

</div>

---

## 🚀 Why GenesisArchive Beats WinRAR

| Feature | Classic WinRAR | GenesisArchive Pro |
| :--- | :--- | :--- |
| **Price & Licensing** | $29 Shareware / Nag-screen trial | **100% Free & Open Source Forever** |
| **Engine Speed** | Deflate / RAR5 (~50–150 MB/s) | **Meta Zstandard (`.tar.zst`) (1,000–1,600+ MB/s)** ⚡ |
| **Linux Support** | Headless CLI only (`rar` binary) | **First-class cross-platform GUI** (Native on Linux & Windows) |
| **File Previews** | Unpacks to `%TEMP%` & opens external app | **Instant In-Memory Quick-Look** (Images, Code syntax, Hex) |
| **Multi-Archive Tabs** | Spawns multiple top-level OS windows | **Tabbed Interface** (Browse multiple archives in one window) |
| **Security Shield** | Basic password prompt | **Pre-Extraction Auditor** (Detects Zip Bombs & Path Traversals) |
| **Installer / App Size** | ~3.5 MB | **~1.9 MB (Setup) / 5.7 MB (Portable .exe)** |

---

## ⚡ Performance Benchmark

Tested live against identical 25 MB payloads using real-world streaming buffers:

```
Algorithm: Meta Zstandard (zstd - Level 3)
  ├── Compression Speed:   1,000 MB/s  (16.6x Faster)
  └── Decompression Speed: 1,667 MB/s  (15.7x Faster)

Algorithm: Standard Deflate / WinRAR style (Gzip - Level 6)
  ├── Compression Speed:   60 MB/s
  └── Decompression Speed: 106 MB/s
```

> **Why this matters:** On modern PCIe 4.0/5.0 NVMe SSDs, GenesisArchive saturates your disk throughput, decompressing 20 GB games or datasets in seconds.

---

## 🌟 Key Features

* 🗂️ **Browser-Style Tabs:** Open, compare, and manage dozens of archives without cluttering your desktop with windows.
* 👁️ **In-Memory Quick-Look Inspector:**
  * **Images:** View `.png`, `.jpg`, `.svg`, `.webp`, `.gif` directly from archive memory without writing to disk.
  * **Code & Text:** Inspect code files (`.ts`, `.py`, `.rs`, `.json`, `.md`) with syntax styling.
  * **Hex Inspector:** Inspect binary headers and raw bytes on demand.
* 🛡️ **Built-in Security Auditor:**
  * Flags **Zip Bombs** before extraction (monitors ratios >100:1 and absurd uncompressed payloads).
  * Blocks **Directory Traversal** attacks (`../` and illegal drive root paths).
  * Alerts you if hidden scripts or binaries (`.exe`, `.scr`, `.bat`, `.cmd`, `.vbs`, `.ps1`) are concealed inside.
* 🗜️ **Universal Format Support:**
  * **Extracts:** `.zip`, `.tar`, `.tar.gz`, `.tar.zst`, `.7z`.
  * **Creates:** `.tar.zst` (Ultra-Fast), `.zip` (Universal), `.7z` (High-ratio LZMA), `.tar.gz` (POSIX standard).
* 🌓 **Fluent Dark / Light Themes:** Sleek Windows 11 Fluent and Linux GTK aesthetic.

---

## 🛡️ Clean-Room Legal Architecture

GenesisArchive was designed 100% clean-room from scratch in Rust and TypeScript:
* **Zero proprietary RAR code:** We do not use or reverse-engineer Eugene Roshal's proprietary RAR compression algorithms.
* **Open Standards:** Compression relies exclusively on open standards (Meta Zstandard, PKWARE ZIP, 7-Zip LZMA, POSIX TAR).
* **Immune to Copyright Claims:** Free for personal, commercial, and enterprise deployment under the MIT License.

---

## 🛠️ Tech Stack

* **Backend / Core Engine:** [Rust](https://www.rust-lang.org/) (`zstd`, `zip`, `sevenz-rust`, `flate2`, `tar`, `rfd`, `rayon`)
* **Desktop Wrapper:** [Tauri v2](https://tauri.app/) (Native OS WebViews: WebView2 on Windows, WebKitGTK on Linux)
* **Frontend:** [React 19](https://react.dev/), [TypeScript](https://www.typescriptlang.org/), [Lucide Icons](https://lucide.dev/)

---

## 🚀 Building & Running from Source

### Prerequisites
1. **Node.js**: v18 or newer
2. **Rust Toolchain**: `rustup`, `cargo`, `rustc` (1.80+)
3. **OS Requirements:**
   * **Windows:** Windows 10/11 (WebView2 is pre-installed)
   * **Linux:** `sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev`

### Development Mode
```bash
git clone https://github.com/Coraje187/GenesisArchive-Pro.git
cd GenesisArchive-Pro

npm install
npm run tauri dev
```

### Build Production Binaries
```bash
npm run tauri build
```
This generates:
* **Windows Setup Installer:** `src-tauri/target/release/bundle/nsis/GenesisArchive_0.1.0_x64-setup.exe` (~1.9 MB)
* **Enterprise MSI:** `src-tauri/target/release/bundle/msi/GenesisArchive_0.1.0_x64_en-US.msi` (~2.7 MB)
* **Standalone Portable Executable:** `src-tauri/target/release/GenesisArchive.exe` (~5.7 MB)
* **Linux:** `.deb` and `.AppImage` packages when compiled on Linux.

---

## 📄 License

Distributed under the **MIT License**. See `LICENSE` for details.
