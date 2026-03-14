# Audio Plan

## Scope

Phase 1 targets desktop only:

- Linux
- macOS
- Windows

The runtime audio stack will use `macroquad::audio`. Audio assets will be shipped as local files with the game.

## Goals

1. Add a cross-platform audio layer without destabilizing the current game loop.
2. Keep audio playback decisions centralized instead of scattering raw sound calls through gameplay code.
3. Make asset generation reproducible with prompts/specs tracked in the repository.
4. Ship the first version with optional audio assets so development can proceed before the full sound pack exists.

## Architecture

### Runtime

- `src/audio.rs`
  - Owns playback policy.
  - Exposes `AudioCue` for one-shot effects.
  - Exposes `MusicTrack` for long-running loops.
  - Tracks current music, pause state, and engine loop state.
- `src/resources.rs`
  - Loads audio assets alongside textures.
  - Uses optional audio handles so missing files do not break the game during rollout.
- `src/main.rs`
  - Owns high-level state transitions.
  - Switches menu/gameplay music and pause behavior.
- `src/game.rs`
  - Will emit gameplay audio cues in Phase 2 rather than calling audio APIs directly.
- `src/components.rs` and `src/systems.rs`
  - Persist audio settings in the existing save file.

### Assets

- `assets/audio/sfx/`
  - Runtime one-shot and looping sound effects.
- `assets/audio/music/`
  - Runtime music loops.
- `assets/audio/source/`
  - Higher-quality source exports, generated stems, or post-processing inputs.
- `scripts/audio/README.md`
  - Generation prompts/specs and normalization rules.

## Asset List

### Core SFX

- `ui_move.ogg`
- `ui_confirm.ogg`
- `player_shot.ogg`
- `enemy_shot.ogg`
- `explosion_small.ogg`
- `explosion_big.ogg`
- `pickup_scrap.ogg`
- `pickup_health.ogg`
- `shield_on.ogg`
- `shield_hit.ogg`
- `ship_hit.ogg`
- `mission_success.ogg`
- `game_over.ogg`
- `engine_loop.ogg`

### Music

- `menu_loop.ogg`
- `gameplay_loop.ogg`

## Generation Direction

Style target:

- retro arcade sci-fi
- compact and readable in a busy mix
- no heavy reverb tails
- clear separation between player and enemy shots
- engine loop should be unobtrusive over long sessions

Technical targets:

- runtime delivery format: `ogg`
- editable/source format: `wav`
- keep assets mono unless stereo materially improves the result
- normalize loudness before committing runtime files

For each generated asset, track:

- prompt/spec
- tool/model used
- seed if available
- post-processing notes
- exported filename

## Phases

### Phase 1: Infrastructure

- Add `src/audio.rs`.
- Extend `Resources` with optional audio assets.
- Persist audio settings in the existing save file.
- Add audio asset directory structure and generation docs.
- Wire menu/gameplay music scaffolding and pause-safe loop control.
- Keep runtime behavior non-breaking when audio files are absent.

Acceptance criteria:

- The game builds and runs without requiring audio files to exist yet.
- Save file loading remains backward-compatible with existing `highscore.json`.
- The audio layer can start/stop music and engine loops without exposing raw audio calls to the rest of the code.

### Phase 2: Core Gameplay Cues

- Emit player/enemy shot cues.
- Emit explosion, pickup, hit, shield, success, and game-over cues.
- Route game events through `AudioCue`.

Acceptance criteria:

- Core interactions have audible feedback.
- Audio logic remains centralized in the audio layer.

### Phase 3: Settings and UI

- Add settings UI for master/music/SFX volume and mute.
- Persist updated settings back into the save file.
- Tune music/SFX balance.

Acceptance criteria:

- Settings survive restarts.
- Volumes clamp correctly and mute is respected.

### Phase 4: Asset Production

- Generate and curate the first full desktop sound pack.
- Replace placeholders with normalized final assets.
- Validate playback on Linux, macOS, and Windows.

Acceptance criteria:

- Runtime assets exist for the planned cue list.
- Mixing is consistent across the three desktop targets.

## Risks

- Continuous sounds will glitch if retriggered every frame instead of being state-driven.
- Audio balance can degrade quickly if generated assets are not normalized to a consistent target.
- Save schema changes can overwrite future settings if score persistence does not preserve the full payload.
- Platform packaging can fail if audio assets are referenced with unstable paths.
