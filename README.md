# Rust in Space

<div align="center">
  <img src="assets/logo.png" alt="Rust in Space" width="400">
</div>

**English** · [Русский](docs/readme/README.ru.md) · [Deutsch](docs/readme/README.de.md) · [Español](docs/readme/README.es.md) · [日本語](docs/readme/README.ja.md) · [简体中文](docs/readme/README.zh-Hans.md) · [한국어](docs/readme/README.ko.md)

> *"You're a mercenary pilot, drifting through the outer rim of civilized space. Out here, the only law is what you can enforce with your ship's cannons. The corporations pay well for rust piles and rare metals mined from the asteroid fields, but they don't tell you about the enemy patrols or the void pirates that call these deep space sectors home.*
>
> *Every mission takes you deeper into the unknown. Every asteroid you crack could be your last. But the credits are good, and in this part of the galaxy, that's all that matters. Strap in, pilot. The void awaits."*

Fly a 2D space shooter campaign solo or with friends. Break up asteroids, fight enemy patrols, and collect scrap and rare metals to upgrade your ship between missions. Choose one of three difficulty levels.

## Download and play

[**Download the latest release**](https://github.com/olegbespalov/rust-in-space/releases/latest)

- **Linux (64-bit Intel or AMD)**. Download the `.AppImage`, make it executable, and open it.
- **macOS**. Download the `aarch64-apple-darwin.zip` build for Apple Silicon or `x86_64-apple-darwin.zip` for Intel. Unzip the file and open **Rust in Space.app**.

See the [installation guide](docs/installation.md) for platform-specific setup.

## Controls

- **Main menu**. Use ↑ / ↓ to select an item, ← / → to change a setting, and Enter to confirm. Select **Language** to change the game language.
- **Briefing**. Press Space to launch.
- **In flight**. Use ← / → to rotate, ↑ to thrust, Space to shoot, and Esc to pause in solo play.
- **Paused**. Press Enter to resume or Esc to return to the main menu.
- **After a mission**. Press Enter to open the **Mercenary Market**. Use ↑ / ↓ to choose an upgrade, then Enter to buy it or continue to the next briefing.
- **Solo game over**. Press R to start a fresh run with the same ship and difficulty, once per campaign. Press Enter to return to the main menu.

The current source supports English, Russian, German, Spanish, Japanese, Simplified Chinese, and Korean. Available game languages depend on the release.

## Play with friends

Up to four pilots can play the campaign together on the same IPv4 local network. Choose **Local Network** in the main menu to host or join a game. See the [co-op setup guide](docs/local-network.md).

## Player guides

- [Gameplay](docs/gameplay.md) covers missions, difficulty, combat, and ship upgrades.
- [Enemies](docs/enemies.md) and [Loot](docs/loot.md) cover opponents, pickups, and scoring.
- [Save data](docs/save-data.md) explains where the game stores your high score and settings.

[Development guide](docs/development.md) · [License](LICENSE) · [README and translations on GitHub](https://github.com/olegbespalov/rust-in-space#readme)
