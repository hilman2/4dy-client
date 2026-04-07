# 4dy Client

A lightweight native desktop wrapper for the 3CX web client. Built with [Tauri 2](https://tauri.app/) and [Svelte 5](https://svelte.dev/), it hosts your existing 3CX web client in a real Windows window and adds the quality-of-life features that the official PWA is missing: global hotkeys, system tray integration, smart window positioning for incoming calls, and a `tel:` link handler.

> *Independent third-party tool. Not affiliated with, endorsed by, or sponsored by 3CX Ltd. "3CX" is a registered trademark of 3CX Ltd. and is used here only to describe compatibility.*

[![CI](https://github.com/hilman2/4dy-client/actions/workflows/ci.yml/badge.svg)](https://github.com/hilman2/4dy-client/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Windows](https://img.shields.io/badge/platform-Windows-blue)
![Tauri 2](https://img.shields.io/badge/Tauri-2-orange)
![Svelte 5](https://img.shields.io/badge/Svelte-5-red)

## Why this exists

3CX discontinued their Electron-based v18 desktop client and replaced it with a Windows Store app (v20) that you can't even resize. The official PWA option exists but is missing basic things like global hotkeys, which makes it pretty much useless if you need to answer calls while working in other applications. So I built my own native wrapper around the regular web client.

## What it does

**Desktop Client** (Tauri + Svelte)
- Hosts your 3CX web client URL in a native window
- Lives in your system tray, closing the window just hides it
- Global hotkeys that work from any application:
  - `Ctrl+F9` Answer call
  - `Ctrl+F10` Dial number from clipboard
  - `Ctrl+F11` Dial selected text (copies your selection, cleans the number, and dials it)
  - `Ctrl+F12` Hang up
  - `Ctrl+Num0` Open dialer
- Smart window positioning: when an incoming call arrives while the app is minimized, it pops up as a compact dialer in the bottom-right corner
- Remembers window position and size between sessions
- All hotkeys are configurable through a JSON config file

**Callback Popup** (standalone Rust binary)
- A small notification window that appears when someone calls
- Shows the caller's name and number
- Has a button to quickly draft a callback email
- Closes automatically after 4 minutes
- Built as a native Win32 window, no frameworks involved

**Tel Handler** (standalone Rust binary)
- Registers as a `tel:` protocol handler on Windows
- When you click a phone number link on any website, it writes the number to a shared file
- The desktop client picks it up within 500ms and dials the number
- Handles all kinds of German phone number formats (international, local, with or without country code)

**i18n Library** (shared Rust crate)
- Auto-detects the Windows system language
- Supports German, English, French, Italian, Spanish, Dutch, Portuguese, Polish, Czech, Hungarian, Slovak, Slovenian, Romanian, Turkish, and Russian
- Used by all three apps

## Project structure

```
desktop-client/          Tauri app + Svelte frontend
  src/                   Svelte components
  src-tauri/             Rust backend (window management, hotkeys, tray, sidecar orchestration)
callback-popup/          Standalone call notification window
tel-handler/             tel: protocol handler
libs/i18n/               Shared translations
```

## Installing the prebuilt installer

Grab the latest `.msi` or `.exe` from the [Releases page](https://github.com/hilman2/4dy-client/releases) and run it.

> **Heads up: SmartScreen will yell at you.** The installers are not code-signed (no budget for a code-signing certificate on a free custom client). Windows therefore shows an "unrecognized app" / "unknown publisher" warning on first launch. Click **More info** → **Run anyway** to proceed.
>
> Every release lists SHA256 checksums for all artifacts in the release notes. If you want to verify the download before running it:
>
> ```powershell
> Get-FileHash -Algorithm SHA256 .\4dy-Client_*_x64_en-US.msi
> ```
>
> Compare the output against the value in the release notes.

## Prerequisites

You will need a few things installed before you can build this.

**Rust**
- Install [rustup](https://rustup.rs/) and make sure you're on the stable toolchain (`rustup default stable`)

**Node.js**
- Version 18 or newer. Grab the LTS from [nodejs.org](https://nodejs.org/)

**Windows Build Tools**
- Visual Studio Build Tools with the "Desktop development with C++" workload
- WebView2 runtime (already included on Windows 10/11)

## Building from source

The desktop app bundles the two helper tools as sidecars. You need to compile them first.

### Step 1: Build the helper tools

```bash
cd callback-popup
cargo build --release
cd ..

cd tel-handler
cargo build --release
cd ..
```

### Step 2: Copy sidecars into the desktop app

Tauri expects the binaries in a specific location with a platform suffix:

```bash
mkdir -p desktop-client/src-tauri/binaries

cp callback-popup/target/release/callback-popup.exe \
   desktop-client/src-tauri/binaries/callback-popup-x86_64-pc-windows-msvc.exe

cp tel-handler/target/release/tel-handler.exe \
   desktop-client/src-tauri/binaries/tel-handler-x86_64-pc-windows-msvc.exe
```

The suffix has to match your Rust target triple. On ARM Windows that would be `aarch64-pc-windows-msvc` instead.

### Step 3: Install frontend dependencies and build

```bash
cd desktop-client
npm install
npx tauri build
```

The finished installers (MSI and NSIS) end up in `desktop-client/src-tauri/target/release/bundle/`.

If you just want to run it in development mode:

```bash
cd desktop-client
npm install
npx tauri dev
```

### Quick copy-paste version

```bash
cd callback-popup && cargo build --release && cd ..
cd tel-handler && cargo build --release && cd ..
mkdir -p desktop-client/src-tauri/binaries
cp callback-popup/target/release/callback-popup.exe desktop-client/src-tauri/binaries/callback-popup-x86_64-pc-windows-msvc.exe
cp tel-handler/target/release/tel-handler.exe desktop-client/src-tauri/binaries/tel-handler-x86_64-pc-windows-msvc.exe
cd desktop-client && npm install && npx tauri build
```

## Configuration

On first launch you get a setup wizard that asks for your 3CX web client URL. After that, the config lives in `%APPDATA%/4dy-client/config.json`. You can edit it with any text editor or through the tray menu.

```json
{
  "pbx_url": "https://your-company.3cx.de",
  "hotkeys": {
    "dial_clipboard": "Ctrl+F10",
    "dial_selection": "Ctrl+F11",
    "answer_call": "Ctrl+F9",
    "hangup": "Ctrl+F12",
    "open_dialer": "Ctrl+Num0"
  },
  "window": {
    "width": 800,
    "height": 600,
    "remember_position": true,
    "start_minimized": false
  }
}
```

To register as the `tel:` link handler, right-click the tray icon and select "Register tel: handler". Windows will open the default apps settings where you can pick 4dy Client for the `tel` protocol.

## How it works under the hood

The desktop client is a thin native shell around a WebView2 window that loads your configured web client URL. Around that hosted window the wrapper layers the things the underlying PWA doesn't provide: Win32 global hotkeys (registered through Tauri's `global-shortcut` plugin), a tray icon with menu, and smart window positioning for incoming calls.

The `tel:` handler and the desktop client communicate through a simple file. When you click a `tel:` link, `tel-handler.exe` writes the cleaned number to `%APPDATA%/4dy-client/dial.txt`. The desktop app polls this file every 500ms and dials the number when it appears.

Phone number normalization handles the mess of German phone number formats. Things like `+49 (0) 8031 / 2626-0`, `0049 89 22334455`, or `(0151) 555-01-00` all get cleaned up to something the dialer can use.

## Limitations

- Windows only (relies on Win32 APIs for the popup window, clipboard access, and window management)
- The file-based IPC between tel-handler and the desktop app is simple but has a ~500ms delay
- Future versions of the hosted web client may change their DOM structure; in that case the selectors used by the wrapper may need updating

## Tests

The Rust crates have unit tests for the pure logic (phone number normalisation, config parsing, i18n lookup). Tauri/Win32 paths are not exercised by the test suite.

```bash
# Rust unit tests (per crate)
cargo test --manifest-path libs/i18n/Cargo.toml
cargo test --manifest-path callback-popup/Cargo.toml
cargo test --manifest-path tel-handler/Cargo.toml
cargo test --manifest-path desktop-client/src-tauri/Cargo.toml --lib

# Frontend (vitest, jsdom)
cd desktop-client
npm install
npm test
```

`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are enforced by CI.

## Releases

Tagged builds are produced by GitHub Actions:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The `release.yml` workflow then builds both sidecars, stages them with the right `x86_64-pc-windows-msvc` suffix, runs `tauri build`, and attaches the resulting `.msi` and NSIS `.exe` to a draft GitHub release. Publish the draft once you've smoke-tested the installers.

## Contributing

Pull requests welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for the short version: run the tests, run `cargo fmt`, keep PRs focused.

## License

[MIT](LICENSE) © Manuel Hilgert
