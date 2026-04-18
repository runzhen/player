# QQPlayer

A minimalist music player for macOS that lives entirely in your system tray. Built with Rust and Tauri for maximum performance and minimal resource usage.

## Overview

QQPlayer is a lightweight, tray-only music player designed for simplicity and efficiency. It runs in the background without a traditional window, allowing you to control playback directly from the macOS menu bar.

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────┐
│          Tauri Application (main.rs)        │
│  ┌───────────────────────────────────────┐  │
│  │   System Tray Icon & Menu UI          │  │
│  │   - Dynamic menu updates              │  │
│  │   - User interaction handling         │  │
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
│  │   │  - Rodio audio backend          │ │  │
│  │   └─────────────────────────────────┘ │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### Component Breakdown

#### 1. Main Application (`main.rs`)

**Responsibilities:**
- Initialize Tauri application framework
- Create and manage system tray icon
- Build and dynamically update menu UI
- Handle user interaction events
- Route commands to player thread via message passing

**Key Design Patterns:**
- **Message Passing:** Uses `mpsc::channel` to communicate between UI thread and player thread
- **Event-Driven:** Menu events trigger commands that are sent to the player
- **State Synchronization:** Menu is rebuilt after each state change to reflect current playback status

**Command Flow:**
```rust
User Click → Menu Event → PlayerCommand → Player Thread → State Update → Menu Rebuild
```

#### 2. Audio Player (`player.rs`)

**Responsibilities:**
- Manage audio playback using the `rodio` library
- Maintain playlist of audio files
- Handle play, pause, stop, and next operations
- Auto-advance to next track when current finishes

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

**Responsibilities:**
- Procedurally generate application icon at runtime
- Create a recognizable music player icon (blue circle with white music note)

**Design Choice:** Instead of bundling static icon files, the icon is generated programmatically, reducing asset dependencies and allowing for potential customization.

### Threading Model

The application uses a **multi-threaded architecture** to prevent UI blocking:

1. **Main Thread:** Runs Tauri event loop and handles tray menu interactions
2. **Player Thread:** Owns the `AudioPlayer` and processes playback commands
3. **Auto-Advance Loop:** Continuously checks if the current track has finished and automatically plays the next track

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
│ Menu Event       │─────>│ File Dialog      │ (if import)
│ Handler          │      │ (rfd)            │
└──────┬───────────┘      └─────┬────────────┘
       │                        │
       ▼                        ▼
┌──────────────────────────────────┐
│ PlayerCommand (enum)             │
│ - Play/Pause/Stop/Next           │
│ - LoadFile(PathBuf)              │
│ - LoadFolder(PathBuf)            │
└────────┬─────────────────────────┘
         │ mpsc::send
         ▼
┌──────────────────────────────────┐
│ Player Thread Loop               │
│ - cmd_rx.try_recv()              │
│ - Process command                │
│ - Update AudioPlayer state       │
└────────┬─────────────────────────┘
         │
         ▼
┌──────────────────────────────────┐
│ rebuild_menu()                   │
│ - Read current player state      │
│ - Build new menu with updated UI │
│ - Replace tray menu              │
└──────────────────────────────────┘
```

## Technical Stack

- **[Tauri](https://tauri.app/)** v2 - Cross-platform desktop application framework
- **[rodio](https://github.com/RustAudio/rodio)** - Pure Rust audio playback library
- **[image](https://github.com/image-rs/image)** - Image processing for icon generation
- **[walkdir](https://github.com/BurntSushi/walkdir)** - Recursive directory traversal
- **[rfd](https://github.com/PolyMeilex/rfd)** - Native file dialog support
- **[serde](https://serde.rs/)** - Serialization framework

## Key Design Decisions

### 1. Tray-Only Interface
**Decision:** No main window, all interaction through system tray menu.

**Rationale:**
- Reduces UI complexity and resource usage
- Natural fit for background music player
- Follows macOS design patterns for utility applications
- Eliminates need for frontend framework (HTML/JS/CSS)

### 2. Procedural Icon Generation
**Decision:** Generate icon at runtime instead of bundling assets.

**Rationale:**
- Reduces build artifacts
- Enables potential future customization
- Demonstrates Rust image processing capabilities

### 3. Message-Passing Concurrency
**Decision:** Use `mpsc::channel` instead of shared state with locks.

**Rationale:**
- Simpler reasoning about state changes
- Eliminates deadlock potential
- Natural fit for command-based architecture
- Works with non-`Send` types like `rodio::Sink`

### 4. Auto-Advance Playback
**Decision:** Continuously poll sink status to detect track completion.

**Rationale:**
- `rodio` doesn't provide track completion callbacks
- Polling is lightweight (just checking `sink.empty()`)
- Enables seamless playlist playback

### 5. MP3-Only Support
**Decision:** Currently only supports MP3 files.

**Rationale:**
- Simplifies initial implementation
- MP3 is universally supported
- Easy to extend to other formats via `rodio`'s `Decoder`

## Installation

### Download

Download the latest release from the [Releases page](../../releases):

| Platform | File |
|----------|------|
| macOS Apple Silicon (M1/M2/M3/M4) | `QQPlayer-macos-arm64.zip` |
| macOS Intel | `QQPlayer-macos-x86_64.zip` |
| Windows | `QQPlayer_x.x.x_x64-setup.exe` |

### macOS — First Launch

Since the app is not signed with an Apple Developer certificate, macOS will block it on first launch. To open it:

1. Unzip the downloaded `.zip` file
2. Drag `QQPlayer.app` to `/Applications`
3. **Right-click** (or Control-click) on `QQPlayer.app` and select **"Open"**
4. Click **"Open"** in the dialog that appears

You only need to do this once. After that, the app opens normally.

> **Alternative:** If you still see "damaged" errors, run this in Terminal:
> ```bash
> xattr -cr /Applications/QQPlayer.app
> ```

## Building and Running

### Prerequisites
- Rust toolchain (1.70+)
- macOS (primary target platform)

### Build

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

### Run

```bash
# Run directly
cargo run

# Run release binary
./target/release/qqplayer
```

### Create macOS App Bundle

```bash
# Using Tauri CLI
cargo tauri build
```

The app bundle will be created in `target/release/bundle/macos/`.

## Usage

1. **Launch:** Run the application; an icon will appear in your menu bar
2. **Import Music:** 
   - Click "📁 Import File…" to add a single MP3
   - Click "📂 Import Folder…" to add all MP3s from a folder
3. **Playback Controls:**
   - **⏯ Play:** Start or resume playback
   - **⏸ Pause:** Pause current track
   - **⏹ Stop:** Stop playback and reset
   - **⏭ Next:** Skip to next track
4. **View Info:** The menu displays current track name and playlist count

## Project Structure

```
qqplayer.rs/
├── src/
│   ├── main.rs           # Application entry, tray management
│   ├── player.rs         # Audio playback engine
│   └── icon.rs           # Procedural icon generation
├── Cargo.toml            # Dependencies and build config
├── tauri.conf.json       # Tauri application configuration
├── build.rs              # Build-time code generation
├── Info.plist            # macOS application metadata
└── capabilities/         # Tauri security permissions
    └── default.json
```

## Future Enhancements

- [ ] Support for additional audio formats (FLAC, OGG, WAV)
- [ ] Persistent playlist (save/load between sessions)
- [ ] Global keyboard shortcuts
- [ ] Shuffle and repeat modes
- [ ] Volume control
- [ ] Album art display
- [ ] Metadata parsing and display (ID3 tags)
- [ ] Mini player window (optional)

## License

This project is provided as-is for educational and personal use.

---

Built with ❤️ in Rust
