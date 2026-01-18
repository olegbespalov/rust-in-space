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
- **Loot System**: Collect scrap, rare metals, health packs, weapon boosts, and shields
  - **Magnet Effect**: Loot items are automatically attracted to your ship when nearby
  - **Animated Loot**: Items rotate and drift realistically in space
- **Shield System**: Activate shields that absorb damage before it reaches your health
- **Weapon Boosts**: Rapid fire mode and big bullet mode for enhanced firepower
- **Resource Management**: Track rust piles (scrap) and gold (rare metals) separately
- **Health Point System**: Start with 100 HP - bigger asteroids deal more damage!
- **Shield System**: Collect shield items to activate temporary shields that absorb damage
- **Variable Damage**: Damage scales with asteroid size and bullet type
- **Enemy Health System**: Enemies have 30 HP and take multiple hits to destroy
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
- **Enemy Ships**: 300 points each (10 points per HP, enemies have 30 HP)

### Loot System

Loot items drop from destroyed asteroids and enemies:

**From Regular Asteroids:**
- **Rust Piles (Scrap)** (55% chance): 1-3 pieces
- **Gold (Rare Metal)** (10% chance): 1 piece
- **Nothing** (35% chance)

**From Rare Asteroids** (10% chance to spawn, always drop loot):
- **Gold (Rare Metal)** (30% chance): 2-5 pieces
- **Rust Piles (Scrap)** (15% chance): 5-9 pieces
- **Health Pack** (25% chance): Restores 25 HP
- **Rapid Fire Boost** (18% chance): Rapid fire for 10 seconds (3x faster shooting)
- **Big Bullet Boost** (12% chance): Bigger, more powerful bullets for 15 seconds (30 damage vs 15)

**From Enemy Ships:**
- **Rust Piles (Scrap)** (30% chance): 5-9 pieces
- **Health Pack** (25% chance): Restores 25 HP
- **Rapid Fire Boost** (18% chance): Rapid fire for 10 seconds (3x faster shooting)
- **Big Bullet Boost** (12% chance): Bigger, more powerful bullets for 15 seconds (30 damage vs 15)
- **Shield** (8% chance): Activates shield with 30-100 HP that lasts 30 seconds
- **Nothing** (7% chance)

**Note**: Health packs, weapon boosts, and shields do NOT count toward resource collection objectives

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
- **Shield System**: 
  - Shields absorb damage before it reaches your health
  - Shield HP is displayed when active: "SHIELD: current/max"
  - Shields have a duration (30 seconds) and deactivate when HP reaches 0 or timer expires
  - Damage is first applied to shield, then to health if shield is depleted
- **Damage System**:
  - **Asteroid Collisions**: Damage scales with asteroid size (bigger asteroids = more damage)
    - Base damage: 5 HP per 10 units of radius
    - Large asteroids (radius 40): ~20 HP damage
    - Medium fragments (radius 20): ~10 HP damage
    - Small fragments (radius 10): ~5 HP damage
  - **Enemy Bullets**: Deal 15 HP damage
  - **Player Bullets**: Deal 15 HP damage to enemies (30 HP with big bullet boost)
  - **Enemy Health**: Enemies have 30 HP and take multiple hits to destroy
- Complete mission objectives to progress (kills, rust piles, and gold)
- Destroy asteroids to break them into smaller pieces
- Rare asteroids (10% spawn chance) have distinct appearance and always drop loot
- Enemy ships spawn based on mission configuration and track your position
- Enemies have 30 HP and shoot at you - destroy them to complete kill objectives
- Collect rust piles and gold separately - missions require specific amounts of each
- Health packs restore 25 HP (capped at maximum of 100 HP)
- **Rapid Fire Boost**: Reduces shooting cooldown by 3x for 10 seconds
- **Big Bullet Boost**: Shoots larger, more powerful bullets (20 damage) for 15 seconds
- **Shield**: Activates a temporary shield that absorbs damage before it reaches your health
- When HP reaches 0, your score is saved if it's a new high score

## Project Structure

```
space_game/
├── src/
│   ├── main.rs      # Main game loop and state management
│   ├── game.rs      # Game logic, updates, and rendering
│   ├── components.rs # Game entities and data structures (Ship, Asteroid, Loot, Mission, etc.)
│   ├── systems.rs   # Game systems (wrapping, save/load, mission generation, loot generation)
│   ├── draw.rs      # Rendering functions
│   └── resources.rs # Resource management (texture loading)
├── assets/          # Game assets (sprites, textures)
│   ├── loot/        # Loot item textures
│   │   ├── resources/ # Resource textures (scrap, gold)
│   │   └── ...       # Power-up textures (health, boosts, shield)
│   └── ...          # Ship, enemy, asteroid, and bullet textures
├── scripts/         # Development scripts
│   └── pre-commit   # Pre-commit hook
├── Cargo.toml       # Project dependencies
├── Makefile         # Build and development commands
├── rustfmt.toml     # Rust formatting configuration
├── clippy.toml      # Clippy linter configuration
└── highscore.json   # Saved high score (auto-generated)
```

## Dependencies

- **macroquad** (0.4): Cross-platform game framework for Rust
- **serde** (1.0): Serialization framework
- **serde_json** (1.0): JSON support for serde

## License

MIT