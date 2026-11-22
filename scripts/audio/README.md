# Audio Generation Workflow

This folder documents how runtime audio assets are generated and finalized.

For each asset, record:

- asset name
- prompt/spec
- tool/model
- seed if available
- source export path
- post-processing steps
- final runtime filename

## Runtime Targets

- The game tries `.ogg` first and falls back to `.wav` for each sound.
- Sound effects live in `assets/audio/sfx/` and music in `assets/audio/music/`.
- Keep clips short and readable in gameplay
- Normalize loudness before exporting runtime files

Run `python3 scripts/audio/generate_audio.py` from the repository root to write procedural WAV clips into those directories. Existing OGG clips take precedence over WAV clips with the same name.

## Initial Batch

- `player_shot`
- `enemy_shot`
- `explosion_small`
- `pickup_scrap`
- `ship_hit`
- `engine_loop`
- `menu_loop`
- `gameplay_loop`

## Style Guide

- retro arcade sci-fi
- punchy transients
- restrained ambience
- low noise floor
- clear player/enemy differentiation
