# QQPlayer — Architecture & Design

## Overview

QQPlayer is a lightweight music player built with Rust and Tauri v2. It uses a multi-window architecture with a main control window, detachable playlist window, and detachable lyrics window.

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────┐
│          Tauri Application (main.rs)        │
│  ┌───────────────────────────────────────┐  │
│  │   Main Window + System Tray           │  │
│  │   - Playback controls                 │  │
│  │   - Progress bar                      │  │
│  │   - Toggle buttons for sub-windows    │  │
│  └─────────────┬─────────────────────────┘  │
│                │ PlayerCommand (mpsc)        │
│                ▼                             │
│  ┌───────────────────────────────────────┐  │
│  │   Player Thread                       │  │
│  │   ┌─────────────────────────────────┐ │  │
│  │   │  AudioPlayer (player.rs)        │ │  │
│  │   │  - Playlist management          │ │  │
│  │   │  - Playback control             │ │  │
│  │   │  - Auto-advance                 │ │  │
│  │   │  - Lyrics synchronization       │ │  │
│  │   │  - Rodio audio backend          │ │  │
│  │   └─────────────────────────────────┘ │  │
│  └───────────────────────────────────────┘  │
│                                             │
│  ┌──────────────┐  ┌───────────────────┐   │
│  │ Playlist Win │  │ Lyrics Window     │   │
│  │ (child)      │  │ (child)           │   │
│  └──────────────┘  └───────────────────┘   │
└─────────────────────────────────────────────┘
```

### Component Breakdown

#### 1. Main Application (`main.rs`)

**Responsibilities:**
- Initialize Tauri application framework
- Create and manage system tray icon
- Build and dynamically update tray menu
- Handle user interaction events
- Route commands to player thread via message passing
- Manage companion windows (playlist, lyrics) as child windows

**Key Design Patterns:**
- **Message Passing:** Uses `mpsc::channel` to communicate between UI thread and player thread
- **Event-Driven:** Menu events and Tauri commands trigger `PlayerCommand` messages
- **Shared State:** `Arc<Mutex<PlayerState>>` provides read access for frontend polling
- **Parent-Child Windows:** Companion windows use `.parent(&main_win)` for native visibility sync

**Command Flow:**
```rust
User Click → Tauri Command → PlayerCommand → Player Thread → State Update → Menu Rebuild
```

#### 2. Audio Player (`player.rs`)

**Responsibilities:**
- Manage audio playback using the `rodio` library
- Maintain multiple playlists of audio files
- Handle play, pause, stop, next, previous, seek operations
- Auto-advance to next track when current finishes
- Support play modes: cycle, repeat one, shuffle
- Parse and synchronize LRC lyrics
- Persist playlist state to disk

**Architecture Details:**
- **Non-Send Design:** `AudioPlayer` is not thread-safe by design, owned entirely by the player thread
- **Sink Pattern:** Uses `rodio::Sink` for audio output control
- **Lazy Playback:** Creates output stream once and reuses it throughout the application lifecycle

**State Machine:**
```
[Empty] ──load──> [Loaded] ──play──> [Playing]
                     │                    │
                     │                    │ pause
                     │                    ▼
                     │                 [Paused]
                     │                    │
                     │<────── stop ───────┘
```

#### 3. Icon Generator (`icon.rs`)

Procedurally generates the application icon at runtime (blue circle with white music note), reducing asset dependencies.

#### 4. Lyrics Parser (`lrc.rs`)

Parses `.lrc` lyric files with timestamp synchronization for real-time lyric display.

### Threading Model

The application uses a **multi-threaded architecture** to prevent UI blocking:

1. **Main Thread:** Runs Tauri event loop, handles tray menu interactions and window management
2. **Player Thread:** Owns the `AudioPlayer`, processes commands at 100ms intervals, updates shared state
3. **Frontend Polling:** Each window polls `cmd_get_state` every 200–250ms for UI updates

**Why Separate Threads?**
- Audio operations can be blocking or long-running
- Keeps UI responsive during file loading and playback operations
- `rodio` types are not `Send`, so they must live on a single thread

### Data Flow

```
┌──────────────┐
│ User Action  │
└──────┬───────┘
       │
       ▼
┌──────────────────┐      ┌──────────────────┐
│ Tauri Command /  │─────>│ File Dialog      │ (if import)
│ Menu Event       │      │ (rfd)            │
└──────┬───────────┘      └─────┬────────────┘
       │                        │
       ▼                        ▼
