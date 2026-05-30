# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build --release
```

The output `.so` library is at `target/release/libwaybar_niri_workspaces.so`.

## Architecture

This is a Waybar CFFI module that displays Niri window manager workspaces and windows. The module is compiled as a dynamic library (`cdylib`) loaded by Waybar.

### Module Structure

**Entry Point (`lib.rs`)**
- Implements the `waybar_cffi::Module` trait
- `WindowButtonsModule::init()` creates the GTK UI hierarchy and spawns the event loop
- `ModuleInstance` manages the UI state with `BTreeMap<u64, WindowButton>` and `BTreeMap<u64, WorkspaceBox>`
- Processes `WorkspaceSnapshot` events from the compositor thread, incrementally updating the UI

**Compositor Communication (`compositor.rs`)**
- `CompositorClient`: sends actions (focus, move, close) to Niri via IPC socket
- Event streams (`start_window_stream`, `start_workspace_stream`): spawn threads that listen to Niri events and send snapshots via async channels
- `WindowTracker`: state machine that tracks windows and workspaces. State transitions:
  - `WindowsOnly` → `Ready` when workspaces event arrives
  - `WorkspacesOnly` → `Ready` when windows event arrives
  - `Ready`: both windows and workspaces are known, generates `WorkspaceSnapshot`
- Reconnects with exponential backoff on IPC errors (max 30s)

**Shared State (`global.rs`)**
- `SharedState` wraps an `Arc<StateInner>` containing settings, icon resolver, and compositor client
- `create_event_stream()` bridges Niri IPC threads (sync) to GTK main loop (async) using `async_channel`

**UI Components (`widget.rs`)**
- `WindowButton`: GTK button with app icon, connects click to `compositor.focus_window()`
- Icons resolved via `IconResolver` or fallback to `IconTheme` default (`application-x-executable`)
- Icons re-rendered on size allocation changes for HiDPI scaling

**Configuration (`settings.rs`)**
- Deserializes from Waybar's JSON config
- `ignore_rules`: filter windows by `app_id`, `title` (exact/contains/regex), or `workspace`

**Icon Resolution (`icons.rs`)**
- `IconResolver` with mutex-protected cache
- Searches `.desktop` files in XDG data directories (with/without `.desktop` suffix, KDE prefixes)
- Falls back to GTK icon theme lookup

### Data Flow

1. `lib.rs::init()` creates `SharedState` and spawns `ModuleInstance::run_event_loop()`
2. `global.rs::create_event_stream()` spawns two compositor threads (windows + workspaces)
3. Compositor threads connect to Niri IPC, receive events, update `WindowTracker`, send `WorkspaceSnapshot` via channel
4. UI receives snapshot, diffs against previous state, adds/removes/reorders GTK widgets

### Event Loop Concurrency

- Niri IPC runs in dedicated threads (blocking socket I/O)
- IPC threads send to intermediate channels (`glib_win_tx`, `glib_ws_tx`)
- GLib main loop bridges intermediate channels to UI event stream (`EventMessage`)
- UI updates happen in the GLib main context