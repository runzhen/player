# QQPlayer

A minimalist music player for macOS built with Rust and Tauri v2.

## Installation

### Download

Download the latest release from the [Releases page](../../releases):

| Platform | File |
|----------|------|
| macOS Apple Silicon (M1/M2/M3/M4) | `QQPlayer-macos-arm64.zip` |
| macOS Intel | `QQPlayer-macos-x86_64.zip` |

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

## Usage

1. **Launch:** Run the application; the main player window and playlist window appear
2. **Import Music:**
   - Click "Import File" to add a single MP3
   - Click "Import Folder" to add all MP3s from a folder
3. **Playback Controls:**
   - **⏯ Play/Pause** — Start or pause playback
   - **⏹ Stop** — Stop playback
   - **⏮⏭ Previous/Next** — Navigate tracks
   - **🔁 Play Mode** — Cycle through: cycle, repeat one, shuffle
4. **Playlist Window** — Toggle with the 🎵 button; supports multi-select, drag reorder, right-click to remove
5. **Lyrics Window** — Toggle with the 🎤 button; auto-syncs `.lrc` files placed next to MP3s
6. **Minimize to Tray** — Close the main window and choose "Yes" to minimize to system tray

## Building from Source

### Prerequisites
- Rust toolchain (1.70+)
- macOS (primary target platform)

### Development

```bash
# Install Tauri CLI
cargo install tauri-cli --locked

# Run in development mode
cargo tauri dev
```

### Release Build

```bash
# Build optimized app bundle
cargo tauri build
```

The app bundle will be created in `target/release/bundle/macos/`.

## Architecture

See [DESIGN.md](DESIGN.md) for architecture details, component breakdown, threading model, and design decisions.

## License

This project is provided as-is for educational and personal use.