┌──────────────────────────────────┐
│ PlayerCommand (enum)             │
│ - Play/Pause/Stop/Next/Previous  │
│ - LoadFile/LoadFolder            │
│ - Seek/PlayIndex/SetPlayMode     │
│ - CreatePlaylist/SwitchPlaylist   │
└────────┬─────────────────────────┘
         │ mpsc::send
         ▼
┌──────────────────────────────────┐
│ Player Thread Loop (100ms)       │
│ - cmd_rx.try_recv()              │
│ - Process command                │
│ - check_auto_advance()           │
│ - update_shared_state()          │
│ - rebuild_menu() on changes      │
│ - save_state() on changes        │
└──────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────┐
│ Arc<Mutex<PlayerState>>          │
│ - Track names, durations         │
│ - Playback position              │
│ - Playlist info                  │
│ - Lyrics + current lyric index   │
└──────────────────────────────────┘
         │
         ▼  (polled by frontends)
┌──────────────────────────────────┐
│ Frontend Windows                 │
│ - Main: progress, controls       │
│ - Playlist: track list, tabs     │
│ - Lyrics: synced lyric display   │
└──────────────────────────────────┘
```

## Technical Stack

- **[Tauri](https://tauri.app/)** v2 — Cross-platform desktop application framework
- **[rodio](https://github.com/RustAudio/rodio)** — Pure Rust audio playback library
- **[image](https://github.com/image-rs/image)** — Image processing for icon generation
- **[walkdir](https://github.com/BurntSushi/walkdir)** — Recursive directory traversal
- **[rfd](https://github.com/PolyMeilex/rfd)** — Native file dialog support
- **[serde](https://serde.rs/)** — Serialization framework
- **[mp3-duration](https://crates.io/crates/mp3-duration)** — MP3 duration reading

## Key Design Decisions

### 1. Multi-Window with Parent-Child Relationship
**Decision:** Playlist and lyrics are separate child windows of the main window.

**Rationale:**
- macOS native parent-child relationship handles visibility sync automatically
- Windows hide/show together when switching apps
- Each window can be independently positioned and resized

### 2. Procedural Icon Generation
**Decision:** Generate icon at runtime instead of bundling assets.

**Rationale:**
- Reduces build artifacts
- Enables potential future customization

### 3. Message-Passing Concurrency
**Decision:** Use `mpsc::channel` instead of shared state with locks for commands.

**Rationale:**
- Simpler reasoning about state changes
- Eliminates deadlock potential
- Natural fit for command-based architecture
- Works with non-`Send` types like `rodio::Sink`

### 4. Polling-Based Frontend Updates
**Decision:** Frontend windows poll shared state every 200–250ms.

**Rationale:**
- Simple and reliable — no complex event subscription system
- Adequate refresh rate for music player UI
- Easy to add new state fields without wiring up new events

### 5. Auto-Advance Playback
**Decision:** Continuously poll sink status to detect track completion.

**Rationale:**
- `rodio` doesn't provide track completion callbacks
- Polling is lightweight (just checking `sink.empty()`)
- Enables seamless playlist playback

### 6. MP3-Only Support
**Decision:** Currently only supports MP3 files.

**Rationale:**
- Simplifies initial implementation
- MP3 is universally supported
- Easy to extend to other formats via `rodio`'s `Decoder`

## Project Structure

```
player/
├── src/
│   ├── main.rs           # Application entry, tray & window management
│   ├── player.rs         # Audio playback engine
│   ├── icon.rs           # Procedural icon generation
│   └── lrc.rs            # LRC lyrics parser
├── frontend/
│   ├── index.html        # Main window
│   ├── main.js           # Main window logic
│   ├── styles.css        # Main window styles
│   ├── playlist.html     # Playlist window
│   ├── playlist.js       # Playlist window logic
│   ├── playlist.css      # Playlist window styles
│   ├── lyrics.html       # Lyrics window
│   ├── lyrics.js         # Lyrics window logic
│   └── lyrics.css        # Lyrics window styles
├── Cargo.toml            # Dependencies and build config
├── tauri.conf.json       # Tauri application configuration
├── build.rs              # Build-time code generation
├── Info.plist            # macOS application metadata
└── capabilities/         # Tauri security permissions
    └── default.json
```
