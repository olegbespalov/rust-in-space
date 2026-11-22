# AGENTS.md

This file defines project-specific guidance for coding agents working in this repository.

## Project Snapshot

- Project: `rust-in-space`
- Language: Rust (edition 2021)
- Framework: `macroquad`
- Entry point: `src/main.rs`
- Goal: mission-based 2D space shooter with solo and LAN co-op campaigns.

## Key File Map

- `src/main.rs`: main loop, solo state transitions, and developer menu
- `src/game/mod.rs`: shared simulation, mission progression, and upgrades
- `src/game/`: movement and weapons in `player.rs`, enemies and bosses in `npc.rs`, collisions and loot collection in `physics.rs`, and gameplay tuning in `constants.rs`
- `src/components.rs`: game entities, states, and save data types
- `src/draw.rs` and `src/resources.rs`: rendering and asset loading
- `src/systems.rs`: save/load, mission definitions, loot drops, and screen wrapping
- `src/lan/`: host simulation and session state, network transport, guest movement prediction, and co-op UI
- `src/localization.rs` and `assets/locales/`: language detection, Fluent formatting, and translation catalogs
- `src/update/`, `packaging/`, and `.github/workflows/release.yml`: GitHub Releases updater and platform packaging

See [Development](docs/development.md) for local setup, localization, and releases, and [Save data](docs/save-data.md) for platform-specific `highscore.json` locations.

## Working Rules

- Keep changes minimal and focused on the requested behavior.
- Git metadata is read-only in the default sandbox; run Git write commands with escalation from the first attempt. The commit hook reads `/dev/tty`, so run commits in a TTY.
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

Run `make ci` to run all four checks. Use `make run-dev` to select a mission or open the test shop.

## Style Conventions

- Use English for code identifiers, comments, and developer documentation.
- Use Fluent message IDs for player-facing text. Write the source strings in `assets/locales/en.ftl` and add translations in the corresponding locale catalogs.
- Follow idiomatic Rust and current module organization.
- Keep functions small and readable; extract helpers when logic gets dense.
- Avoid `unwrap`/`expect` in runtime gameplay code unless there is a clear invariant.
- Prefer explicit names for gameplay constants; avoid unexplained magic numbers.
- Add short comments only for non-obvious logic (physics tuning, spawn math, timing edge cases).

## Gameplay Change Safety

When changing gameplay logic, verify:

1. Solo mission progression still works from briefing through launch and combat to the market or game over.
2. Pause/resume behavior remains intact.
3. Collision paths still compile and run for ship/enemy/loot/projectiles.
4. Score and high score persistence still behave correctly.
5. Localization-dependent UI text still renders.
6. For shared gameplay or LAN changes, verify the affected path with a host and guest. The host owns mission state and collision outcomes.

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
