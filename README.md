<div align="center">

# can-viewer

![can-viewer Logo](assets/svg/logo-256x256.svg)

**Open-source cross-platform CAN/LIN bus data analysis tool**

[![Build](https://img.shields.io/github/actions/workflow/status/cantool/can-viewer/build.yml?branch=main&style=flat-square)](https://github.com/cantool/can-viewer/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE.txt)
[![Rust](https://img.shields.io/badge/rust-nightly-orange?style=flat-square)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey?style=flat-square)](https://github.com/cantool/can-viewer/releases)

Fully open source — your ⭐ keeps development going!

[English](README.md) | [中文文档](README_zh.md)

</div>

---

## Overview

can-viewer is a high-performance automotive bus analysis tool built in Rust. It integrates BLF log parsing, DBC/LDF signal library management, and a [GPUI](https://gpui.rs/)-powered GPU-accelerated desktop UI.

- 🚀 **Rust native** — zero-copy parsing, fast startup
- ⚡ **Parallel BLF loading** — zlib decompress runs across CPU cores via rayon; 9 MB / 860 K-object files load in ~280 ms
- 🖥️ **GPU-accelerated UI** — built on GPUI, smooth rendering
- 📊 **Interactive waveform plot** — zoom, pan, hover to inspect
- 📚 **Signal library management** — multi-version DBC/LDF libraries, one-click activation, auto-load
- 🎯 **Signal sets** — save named collections of channel+message+signal combinations on a library, then batch-apply them to the plot sidebar for quick replay
- 🔗 **LAN sharing** — share signal libraries within your local network via HTTP
- 🔌 **Multi-bus support** — CAN / CAN FD / LIN / FlexRay / Ethernet
- 🖱️ **Drag-and-drop loading** — drop `.blf` files (or a folder) onto the window; macOS Dock-icon drop also supported
- 📁 **Multi-file merged timeline** — load multiple BLF files in parallel; messages are sorted by absolute timestamp (`measurement_start_time + object_time_stamp`), so files recorded at different times line up correctly
- ⚠️ **Parse-error surfacing** — failed parses show a warning in the status bar; the Loaded Files popover lists per-file errors with `❌`
- 🌐 **Cross-platform** — Windows, macOS, Linux

---

## Screenshots

| Log Viewer | Signal Plot |
|:---:|:---:|
| ![BLF Logs Viewer](assets/blf_logs_screenshot.png) | ![Signal Plotter](assets/plot_screen.png) |

| Signal Library Management |
|:---:|
| ![Signal Library](assets/dbc_library.png) |

---

## Quick Start

**Requirements:** Rust nightly, Git

```bash
# Linux additional dependencies
sudo apt-get install libxkbcommon-dev libx11-dev libegl1-mesa-dev

# Build and run
git clone https://github.com/cantool/can-viewer.git
cd can-viewer
cargo run --release --bin viewer
```

Pre-built binaries: [Releases](https://github.com/cantool/can-viewer/releases) (Windows / macOS / Linux)

---

## Usage

1. **Open log** — click File → Open BLF… (single file), or Open Multiple BLF… (append). You can also **drag-and-drop** `.blf` files onto the window — dropping always clears the current session and reloads. On macOS, dropping on the Dock icon also works.
2. **Manage signal libraries** — switch to the Library tab, add DBC/LDF files, activate a version
3. **Signal sets** — on a library, save named collections of channel+message+signal combinations. Open the set dropdown from the plot sidebar header to apply (batch-select), rename, or delete a set — useful for replaying the same group of signals across recordings.
4. **Browse log** — switch to the Log tab to view decoded signal values
5. **Plot waveforms** — switch to Signal Plot, select signals; supports zoom and hover
6. **Manage loaded files** — when ≥ 2 files are loaded (or any file has errors), click the **📂 N files** segment in the bottom-left status bar to open the Loaded Files popover. Remove individual files with ✕, or **Remove All** to clear.

Drop semantics: only `.blf` is accepted (`.bin` is ignored via drag — use File menu for `.bin`); folders are expanded one level deep; total > 1 GB triggers a confirmation prompt; mid-load drops cancel the in-flight load and queue the new one.

Configuration is auto-saved to `multi_channel_config.json`.

📖 **Full user guide:** [docs/USAGE.md](docs/USAGE.md) — window layout, log/plot views, library management, signal sets, LAN sharing, keyboard shortcuts, troubleshooting.

---

## Roadmap

### Done

- ✓ BLF parsing core (CAN / CAN FD / LIN / FlexRay / Ethernet)
- ✓ DBC / LDF database parsing
- ✓ GPUI desktop UI (GPU-accelerated, cross-platform)
- ✓ Message filtering (by ID and channel) and signal decoding
- ✓ Signal waveform plot — zoom, hover, absolute time, live plot on selection, no-data placeholders
- ✓ Multi-version signal library management (create / activate / share via LAN, one-click import)
- ✓ Signal sets — save named channel+message+signal combinations on a library, batch-apply to plot
- ✓ Multi-file loading with merged timeline (Open BLF... replaces, Open Multiple BLF... appends)
- ✓ Drag-and-drop BLF loading (in-window + macOS Dock-icon), folder expansion, large-file guard
- ✓ Status bar multi-file segment with ⚠️ parse-error indicator and Loaded Files popover
- ✓ Parallel BLF loading (rayon parallel zlib decompress + index-sort merge, ~3× faster on multi-core CPUs)

### Planned

- ○ Live streaming mode (socketcan / virtual bus)
- ○ Export to CSV / JSON
- ○ Diagnostic rule DSL (custom warning/error conditions)
- ○ DBC generator from observed traffic
- ○ Replay / re-injection (send back to bus)

---

## Development

```bash
cargo test --workspace   # Run tests
cargo fmt --all          # Format
cargo clippy --workspace # Lint
```

For cross-platform builds, see [BUILD.md](BUILD.md).

---

## Contributing

Fork and submit a Pull Request. Please ensure your code passes `cargo fmt` and `cargo clippy`.

---

## License

[MIT License](LICENSE.txt) © 2026 can-viewer

---

<div align="center">

**Built with ❤️ in Rust**

[Issues](https://github.com/cantool/can-viewer/issues) · [Discussions](https://github.com/cantool/can-viewer/discussions) · admin@ucan.me

</div>
