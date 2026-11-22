# Rust in Space

<div align="center">
  <img src="assets/logo.png" alt="Logo" width="400">

  [![Latest Release](https://img.shields.io/github/v/release/olegbespalov/rust-in-space)](https://github.com/olegbespalov/rust-in-space/releases/latest)
  ![CI](https://github.com/olegbespalov/rust-in-space/workflows/CI/badge.svg)
</div>

A mission-based 2D space shooter built with Rust and Macroquad. Blast through asteroid fields, fight enemy patrols, and spend salvage on ship upgrades between missions. Play in English, Russian, or German across three difficulty levels.

## Quick start

- Download the Linux or macOS package from [GitHub Releases](https://github.com/olegbespalov/rust-in-space/releases).
- To run from source, install [Rust](https://www.rust-lang.org/tools/install) and run `make run` from the repository root.

See [Installation](docs/installation.md) for platform-specific setup.

## Controls

- In the **main menu**, use Up/Down to select an item, Left/Right to change a setting, and Enter to start.
- At the **briefing**, press Space to launch.
- **In flight**, use Left/Right to rotate, Up to thrust, Space to shoot, and Esc to pause.
- **While paused**, press Enter to resume or Esc to return to the main menu.
- **After a mission**, press Enter to open the Upgrade Bay. Use Up/Down to select an item and Enter to buy or continue.
- At **game over**, press Enter to return to the main menu.

## Local network co-op

Up to four pilots can play the campaign together over IPv4. See the [local
network guide](docs/local-network.md) for setup and troubleshooting.

## Documentation

- [Gameplay](docs/gameplay.md) covers missions, difficulty, combat, and upgrades.
- [Enemies](docs/enemies.md) and [Loot](docs/loot.md) list enemy stats, scoring, pickups, and drop rates.
- [Save data](docs/save-data.md) explains where scores and settings are stored and how older saves are imported.
- [Development](docs/development.md) covers build commands, project layout, release packaging, and asset credits.

The [online documentation](https://github.com/olegbespalov/rust-in-space/tree/main/docs) is also available when reading this README from a release package.

The code is available under the MIT License.
