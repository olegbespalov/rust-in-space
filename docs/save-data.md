# Save data

The game stores your high score, language, and audio settings in a per-user `highscore.json`:

- **Linux**: `$XDG_DATA_HOME/rust-in-space/highscore.json`. If `XDG_DATA_HOME` is unset or relative, the game uses `~/.local/share/rust-in-space/highscore.json`.
- **macOS**: `~/Library/Application Support/Rust in Space/highscore.json`.
- **Windows (native builds)**: `%LOCALAPPDATA%\Rust in Space\highscore.json`.

The JSON object includes `high_score` (a nonnegative integer), `language` (`english`, `russian`, or `german`), and `audio` (volume and mute settings). If the game cannot load a save, it starts with a score of zero and the default audio settings. If the save has no language preference, the game selects Russian or German based on the system locale and uses English for other locales.

If the per-user save is missing, the game imports a valid `highscore.json` from the working directory and leaves the original file in place. To migrate a save from an older development or standalone build, launch the game once from the directory containing that file. Existing AppImage and macOS app saves already use the per-user paths above.
