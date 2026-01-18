# Rust in Space

<div align="center">
  <img src="assets/logo.png" alt="Logo" width="400">
  
  ![CI](https://github.com/olegbespalov/rust-in-space/workflows/CI/badge.svg)
</div>

> *"You're a mercenary pilot, drifting through the outer rim of civilized space. Out here, the only law is what you can enforce with your ship's cannons. The corporations pay well for rust piles and rare metals mined from the asteroid fields, but they don't tell you about the enemy patrols or the void pirates that call these deep space sectors home.*
> 
> *Every mission takes you deeper into the unknown. Every asteroid you crack could be your last. But the credits are good, and in this part of the galaxy, that's all that matters. Strap in, pilot. The void awaits."*

## Features

- **Mission-Based Gameplay**: Complete objectives across multiple levels with increasing difficulty
- **Classic Space Shooter Controls**: Rotate and thrust your ship with smooth engine mechanics
- **Asteroid Destruction**: Break large asteroids into smaller fragments, with rare asteroids dropping valuable loot
- **Enemy Ships**: Battle enemy ships that track and shoot at you
- **Loot System**: Collect scrap, rare metals, health packs, and weapon boosts
  - **Magnet Effect**: Loot items are automatically attracted to your ship when nearby
  - **Animated Loot**: Items rotate and drift realistically in space
- **Resource Management**: Track rust piles (scrap) and gold (rare metals) separately
- **Health Point System**: Start with 100 HP - bigger asteroids deal more damage!
- **Variable Damage**: Damage scales with asteroid size and bullet type
- **High Score System**: Your high score is automatically saved and persists between sessions

## Controls

- **Left Arrow**: Rotate ship counter-clockwise
- **Right Arrow**: Rotate ship clockwise
- **Up Arrow**: Thrust forward (with smooth engine ramp-up)
- **Space**: Shoot bullets / Launch mission (from briefing screen)
- **Enter**: Start game (from menu) / Next mission (from success screen) / Return to menu (from game over screen)

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- Cargo (comes with Rust)

### Building and Running

1. Clone or download this repository
2. Navigate to the project directory:
   ```bash
   cd rust-in-space
   ```

3. Run the game:
   ```bash
   make run
   ```

   Or for release mode (optimized):
   ```bash
   make run-release
   ```

### Development Commands

This project includes a Makefile with useful commands:

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

## Game Mechanics

### Mission System

The game features a mission-based progression system:
- **Briefing Screen**: View mission objectives before launching
- **Mission Objectives**: Each mission requires completing specific goals:
  - Destroy a certain number of enemies
  - Collect a certain amount of rust piles (scrap)
  - Collect a certain amount of gold (rare metals)
- **Mission Success**: Complete all objectives to progress to the next level
- **Progressive Difficulty**: Missions become increasingly challenging with more enemies and asteroids

### Scoring
- **Asteroids**: 100 points each
- **Enemy Ships**: 500 points each

### Loot System

Loot items drop from destroyed asteroids and enemies:

**From Regular Asteroids:**
- **Rust Piles (Scrap)** (40% chance): 1-3 pieces
- **Gold (Rare Metal)** (5% chance): 1 piece

**From Rare Asteroids** (10% chance to spawn, always drop loot):
- **Gold (Rare Metal)** (50% chance): 2-5 pieces
- **Rust Piles (Scrap)** (30% chance): 5-9 pieces
- **Health Pack** (10% chance): Restores health points
- **Weapon Boost** (10% chance): Rapid fire for 10 seconds

**From Enemy Ships:**
- **Rust Piles (Scrap)** (30% chance): 5-9 pieces
- **Health Pack** (10% chance): Restores health points
- **Weapon Boost** (5% chance): Rapid fire for 10 seconds

**Note**: Health packs and weapon boosts do NOT count toward resource collection objectives

**Loot Mechanics:**
- Items drift and rotate in space for visual appeal
- **Magnet Effect**: When within 150 units of your ship, loot is automatically attracted to you
- Items are collected on contact with your ship
- **Resource Tracking**: 
  - Mission objectives track rust piles and gold separately
  - Your inventory shows total resources collected: "Resources: Rust X | Gold Y"
  - Mission progress shows: "Kills: X/Y  Rust: X/Y  Gold: X/Y"

### Gameplay

- **Health System**: Start with 100 HP (displayed as HP: current/max)
- **Damage System**:
  - **Asteroid Collisions**: Damage scales with asteroid size (bigger asteroids = more damage)
    - Base damage: 5 HP per 10 units of radius
    - Large asteroids (radius 40): ~20 HP damage
    - Medium fragments (radius 20): ~10 HP damage
    - Small fragments (radius 10): ~5 HP damage
  - **Enemy Bullets**: Deal 15 HP damage
  - **Player Bullets**: Deal 10 HP damage to enemies
- Complete mission objectives to progress (kills, rust piles, and gold)
- Destroy asteroids to break them into smaller pieces
- Rare asteroids (10% spawn chance) have distinct appearance and better loot
- Enemy ships spawn based on mission configuration and track your position
- Collect rust piles and gold separately - missions require specific amounts of each
- Health packs restore HP (capped at maximum)
- When HP reaches 0, your score is saved if it's a new high score

## Project Structure

```
space_game/
├── src/
│   ├── main.rs      # Main game loop and state management
│   ├── components.rs # Game entities and data structures (Ship, Asteroid, Loot, Mission, etc.)
│   ├── systems.rs   # Game systems (wrapping, save/load, mission generation, loot generation)
│   ├── draw.rs      # Rendering functions
│   └── resources.rs # Resource management (texture loading)
├── assets/          # Game assets (sprites, textures)
│   ├── loot/        # Loot item textures
│   └── ...
├── Cargo.toml       # Project dependencies
└── highscore.json   # Saved high score (auto-generated)
```

## Dependencies

- **macroquad** (0.4): Cross-platform game framework for Rust
- **serde** (1.0): Serialization framework
- **serde_json** (1.0): JSON support for serde

## License

MIT