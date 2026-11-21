# Installation

## Download a release

Download the package for your platform from [GitHub Releases](https://github.com/olegbespalov/rust-in-space/releases).

- **Linux (64-bit Intel or AMD)**. Download the `x86_64-unknown-linux-gnu.AppImage` file, make it executable with `chmod +x ./rust-in-space-*.AppImage`, and open it. An AppImage integration tool can add the game and its icon to your application menu. The Linux build targets Ubuntu 22.04 and uses the system ALSA runtime (`libasound2` on Ubuntu 22.04). If FUSE is unavailable, run the file with `--appimage-extract-and-run`. The `.tar.gz` is also available. Extract it and keep `assets` beside the executable.
- **macOS (Intel or Apple Silicon)**. Download the `x86_64-apple-darwin.zip` for Intel or `aarch64-apple-darwin.zip` for Apple Silicon. Unzip it, move **Rust in Space.app** to Applications, and open it. The bundle is ad hoc signed. macOS Gatekeeper may block a downloaded copy because it has no Apple notarization.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Cargo (comes with Rust)

## Building and running

1. Clone or download this repository.
2. Open a terminal in the repository root and run the game.
   ```bash
   make run
   ```
3. For an optimized build, use release mode.
   ```bash
   make run-release
   ```

See [Development](development.md) for build checks and release packaging.
