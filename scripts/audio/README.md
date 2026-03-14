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

- Runtime format: `ogg`
- Source/edit format: `wav`
- Keep clips short and readable in gameplay
- Normalize loudness before exporting runtime files

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
