# Development

See [Installation](installation.md) to build and run the game locally.

## Development commands

- `make help` - Show all available commands
- `make run` - Run the game in debug mode
- `make run-dev` - Open the developer menu
- `make run-release` - Run the game in release mode (optimized)
- `make build` - Build the project (debug)
- `make build-release` - Build the project (release)
- `make test` - Run tests
- `make fmt` - Check code formatting
- `make fmt-fix` - Fix code formatting
- `make clippy` - Run clippy linter
- `make check` - Run all checks (fmt, clippy, build, test)
- `make ci` - Run CI checks (same as CI pipeline)
- `make clean` - Clean build artifacts
- `make install-hooks` - Install pre-commit hook

Install [Gitleaks](https://github.com/gitleaks/gitleaks/releases/tag/v8.30.1) 8.30.1 or newer and run `make install-hooks` to install or refresh the pre-commit hook. It scans staged changes for secrets before running the Rust checks.

Use the developer menu to choose a level and ship, then start at that mission's briefing. **Open test shop** starts the selected ship with 999 scrap and 999 rare metal to test purchases.

## Regenerating model sprites

Run `cargo run --features bake-sprites -- --bake-sprites` from the repository root after editing a model in `art/sprite-models`. The command opens a Macroquad window to render the models, then exits. Commit the updated PNGs in `assets/sprites` alongside the model changes. Release packages include those PNGs and the four animated player models in `assets/models`.

## Security checks

The Security workflow runs zizmor, Gitleaks, and cargo-audit on pushes and pull requests targeting `main`. It also checks `Cargo.lock` weekly for newly published RustSec advisories.

- `zizmor --offline .` audits GitHub Actions workflows locally without a GitHub token.
- `gitleaks git --verbose --redact --log-opts=HEAD .` scans the current branch's history. Use `--log-opts=origin/main..HEAD` to scan only the local commits ahead of `origin/main`.
- `cargo audit` checks `Cargo.lock` against the RustSec advisory database. Vulnerability findings fail CI. Informational advisories, such as unmaintained dependencies, appear as warnings.

## Publishing a release

1. Run `make bump-version VERSION=0.6.1` with the intended version and commit `Cargo.toml` and `Cargo.lock`. The command uses Python 3 and Cargo.
2. After the commit passes CI, tag it with the same version prefixed by `v`, such as `v0.6.1`, and push the tag. The release workflow checks the tag against `Cargo.toml` before building.
3. Wait for the **Release** workflow to succeed. It creates a draft with generated release notes or adds files to an existing draft for the same tag. It attaches a Linux AppImage, a Linux tarball, and ZIP packages for Intel and Apple Silicon Macs.
4. In GitHub **Releases**, edit the draft's title and describe the player-facing changes. A new draft for a tag with a prerelease suffix such as `-beta.1` is marked as a prerelease automatically. If you created the draft yourself, set the prerelease checkbox as appropriate.
5. Once the packages are attached and the notes are ready, click **Publish release**.

Packaged builds display the version from the release tag. The updater uses the latest stable release from GitHub's public API and verifies the downloaded asset against its GitHub SHA-256 digest.

### macOS update signing

The release build embeds the public key from the `RUST_IN_SPACE_UPDATE_PUBLIC_KEY` repository variable. The draft preparation job uses the `RUST_IN_SPACE_UPDATE_PRIVATE_KEY` secret from the `macos-release` environment to sign each Mac ZIP. If the key is replaced, players need to install the build with the new public key manually. Without a configured key pair, the game directs Mac players to GitHub for updates.

The release key verifies updates inside the game. Apple Developer ID and notarization are separate and determine how Gatekeeper treats downloaded builds.

## Project structure

- `src/main.rs` runs the game loop and handles state transitions.
- `src/game/` handles mission progression, upgrades, player controls, enemies, physics, and collisions.
- `src/components.rs` defines game entities and state.
- `src/draw.rs` renders the game and UI. `src/resources.rs`, `src/ship_model.rs`, and `src/model_sprite.rs` handle game assets and models.
- `src/systems.rs` handles saves, mission definitions, loot generation, and screen wrapping.
- `src/lan/` handles host and guest sessions, transport, guest ship prediction, and LAN UI.
- `src/update/` handles in-game updates on Linux and macOS. `packaging/` builds the distributable packages.
- `src/localization.rs` provides translations. `src/audio.rs` handles audio playback and settings.
- `assets/` contains game assets. See the [audio guide](../scripts/audio/README.md).
- `Makefile` provides development commands, and `scripts/pre-commit` runs checks before commits.
- `rustfmt.toml` and `clippy.toml` configure formatting and linting.

## Dependencies

The game runs on Macroquad. It uses Serde and serde_json for saves, MessagePack for LAN snapshots, glTF for models, and Fluent and sys-locale for localization. See [Cargo.toml](../Cargo.toml) for versions and build dependencies.

## Localization

English is the project's development language. Use English for code identifiers, comments, and developer documentation.

Use message IDs for player-facing text. Add the English source strings to [en.ftl](../assets/locales/en.ftl) and translations to the corresponding catalogs in [assets/locales](../assets/locales). The game embeds these [Fluent](https://projectfluent.org/fluent/guide/) catalogs.
To add a language, copy `en.ftl`, translate its values, then register the file and language tag in `CATALOGS` and add a saved language variant in [src/localization.rs](../src/localization.rs).
Keep the message IDs and variable names consistent across catalogs. Placeholders such as `{ $level }` and `{ $name }` can move within a message.
The game uses English for missing messages and uses the system locale when the save has no language preference.

Check that the font covers the translated text. The built-in CJK fonts cover all shipped messages and commonly used characters. Run `make test` to check catalog coverage and formatting, and check the translated screens in-game.

## Credits & licenses

### Code

The code is licensed under the [MIT License](../LICENSE).

### Assets

- [Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P) by CodeMan38 is licensed under the SIL Open Font License (OFL).
- The Japanese, Simplified Chinese, and Korean UI fonts are renamed subsets of [Noto Sans CJK](https://github.com/notofonts/noto-cjk), distributed under the [SIL Open Font License](../assets/fonts/OFL-Rust-In-Space-CJK.txt).
- Graphics include AI-generated and original artwork.
