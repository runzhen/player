.PHONY: build build-arm build-x86 build-universal build-windows build-linux \
       install-target-arm install-target-x86 install-target-windows install-target-linux \
       clean run

# Default: build for current architecture
build:
	cargo build --release

# --- macOS Apple Silicon (ARM) ---
install-target-arm:
	rustup target add aarch64-apple-darwin

build-arm: install-target-arm
	cargo build --release --target aarch64-apple-darwin

# --- macOS Apple Intel (x86) ---
install-target-x86:
	rustup target add x86_64-apple-darwin

build-x86: install-target-x86
	cargo build --release --target x86_64-apple-darwin

# --- macOS Universal binary ---
build-universal: build-arm build-x86
	mkdir -p target/universal-apple-darwin/release
	lipo -create \
		target/aarch64-apple-darwin/release/qqplayer \
		target/x86_64-apple-darwin/release/qqplayer \
		-output target/universal-apple-darwin/release/qqplayer

# --- Windows (x86_64, cross-compile from macOS) ---
install-target-windows:
	rustup target add x86_64-pc-windows-msvc

build-windows: install-target-windows
	cargo build --release --target x86_64-pc-windows-msvc

# --- Ubuntu Linux (x86_64, native build) ---
# Run on an Ubuntu machine or VM
install-target-linux:
	rustup target add x86_64-unknown-linux-gnu
	sudo apt-get update
	sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libasound2-dev pkg-config

build-linux: install-target-linux
	cargo build --release --target x86_64-unknown-linux-gnu

clean:
	cargo clean

run:
	cargo run
