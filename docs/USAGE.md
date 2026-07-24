# CANVIEW User Guide

A practical manual for using CANVIEW — the BLF log viewer, signal plotter, and DBC/LDF library manager.

> **Other languages:** [中文使用文档](USAGE_zh.md)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Window Layout](#window-layout)
3. [Loading BLF Files](#loading-blf-files)
4. [Log View](#log-view)
5. [Plot View](#plot-view)
6. [Library Management](#library-management)
7. [Server Sharing & Import](#server-sharing--import)
8. [Configuration View](#configuration-view)
9. [Status Bar Reference](#status-bar-reference)
10. [Keyboard Shortcuts](#keyboard-shortcuts)
11. [Persisted State](#persisted-state)
12. [Troubleshooting](#troubleshooting)

---

## Quick Start

1. **Launch** the app.
2. Click **File → Open BLF…** in the top bar and pick a `.blf` (or `.bin`) file.
   - Files larger than 1 GB trigger a confirmation dialog.
3. The default **Log** view shows decoded messages. Switch to **Plot** in the bottom-left of the status bar to plot signals.
4. On first plot, if no DBC/LDF library is active, the **Select signal library** overlay appears — pick a library version to decode signals.

For a fresh install with no libraries configured yet, go to **Library** (top bar) → **+ New Library**, add a version, then add channels pointing to your `.dbc` (CAN) or `.ldf` (LIN) files.

---

## Window Layout

```
┌────────────────────────────────────────────────────────────┐
│  [macOS traffic lights]   File  Library                     │ ← Top bar (36px)
├────────────────────────────────────────────────────────────┤
│                                                            │
│                                                            │
│                  Content area                              │ ← Log / Plot / Library / Config
│                                                            │
│                                                            │
├────────────────────────────────────────────────────────────┤
│ Log Plot │ 📂 file │ msgs │ DBC │ LDF │  Server │ 📚 lib │ log mode │ ← Status bar (24px)
└────────────────────────────────────────────────────────────┘
```

- **Top bar** — File menu, Library button, window controls (Win/Linux).
- **Content area** — switches between Log / Plot / Library / Config.
- **Status bar** — file info, message/DBC/LDF counts, server status, active library, view name, and the Log ⇄ Plot toggle.

---

## Loading BLF Files

Open the **File** dropdown in the top bar. Two options:

### Open BLF…

Replaces the currently loaded file with a single new one. Filtered to `.blf` / `.bin`. Files > 1 GB trigger a confirmation prompt.

### Open Multiple BLF…

Appends multiple files to the current session. Each file is loaded in parallel, then merged on a shared timeline. The total size limit (> 1 GB combined) triggers a confirmation prompt.

### During loading

- The status bar shows progress: `⏳ Loading X/Y files (N failed so far) — M messages`.
- An **❌ Cancel** button appears on the right of the status bar — clicking it cancels pending files (already-loaded ones stay).
- Per-file parse errors don't abort the whole batch — failed files appear in the **Loaded Files** popover with `❌` and an error list.

### Managing loaded files

Click the **file segment** in the status bar (left side) when ≥ 2 files are loaded, or when any file has parse errors, to open the **Loaded Files** popover:

- Each row shows: status icon (`✅` clean / `❌` errors), file name, message count, byte size, deduplicated error list.
- Click **✕** on a row to remove that file.
- **Remove All** (bottom-left) clears everything.
- **Done** (bottom-right) closes the popover.
- The list scrolls when more than 8 files are loaded; the popover shrinks to content size when ≤ 8 files.

---

## Log View

Default view after loading a BLF. Shows one row per CAN/LIN message.

### Columns

| Column   | Behavior |
|----------|----------|
| **#**    | Row index. |
| **TIME** | Absolute (`YYYY-MM-DD HH:MM:SS.ffffff`) when the BLF has a start time; relative seconds otherwise. |
| **CH**   | Channel number. Click ⚙ next to the header to open the channel filter dropdown; click ✓ to clear the filter. |
| **TYPE** | Color-coded: CAN/CAN2 green, CAN_ERR red, CAN_FD/CAN_FD64 purple, CAN_OV amber, LIN/LIN2 blue. |
| **ID**   | Message ID. Click the label to toggle decimal ⇄ hex (small `10` / `16` suffix indicates the active base). ⚙ / ✓ toggles the ID filter dropdown. |
| **DLC**  | Data length code. |
| **DATA** | Raw payload bytes. |

### Filtering

Two filters — **ID filter** and **Channel filter** — work the same way:

1. Click ⚙ next to the column header to open a dropdown listing every unique value.
2. Click a value to set the filter, or type digits (ID only) to refine.
3. ✓ clears the filter. **Escape** closes the dropdown (and clears the filter if pressed again).

Both filters combine with AND when both are set.

### Empty state

Shows *"No messages loaded. Click '📂 Open BLF' to load a file."*

---

## Plot View

Switch via the **Plot** button (bottom-left of the status bar). Renders one chart per selected signal.

### Signal sidebar (left, 320px)

- Header shows signal count.
- **Search box** filters by signal name, message name, or ID.
- Items are grouped by channel (CAN blue / LIN yellow), then message, then signal.
- Channels that have a configured library mapping but aren't loaded show a **Load** button — click to load that library version.
- Each signal has a 12×12 px checkbox; click to add/remove from the selection.
- The **Plot N signals (Plot)** button at the bottom of the sidebar runs the data extraction.

### Toolbar

- **Reset Zoom** — appears only when zoomed; restores the full time range.
- **Points: ON/OFF** — toggles whether individual data points are rendered as dots.

### Zoom

- **Mouse wheel** — zoom in/out centered on the cursor. Min range is bounded by the smallest gap between adjacent points across all series.
- **Drag to zoom** — click-drag horizontally on a chart to select a region. Selections > 10 px commit the zoom; smaller selections are ignored.
- **Double-click** — resets zoom.

### Hover tooltip

Moving the mouse over a chart shows:

- A vertical hover line at the cursor X.
- A tooltip with the time (absolute if `start_time` is set, otherwise relative seconds) and one row per series with its colored dot, value, and unit.
- The tooltip flips to the left of the cursor when there's not enough space on the right.

### Time display

X-axis labels use absolute `YYYY-MM-DD HH:MM:SS.mmm` when the BLF has a start time; otherwise relative seconds from the first message.

### Per-chart header

`{signal name} [{unit}] | {N} pts | {min}s-{max}s (span: {span}s)`

### Empty state

Shows *"尚无数据显示"* / *"请在信号选择(Signals)中选择信号,点击Plot按钮加载数据"* plus a debug line with message and selection counts.

---

## Library Management

Switch via the **Library** button in the top bar. Three-column layout: libraries | versions | channels.

### Create a library

1. In the left column, click **+ New Library**.
2. Type a name and press **Enter** (or click ✓). **Escape** cancels.

### Add a version

1. Select a library on the left.
2. Click **+ Add Version** in the middle column.
3. Type a version name (e.g., `v1.0`) and press **Enter**.

### Add a channel (DBC/LDF file)

1. Select a library and a version.
2. Click **+ Add Channel** in the right column.
3. Toggle the type (CAN ⇄ LIN) if needed.
4. Enter channel ID and channel name (the name auto-fills from the file name stem if left empty).
5. Click **Select File…** and pick a `.dbc` (CAN) or `.ldf` (LIN) file.
6. Click ✓ to save or ✕ to cancel. **Enter** saves, **Escape** cancels.

The database file is copied into `<config_dir>/libraries/<lib>/<version>/<channel>/` so the original can be moved or deleted.

### Activate a version

Click the ▷ button next to a version (becomes ▶ when active). Activating:

- Loads the DBC/LDF databases for all configured channels into memory.
- Updates the active library badge in the status bar (`📚 lib_name / version`).
- Persists the active state to config so it survives restarts.

Click ▶ to deactivate. The library picker overlay reappears in Plot view when no version is active.

### Rename / Delete

- **✎** — rename a library or version. Inline input with ✓ / ✕ confirmation.
- **🗑** — delete a library, version, or channel. No undo.

### Apply to Plot

Click **Apply to Plot** at the top of the right column to write the current version's channel mappings into the app config (without activating). This updates `app_config.mappings` and saves to disk.

---

## Server Sharing & Import

Share your library database files with other machines on the same LAN, or import a shared library.

### Start sharing

1. In the **Library** view, click **📡 Share** (bottom-left).
2. The app binds `0.0.0.0:0` (OS-assigned port) and exposes:
   - `GET /api/health` — unauthenticated health check.
   - `GET /api/libraries?token=...` — library list as JSON, with `database_path` rewritten to per-channel download URLs.
   - `GET /api/libraries/{lib_id}/versions/{ver}/files/{channel_id}?token=...` — streams the DBC/LDF file.
3. A dialog appears with the **LAN URL** (preferred `192.168.x.x` / `10.x.x.x` / `172.16-31.x.x` address) and a local `127.0.0.1` fallback.
4. **📋 Copy** writes the URL to the clipboard. **🌐 Open** opens it in the system browser.
5. The server stops when you click **📡 Stop Share**, close the app, or the `ServerHandle` is dropped.

The token is a random opaque string validated on every request — keep it in the URL when sharing.

### Import on another machine

1. In the **Library** view, click **📥 Import**.
2. Paste the shared URL (`http://...?token=...`) into the input.
3. Click **Import**. The app fetches the library list and downloads each channel database to a local `libraries/` subdirectory.
4. On completion, the imported libraries appear in your library manager and the dialog closes. Duplicate names get `_1`, `_2` suffixes to avoid clobbering existing entries.

---

## Configuration View

Switch via the status bar (view name shows `config mode`). Mostly a read-only summary.

- **Import Database** — currently a stub; use the Library view instead.
- **Save Config** — writes `multi_channel_config.json` next to the executable.
- **Channel Mappings** — display-only list of current mappings (channel ID, type, file path).
- **System Status** — message count, DBC count, LIN count.

Editing happens in the Library view; this view just lets you trigger a manual save and inspect state.

---

## Status Bar Reference

The 24-px bar at the bottom of the window.

### Left side

| Segment | Description |
|---|---|
| **Log / Plot toggle** | Switches between Log and Plot views. Active button gets the primary-color background. |
| **File** | `📂 No file loaded — File > Open BLF...` (empty), `📂 <name>` (1 file, no errors), `📂 <name> ⚠️` (1 file with errors), `📂 N files` (≥2 files, no errors), or `📂 N files ⚠️` (≥2 files with errors). Click the segment to open the Loaded Files popover when clickable. ⚠️ is yellow. |
| **BLF progress** | `consumed / total (pct)` while parsing. Yellow below 100%, secondary text at 100%. |
| **msgs** | Total loaded message count. |
| **DBC** | Number of loaded DBC databases. |
| **LDF** | Number of loaded LDF databases. |

### Right side

| Segment | Description |
|---|---|
| **Server** | Green dot + `Server ON <url>` when running, gray dot + `Share disabled` when stopped. Click to copy the URL to the clipboard. |
| **status_msg** | Transient status text (Loading…, Ready, errors). Truncated if long. |
| **Library badge** | `📚 <lib_name> / <version>` when a library version is active. |
| **Cancel** | `❌ Cancel` shown only during loading. Click to abort pending loads. |
| **view name** | `log mode` / `plot mode` / `library mode` / `config mode`. |

### Loaded Files popover

Triggered by clicking the file segment when clickable. See [Loading BLF Files](#loading-blf-files) for details.

---

## Keyboard Shortcuts

CANVIEW has no global hotkey registry. Shortcuts are per-view.

### Log view

| Key | Action |
|---|---|
| `0`–`9` | Append digit to the ID filter text (works whether or not the dropdown is open). |
| `Backspace` | Delete last digit of the ID filter text; clears the ID filter if the text is empty. |
| `Enter` | Apply the ID filter (parses the current text as a decimal u32). |
| `Escape` | Close the ID filter dropdown if open; otherwise clear the ID filter. |

### Library view

| Key | Context | Action |
|---|---|---|
| `Enter` | Add Library input | Create the library and close the input. |
| `Enter` | Add Version input | Add the version and close the input. |
| `Enter` | Add Channel input | Save the channel config. |
| `Escape` | Any add/rename input | Cancel without saving. |

### Plot view

- **Double-click** on a chart — reset zoom.
- **Mouse wheel** — zoom in/out.
- No keyboard shortcuts.

### Window controls (Win/Linux)

The minimize / maximize / close buttons in the top-right are mouse-only.

---

## Persisted State

### Saved to disk (`multi_channel_config.json`)

Located next to the executable (or the current working directory as a fallback). Pretty-printed JSON. Saved on:

- Create / delete library
- Add / delete version
- Add / delete channel
- Activate / deactivate version
- Apply to Plot
- Manual **Save Config** in the Configuration view
- After a successful LAN import

Contents:

- `libraries` — full library / version / channel hierarchy.
- `mappings` — channel-to-database mappings (type, ID, file path, library ID, version name).
- `active_library_id`, `active_version_name` — the currently active version.

### Library files on disk

DBC/LDF files are copied into `<config_dir>/libraries/<lib>/<version>/<channel>/` when you add a channel. On macOS the config dir is `~/Library/Application Support/canview`; on Linux it's `~/.config/canview`; on Windows it's `%APPDATA%/canview`.

### In-memory only (not persisted)

Loaded BLF messages, decoded DBC/LDF databases, plot zoom state, and selected signals survive a window maximize/restore (saved in `RuntimeState`), but are **not** written to disk — they're lost when the app quits.

---

## Troubleshooting

### No messages after loading

- Check the file size — files > 1 GB may need confirmation.
- Open the Loaded Files popover (click the file segment) and look for `❌` icons — the file may have parse errors.

### ⚠️ appears next to the file name

The file loaded but had parse errors. Click the segment to open the Loaded Files popover and read the error list under the offending file.

### Signals don't decode in Plot view

- A library version must be **activated**, not just configured. Check the status bar's library badge.
- If the **Select signal library** overlay appears, pick a version or click ✕ to dismiss.
- Click **Load** next to unloaded channels in the signal sidebar to load their library version.

### Server URL shows `127.0.0.1` instead of a LAN IP

The app couldn't detect a preferred LAN IP (192.168/10./172.16-31). Check that you're on a network with a routable LAN address. The local URL still works on the same machine.

### Import fails

- Make sure the URL includes the `?token=...` parameter.
- The sharing machine must be reachable on the same LAN and the server must be running.
- Check the status message at the bottom for the import result.

### Changes don't persist across restarts

Library / version / channel changes and active state are saved automatically. Loaded BLF files and plot state are not — reload the file after restarting.
