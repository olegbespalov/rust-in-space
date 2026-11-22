# Save data

The game stores your high score, commander name, language, and audio settings in a per-user `highscore.json`:

- **Linux**: `$XDG_DATA_HOME/rust-in-space/highscore.json`. If `XDG_DATA_HOME` is unset or relative, the game uses `~/.local/share/rust-in-space/highscore.json`.
- **macOS**: `~/Library/Application Support/Rust in Space/highscore.json`.
- **Windows (native builds)**: `%LOCALAPPDATA%\Rust in Space\highscore.json`.

The JSON object includes `high_score` (a nonnegative integer), `player_name`, `language`, and `audio` (volume and mute settings). The language values are `english`, `russian`, `german`, `spanish`, `japanese`, `chinese` (Simplified Chinese), and `korean`. If the game cannot load a save, it starts with a score of zero and the default audio settings. If the save has no language preference, the game uses the system language when supported and falls back to English.

If the save has no `player_name`, the game asks for your commander name at startup. Confirm the default `player1` or enter a name to save.

If the per-user save is missing, the game imports a valid `highscore.json` from the working directory and leaves the original file in place. To migrate a save from an older development or standalone build, launch the game once from the directory containing that file. Existing AppImage and macOS app saves already use the per-user paths above.
