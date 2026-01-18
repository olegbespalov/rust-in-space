use macroquad::prelude::*;
use macroquad::rand::gen_range;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Mission {
    pub level_id: u32,
    pub title: String,
    pub description: String,

    // mission objectives
    pub target_kills: u32,      // how many enemies to destroy
    pub target_scrap: u32,      // how many scrap (rust piles) to collect
    pub target_rare_metal: u32, // how many rare metal (gold) to collect

    // level difficulty settings
    pub enemy_spawn_interval: f32,
    pub asteroid_count: usize,
}

pub enum GameState {
    Menu,
    Briefing, // briefing screen before the mission
    Playing,
    MissionSuccess, // level completed
    GameOver(u32),
}

#[derive(Clone, Copy, PartialEq)]
pub enum BulletStyle {
    Player,
    Enemy,
}

pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life_time: f32,
    pub style: BulletStyle,
    pub damage: f32, // Damage dealt by this bullet
    pub radius: f32, // Bullet radius (for collision and drawing)
}

pub struct Asteroid {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub is_rare: bool,
}

pub struct EnemyShip {
    pub pos: Vec2,
    pub vel: Vec2,
    pub shoot_timer: f32,
    pub rotation: f32,
    pub health: f32,     // Current health points
    pub max_health: f32, // Maximum health points
}

pub struct Ship {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rotation: f32,
    pub health: f32,     // Current health points
    pub max_health: f32, // Maximum health points
    pub shoot_timer: f32,
    pub rapid_fire_timer: f32,
    pub engine: Engine,

    pub scrap: u32,      // Ordinary money
    pub rare_metal: u32, // Premium money

    // Shield state
    pub shield_hp: f32,     // Current shield HP (0 if inactive)
    pub shield_max_hp: f32, // Maximum shield HP capacity
    pub shield_timer: f32,  // Time remaining for shield (0 if inactive)

    // Weapon boost timers
    pub big_bullet_timer: f32, // Time remaining for big bullet boost (0 if inactive)
}

pub struct Engine {
    pub current_thrust: f32, // Current thrust (0.0 - 1.0)
    pub ramp_up: f32,        // Speed of thrust increase
    pub decay: f32,          // Speed of decay
    pub offset: f32,         // Offset of the nozzles relative to the center of the ship
}

// 1. Types of loot
#[derive(Clone, PartialEq)]
pub enum LootType {
    // Currencies
    Scrap(u32),     // Scrap (ordinary resource)
    RareMetal(u32), // Rare metal/Gold (for big upgrades)

    // Buffs (applied immediately)
    HealthPack(i32), // Health recovery
    RapidFireBoost,  // Fast ammo boost (reduced cooldown)
    BigBulletBoost,  // Bigger bullets with more damage
    Shield(u32),     // Shield with HP capacity
}

// 2. The entity of the dropped item
pub struct LootItem {
    pub pos: Vec2,
    pub vel: Vec2,       // Initial explosion velocity (decays)
    pub drift_vel: Vec2, // Slow constant drift in space
    pub item_type: LootType,
    pub radius: f32,
    pub magnet_active: bool, // Is the magnet active?
    pub rotation: f32,       // Rotation angle in radians
    pub rotation_speed: f32, // Rotation speed in radians per second (can be negative)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveData {
    pub high_score: u32,
}

impl Asteroid {
    pub fn new_large() -> Self {
        // 10% chance of being rare
        let is_rare = gen_range(0, 100) < 10;
        Self {
            pos: vec2(
                gen_range(0.0, screen_width()),
                gen_range(0.0, screen_height()),
            ),
            vel: vec2(gen_range(-80.0, 80.0), gen_range(-80.0, 80.0)),
            radius: 40.0,
            is_rare,
        }
    }

    pub fn new_fragment(pos: Vec2, radius: f32) -> Self {
        // Fragments are never rare
        Self {
            pos,
            vel: vec2(gen_range(-120.0, 120.0), gen_range(-120.0, 120.0)),
            radius: radius / 2.0,
            is_rare: false,
        }
    }
}

impl EnemyShip {
    pub fn new() -> Self {
        let side = gen_range(0, 2);
        let x = if side == 0 {
            -30.0
        } else {
            screen_width() + 30.0
        };
        let y = gen_range(50.0, screen_height() - 50.0);
        let speed_x = if side == 0 { 120.0 } else { -120.0 }; // Use constant or number
        let max_health = 50.0;
        Self {
            pos: vec2(x, y),
            vel: vec2(speed_x, gen_range(-20.0, 20.0)),
            shoot_timer: 1.5,
            rotation: 0.0,
            health: max_health,
            max_health,
        }
    }

    // Returns true if the enemy is destroyed
    pub fn take_damage(&mut self, damage: f32) -> bool {
        self.health -= damage;
        self.health <= 0.0
    }
}

impl Ship {
    // Returns true if the game is over
    // Damage is first applied to shield if active, then to ship health
    pub fn take_damage(&mut self, damage: f32, score: u32) -> bool {
        // If shield is active, absorb damage with shield first
        if self.shield_hp > 0.0 {
            if self.shield_hp >= damage {
                // Shield absorbs all damage
                self.shield_hp -= damage;
                return false; // Ship is safe
            } else {
                // Shield is depleted, remaining damage goes to ship
                let remaining_damage = damage - self.shield_hp;
                self.shield_hp = 0.0;
                self.shield_timer = 0.0; // Shield deactivates when HP reaches 0
                self.health -= remaining_damage;
            }
        } else {
            // No shield, damage goes directly to ship
            self.health -= damage;
        }

        if self.health <= 0.0 {
            // Save score immediately using our system
            crate::systems::save_score(score);
            true // Game Over
        } else {
            false // Still alive
        }
    }

    // Restore health (used by health packs)
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    // Activate shield with given HP capacity and duration
    pub fn activate_shield(&mut self, hp: f32, duration: f32) {
        self.shield_max_hp = hp;
        self.shield_hp = hp;
        self.shield_timer = duration;
    }

    // Check if shield is active
    pub fn has_shield(&self) -> bool {
        self.shield_hp > 0.0 && self.shield_timer > 0.0
    }
}

impl Engine {
    pub fn basic() -> Self {
        Self {
            current_thrust: 0.0,
            ramp_up: 5.0,
            decay: 3.0,
            offset: 42.0,
        }
    }

    // All the logic of changing the thrust is now encapsulated here
    pub fn update(&mut self, dt: f32, is_active: bool) {
        if is_active {
            self.current_thrust = (self.current_thrust + self.ramp_up * dt).min(1.0);
        } else {
            self.current_thrust = (self.current_thrust - self.decay * dt).max(0.0);
        }
    }
}

pub struct Explosion {
    pub pos: Vec2,
    pub timer: f32,        // Timer for frame change
    pub frame: usize,      // Текущий кадр (0, 1, 2...)
    pub max_frames: usize, // How many frames in the texture (e.g. 8)
    pub frame_time: f32,   // Animation speed (e.g. 0.1 sec per frame)
    pub scale: f32,        // Explosion size (for boss large, for enemy small)
}

impl SaveData {
    pub fn new() -> Self {
        Self { high_score: 0 }
    }
}

impl Explosion {
    pub fn new(pos: Vec2, scale: f32) -> Self {
        Self {
            pos,
            timer: 0.0,
            frame: 0,
            max_frames: 8,    // Depends on sprite!
            frame_time: 0.05, // Fast explosion
            scale,
        }
    }
}
