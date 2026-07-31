<div align="center">

# can-viewer

![can-viewer Logo](assets/svg/logo-256x256.svg)

**Open-source cross-platform CAN/LIN bus data analysis tool**

[![Build](https://img.shields.io/github/actions/workflow/status/cantool/can-viewer/build.yml?branch=main&style=flat-square)](https://github.com/cantool/can-viewer/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE.txt)
[![Rust](https://img.shields.io/badge/rust-stable-orange?style=flat-square)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20|%20macOS%20|%20Linux-lightgrey?style=flat-square)](https://github.com/cantool/can-viewer/releases)

Fully open source — your ⭐ keeps development going!

[English](README.md) | [中文文档](README_zh.md)

</div>

---

## Overview

`can-viewer` is a high-performance automotive bus analysis tool built in Rust. It parses Vector BLF log files, decodes signals against DBC / LDF databases, and renders interactive waveform plots in a [GPUI](https://gpui.rs/)-accelerated desktop UI. Built for engineers who need to inspect large recordings without waiting on slow Java-based tools.

A typical session: drop a `.blf` recording onto the window, activate a DBC database for the matching channel, browse the decoded log, then plot selected signals to scrub through time. Multiple recordings can be loaded side-by-side on a single merged timeline.

---

## Features

**Performance**
- 🚀 **Rust native** — zero-copy parsing, fast startup
- ⚡ **Parallel BLF loading** — zlib decompress runs across CPU cores via rayon; a 9 MB / 860 K-object file loads in ~280 ms warm
- 🖥️ **GPU-accelerated UI** — built on GPUI, smooth rendering even with hundreds of thousands of rows

**Bus & databases**
- 🔌 **Multi-bus support** — CAN / CAN FD / LIN / FlexRay / Ethernet
- 📚 **Multi-version signal libraries** — keep multiple DBC / LDF versions per channel; one-click activation, auto-load on startup
- 🎯 **Signal sets** — save named channel+message+signal combinations on a library, then batch-apply them to the plot sidebar for fast replay across recordings

**Log viewing**
- 📊 **Interactive waveform plot** — zoom, pan, hover to inspect, live plot on selection, no-data placeholders for signals absent from the log
- 🔍 **ID / channel filters** — narrow the log view by message ID or bus channel
- ⚠️ **Parse-error surfacing** — failed parses show ⚠️ in the status bar; the Loaded Files popover lists per-file errors with `❌`

**Multi-file & UX**
- 📁 **Merged timeline** — load multiple BLF files in parallel; messages sorted by absolute timestamp (`measurement_start_time + object_time_stamp`), so recordings from different sessions line up
- 🖱️ **Drag-and-drop loading** — drop `.blf` files or a folder onto the window; macOS Dock-icon drop also supported; large files (> 1 GB) trigger a confirmation prompt
- 🔗 **LAN sharing** — share signal libraries over HTTP within your local network, one-click import on the receiving side
- 🌐 **Cross-platform** — Windows, macOS, Linux

---

## Screenshots

| Log Viewer | Signal Plot |
|:---:|:---:|
| <img src="assets/blf_logs_screenshot.png" width="400" alt="BLF Logs Viewer" /> | <img src="assets/plot_screen.png" width="400" alt="Signal Plotter" /> |

| Signal Library Management |
|:---:|
| <img src="assets/dbc_library.png" width="600" alt="Signal Library" /> |

---

## Quick Start

**Requirements:** Rust stable (1.85+ for edition 2024), Git

```bash
# Linux additional dependencies
sudo apt-get install libxkbcommon-dev libx11-dev libegl1-mesa-dev

# Build and run
git clone https://github.com/cantool/can-viewer.git
cd can-viewer
cargo run --release --bin viewer
```

Pre-built binaries: [Releases](https://github.com/cantool/can-viewer/releases) (Windows / macOS / Linux)

### Try it with demo data

A small demo database and matching BLF are included in the repo root:

- `demo.dbc` — 6 messages (EngineStatus, VehicleSpeed, ControlCommand, TirePressure, BrakeStatus) with mixed signal types
- `demo.blf` — 2 minutes / 6 000 CAN frames across 120 zlib-compressed LogContainers; IDs match `demo.dbc`

Drop `demo.blf` onto the window, activate `demo.dbc` on channel 1, then switch to Signal Plot to see decoded signals.

---

## Usage

1. **Open log** — File → Open BLF… (single, replaces) or Open Multiple BLF… (append). Drag-and-drop also works — dropping always clears the current session and reloads. On macOS, Dock-icon drop is supported too.
2. **Manage signal libraries** — Library tab → add DBC / LDF files → activate a version for the channel.
3. **Signal sets** — on a library, save the current channel+message+signal selection as a named set. Open the dropdown from the plot sidebar header to apply (batch-select), rename, or delete.
4. **Browse log** — Log tab shows decoded signal values per frame.
5. **Plot waveforms** — Signal Plot tab → expand a channel/message → select signals. Supports zoom, hover, absolute time.
6. **Manage loaded files** — when ≥ 2 files are loaded (or any file has errors), click **📂 N files** in the bottom-left status bar to open the Loaded Files popover. Remove individual files with ✕, or **Remove All**.

Drop semantics: only `.blf` is accepted via drag (use File menu for `.bin`); folders are expanded one level deep; > 1 GB total triggers a confirmation; mid-load drops cancel the in-flight load and queue the new one.

Configuration is auto-saved to `multi_channel_config.json` next to the binary.

📖 **Full user guide:** [docs/USAGE.md](docs/USAGE.md) — window layout, log/plot views, library management, signal sets, LAN sharing, keyboard shortcuts, troubleshooting.

---

## Roadmap

### Done

- ✅ BLF parsing core (CAN / CAN FD / LIN / FlexRay / Ethernet)
- ✅ DBC / LDF database parsing
- ✅ GPUI desktop UI (GPU-accelerated, cross-platform)
- ✅ Message filtering (by ID and channel) and signal decoding
- ✅ Signal waveform plot — zoom, hover, absolute time, live plot on selection, no-data placeholders
- ✅ Multi-version signal library management (create / activate / share via LAN, one-click import)
- ✅ Signal sets — named channel+message+signal collections, batch-apply to plot
- ✅ Multi-file loading with merged timeline (Open BLF... replaces, Open Multiple BLF... appends)
- ✅ Drag-and-drop BLF loading (in-window + macOS Dock-icon), folder expansion, large-file guard
- ✅ Status bar multi-file segment with ⚠️ parse-error indicator and Loaded Files popover
- ✅ Parallel BLF loading (rayon parallel zlib decompress + index-sort merge, ~3× faster on multi-core CPUs)

### Planned

- ⬜ Live streaming mode (socketcan / virtual bus)
- ⬜ Export to CSV / JSON
- ⬜ Diagnostic rule DSL (custom warning / error conditions)
- ⬜ DBC generator from observed traffic
- ⬜ Replay / re-injection (send back to bus)

---

## Development

```bash
cargo test --workspace   # Run tests
cargo fmt --all          # Format
cargo clippy --workspace # Lint
```

For cross-platform builds and packaging scripts, see [`scripts/`](scripts/) and the [build workflow](.github/workflows/build.yml).

---

## Contributing

Fork and submit a Pull Request. Please ensure your code passes `cargo fmt` and `cargo clippy` before pushing.

---

## License

[MIT License](LICENSE.txt) © 2026 can-viewer

---

<div align="center">

**Built with ❤️ in Rust**

[Issues](https://github.com/cantool/can-viewer/issues) · [Discussions](https://github.com/cantool/can-viewer/discussions) · admin@ucan.me

</div>
