# Development

See [Installation](installation.md) to build and run the game locally.

## Development commands

- `make help` - Show all available commands
- `make run` - Run the game in debug mode
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

The pre-commit hook automatically runs the same checks as CI before each commit.

## Publishing a release

Push a version tag such as `v0.1.0` from a commit that has passed CI. The release workflow attaches a Linux AppImage, a Linux tarball, and ZIP packages for Intel and Apple Silicon Macs. A tag with a prerelease suffix, such as `-beta.1`, is published as a prerelease.

## Project structure

- `src/main.rs` runs the game loop and handles state transitions.
- `src/game/` handles mission progression, upgrades, player controls, enemies, physics, and collisions.
- `src/components.rs` defines game entities and state.
- `src/draw.rs` renders the game and UI. `src/resources.rs`, `src/ship_model.rs`, and `src/model_sprite.rs` handle game assets and models.
- `src/systems.rs` handles saves, mission definitions, loot generation, and screen wrapping.
- `src/localization.rs` provides translations. `src/audio.rs` handles audio playback and settings.
- `assets/` contains game assets. See the [audio guide](../scripts/audio/README.md).
- `Makefile` provides development commands, and `scripts/pre-commit` runs checks before commits.
- `rustfmt.toml` and `clippy.toml` configure formatting and linting.

## Dependencies

The game runs on Macroquad. It uses Serde and serde_json for saves, glTF for models, and sys-locale for language detection. See [Cargo.toml](../Cargo.toml) for versions and build dependencies.

## Credits & licenses

### Code

The code is licensed under the [MIT License](../LICENSE).

### Assets

- [Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P) by CodeMan38 is licensed under the SIL Open Font License (OFL).
- Graphics include AI-generated and original artwork.
