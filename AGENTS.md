# AGENTS.md

This file defines project-specific guidance for coding agents working in this repository.

## Project Snapshot

- Project: `rust-in-space`
- Language: Rust (edition 2021)
- Framework: `macroquad`
- Entry point: `src/main.rs`
- Goal: mission-based 2D space shooter with gameplay updates in `src/game.rs`, rendering in `src/draw.rs`, and data models in `src/components.rs`.

## Key File Map

- `src/main.rs`: main loop and game state transitions
- `src/game.rs`: gameplay update systems and high-level rendering entrypoints
- `src/components.rs`: core structs/enums (game entities and state)
- `src/draw.rs`: drawing/render helpers
- `src/systems.rs`: utility systems (save/load, generation, wrapping, etc.)
- `src/resources.rs`: asset/resource loading
- `src/localization.rs`: language and strings
- `assets/`: textures/fonts
- `highscore.json`: persisted score data

## Working Rules

- Keep changes minimal and focused on the requested behavior.
- Prefer extending existing systems/functions over adding new architecture.
- Do not silently change gameplay balance unless explicitly requested.
- Preserve existing keyboard controls and game state flow unless task requires changes.
- Keep code warnings-free under clippy settings (`-D warnings` in Makefile).

## Build And Validation

Use these commands (via `Makefile`) after code changes:

1. `make fmt`
2. `make clippy`
3. `make build`
4. `make test`

For full local parity with CI: `make ci`.

## Style Conventions

- Follow idiomatic Rust and current module organization.
- Keep functions small and readable; extract helpers when logic gets dense.
- Avoid `unwrap`/`expect` in runtime gameplay code unless there is a clear invariant.
- Prefer explicit names for gameplay constants; avoid unexplained magic numbers.
- Add short comments only for non-obvious logic (physics tuning, spawn math, timing edge cases).

## Gameplay Change Safety

When changing gameplay logic, verify:

1. Mission progression still works (`Briefing -> Playing -> MissionSuccess/GameOver`).
2. Pause/resume behavior remains intact.
3. Collision paths still compile and run for ship/enemy/loot/projectiles.
4. Score and high score persistence still behave correctly.
5. Localization-dependent UI text still renders.

## Rendering And Assets

- Reuse already-loaded textures/resources from `Resources` instead of loading ad hoc.
- Keep draw code in rendering modules (`draw.rs` / render helpers), not in update systems.
- Respect existing coordinate and camera assumptions used by Macroquad draw calls.

## Definition Of Done (For Agent PRs/Commits)

A change is complete when:

1. Requested behavior is implemented.
2. Formatting/lint/build/tests pass (or failures are documented with reason).
3. No unrelated files are modified.
4. Any gameplay-affecting tradeoffs are stated clearly in the final summary.
