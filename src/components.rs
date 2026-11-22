use crate::game::constants::{
    BIG_BULLET_DURATION, BOSS_MAX_HEALTH, BOSS_RING_COUNT, BOSS_SIZE, ENEMY_SMALL_RADIUS,
    GUARDIAN_RADIUS, HEAVY_RADIUS, INTERCEPTOR_RADIUS, INVISIBILITY_DURATION,
    MAGNETIC_AMMO_DURATION, RAPID_FIRE_DURATION, SHIELD_DURATION,
};
use crate::localization::Language;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use serde::{Deserialize, Serialize};

fn default_master_volume() -> f32 {
    1.0
}

fn default_music_volume() -> f32 {
    0.7
}

fn default_sfx_volume() -> f32 {
    0.85
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Mission {
    pub level_id: u32,
    pub title: String,
    pub objectives: Vec<MissionObjective>,
    pub enemy_spawn_interval: f32,
    pub asteroid_count: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissionObjective {
    DefeatEnemies(u32),
    DestroyAsteroids(u32),
    DefeatInterceptors(u32),
    DefeatHeavyShips(u32),
    CollectScrap(u32),
    Survive(u32),
    DefeatBoss,
}

impl MissionObjective {
    pub fn target(self) -> u32 {
        match self {
            Self::DefeatEnemies(target)
            | Self::DestroyAsteroids(target)
            | Self::DefeatInterceptors(target)
            | Self::DefeatHeavyShips(target)
            | Self::CollectScrap(target)
            | Self::Survive(target) => target,
            Self::DefeatBoss => 1,
        }
    }
}

pub enum GameState {
    PlayerName(String),
    Menu,
    Lan(Box<crate::lan::LanMode>),
    ShipSelection(ShipKind),
    Briefing, // briefing screen before the mission
    Playing,
    Paused,         // game is paused
    MissionSuccess, // level completed
    UpgradeBay,     // intermission upgrades screen
    GameOver(u32),
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum MenuItem {
    #[default]
    Start,
    Lan,
    Difficulty,
    Language,
    MasterVolume,
    MusicVolume,
    SfxVolume,
    AudioMute,
}

impl MenuItem {
    pub fn next(self) -> Self {
        match self {
            MenuItem::Start => MenuItem::Lan,
            MenuItem::Lan => MenuItem::Difficulty,
            MenuItem::Difficulty => MenuItem::Language,
            MenuItem::Language => MenuItem::MasterVolume,
            MenuItem::MasterVolume => MenuItem::MusicVolume,
            MenuItem::MusicVolume => MenuItem::SfxVolume,
            MenuItem::SfxVolume => MenuItem::AudioMute,
            MenuItem::AudioMute => MenuItem::Start,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            MenuItem::Start => MenuItem::AudioMute,
            MenuItem::Difficulty => MenuItem::Lan,
            MenuItem::Lan => MenuItem::Start,
            MenuItem::Language => MenuItem::Difficulty,
            MenuItem::MasterVolume => MenuItem::Language,
            MenuItem::MusicVolume => MenuItem::MasterVolume,
            MenuItem::SfxVolume => MenuItem::MusicVolume,
            MenuItem::AudioMute => MenuItem::SfxVolume,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BulletStyle {
    Player,
    Enemy,
    Boss,
    BossMissile,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EnemyType {
    Regular,
    Kamikaze,
    Interceptor,
    Heavy,
    Guardian,
}

impl EnemyType {
    pub fn projectile_hitbox_radius(self) -> f32 {
        match self {
            Self::Regular | Self::Kamikaze => ENEMY_SMALL_RADIUS,
            Self::Interceptor => INTERCEPTOR_RADIUS,
            Self::Heavy => HEAVY_RADIUS,
            Self::Guardian => GUARDIAN_RADIUS,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Bullet {
    pub pos: Vec2,
    pub vel: Vec2,
    #[serde(skip)]
    pub owner: Option<usize>,
    #[serde(skip)]
    pub life_time: f32,
    pub style: BulletStyle,
    pub homing: bool,
    #[serde(skip)]
    pub damage: f32, // Damage dealt by this bullet
    pub radius: f32, // Bullet radius (for collision and drawing)
}

#[derive(Serialize, Deserialize)]
pub struct Asteroid {
    pub pos: Vec2,
    #[serde(skip)]
    pub vel: Vec2,
    pub radius: f32,
    pub is_rare: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EnemyShip {
    pub pos: Vec2,
    #[serde(skip)]
    pub vel: Vec2,
    #[serde(skip)]
    pub shoot_timer: f32,
    pub rotation: f32,
    pub health: f32,     // Current health points
    pub max_health: f32, // Maximum health points
    pub enemy_type: EnemyType,
    pub power_ups: EnemyPowerUps,
}

#[derive(Serialize, Deserialize)]
pub struct Boss {
    pub pos: Vec2,
    #[serde(skip)]
    pub vel: Vec2,
    pub rotation: f32,
    pub health: f32,
    pub max_health: f32,
    #[serde(skip)]
    pub shoot_timer: f32,
    #[serde(skip)]
    pub rings_left: u32,
    #[serde(skip)]
    pub next_attack: BossAttackKind,
    pub pending_attack: Option<BossAttack>,
    #[serde(skip)]
    pub shield_timer: f32,
    pub power_ups: EnemyPowerUps,
    pub phase: BossPhase,
    #[serde(skip)]
    pub summon_timer: f32,
    pub pending_summon: Option<BossSummon>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BossPhase {
    Opening,
    MissileSalvos,
    Escalated,
}

impl BossPhase {
    pub const COUNT: u8 = 3;

    pub fn number(self) -> u8 {
        match self {
            Self::Opening => 1,
            Self::MissileSalvos => 2,
            Self::Escalated => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossAttackKind {
    #[default]
    Cannons,
    Missiles,
    Rings,
}

impl BossAttackKind {
    pub fn warning_duration(self) -> f32 {
        match self {
            Self::Cannons => 0.85,
            Self::Missiles => 1.05,
            Self::Rings => 1.15,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BossAttack {
    pub kind: BossAttackKind,
    pub remaining: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BossSummon {
    pub from_left: bool,
    pub y: f32,
    pub remaining: f32,
}

#[derive(Default, Serialize, Deserialize)]
pub struct EnemyPowerUps {
    #[serde(skip)]
    pub rapid_fire_timer: f32,
    #[serde(skip)]
    pub big_bullet_timer: f32,
    #[serde(skip)]
    pub magnetic_ammo_timer: f32,
    pub shield: Option<EnemyShield>,
    invisibility_timer: f32,
}

#[derive(Serialize, Deserialize)]
pub struct EnemyShield {
    pub kind: ShieldKind,
    pub hp: f32,
    pub max_hp: f32,
    #[serde(skip)]
    time_left: f32,
}

impl EnemyPowerUps {
    pub fn is_invisible(&self) -> bool {
        self.invisibility_timer > 0.0
    }

    pub fn tick(&mut self, dt: f32) {
        self.rapid_fire_timer -= dt;
        self.big_bullet_timer -= dt;
        self.magnetic_ammo_timer -= dt;
        self.invisibility_timer -= dt;
        if let Some(shield) = &mut self.shield {
            shield.time_left -= dt;
            if shield.time_left <= 0.0 {
                self.shield = None;
            }
        }
    }

    pub fn pick_up(&mut self, item: &LootType, health: &mut f32, max_health: f32) {
        match *item {
            LootType::RapidFireBoost => self.rapid_fire_timer = RAPID_FIRE_DURATION,
            LootType::BigBulletBoost => self.big_bullet_timer = BIG_BULLET_DURATION,
            LootType::MagneticAmmo => self.magnetic_ammo_timer = MAGNETIC_AMMO_DURATION,
            LootType::Invisibility => self.invisibility_timer = INVISIBILITY_DURATION,
            LootType::Shield(kind, hp) => {
                self.activate_shield(kind, hp as f32, kind.duration());
            }
            LootType::HealthPack(hp) => *health = (*health + hp as f32).min(max_health),
            LootType::MysteryBox => {
                self.apply_mystery_effect(
                    MysteryEffect::from_roll(gen_range(0, 4)),
                    health,
                    max_health,
                );
            }
            LootType::Scrap(_) | LootType::RareMetal(_) => {}
        }
    }

    fn apply_mystery_effect(&mut self, effect: MysteryEffect, health: &mut f32, max_health: f32) {
        match effect {
            MysteryEffect::Repair(hp) => *health = (*health + hp).min(max_health),
            MysteryEffect::Gold(_) => {}
            MysteryEffect::Shield(hp) => {
                if let Some(shield) = &mut self.shield {
                    shield.time_left = shield.time_left.max(SHIELD_DURATION);
                    if shield.hp < hp {
                        shield.hp = hp;
                        shield.max_hp = hp;
                    }
                } else {
                    self.activate_shield(ShieldKind::Standard, hp, SHIELD_DURATION);
                }
            }
            MysteryEffect::Trap(damage) => *health -= self.absorb_damage(damage),
        }
    }

    pub fn activate_shield(&mut self, kind: ShieldKind, hp: f32, duration: f32) {
        self.shield = Some(EnemyShield {
            kind,
            hp,
            max_hp: hp,
            time_left: duration,
        });
    }

    fn absorb_damage(&mut self, damage: f32) -> f32 {
        let Some(shield) = &mut self.shield else {
            return damage;
        };
        let absorbed = shield.hp.min(damage);
        shield.hp -= absorbed;
        if shield.hp <= 0.0 {
            self.shield = None;
        }
        damage - absorbed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipKind {
    Alpha,
    Delta,
}

impl ShipKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Alpha => "ship_alpha",
            Self::Delta => "ship_delta",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Ship {
    pub kind: ShipKind,
    pub pos: Vec2,
    #[serde(skip)]
    pub vel: Vec2,
    pub rotation: f32,
    pub health: f32,     // Current health points
    pub max_health: f32, // Maximum health points
    #[serde(skip)]
    pub shoot_timer: f32,
    #[serde(skip)]
    pub rapid_fire_timer: f32,
    pub engine: Engine,

    pub scrap: u32,      // Ordinary money
    pub rare_metal: u32, // Premium money

    // Shield state
    pub shield_kind: ShieldKind,
    pub shield_hp: f32,     // Current shield HP (0 if inactive)
    pub shield_max_hp: f32, // Maximum shield HP capacity
    pub shield_timer: f32,  // Time remaining for shield (0 if inactive)

    pub invisibility_timer: f32,

    // Weapon boost timers
    #[serde(skip)]
    pub magnetic_ammo_timer: f32,
    #[serde(skip)]
    pub big_bullet_timer: f32, // Time remaining for big bullet boost (0 if inactive)
}

#[derive(Serialize, Deserialize)]
pub struct Engine {
    pub current_thrust: f32, // Current thrust (0.0 - 1.0)
    #[serde(skip)]
    pub ramp_up: f32, // Speed of thrust increase
    #[serde(skip)]
    pub decay: f32, // Speed of decay
    pub offset: f32,         // Offset of the nozzles relative to the center of the ship
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpgradeId {
    ReinforcedHull,
    WeaponTuning,
    EngineOverdrive,
    MagnetArray,
    ShieldCapacitor,
}

#[derive(Clone, Copy)]
pub struct UpgradeCost {
    pub scrap: u32,
    pub rare_metal: u32,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct UpgradeLevels {
    pub reinforced_hull: u8,
    pub weapon_tuning: u8,
    pub engine_overdrive: u8,
    pub magnet_array: u8,
    pub shield_capacitor: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShieldKind {
    Standard,
    Golden,
}

impl ShieldKind {
    pub fn duration(self) -> f32 {
        use crate::game::constants::{GOLDEN_SHIELD_DURATION, SHIELD_DURATION};
        match self {
            Self::Standard => SHIELD_DURATION,
            Self::Golden => GOLDEN_SHIELD_DURATION,
        }
    }
}

// 1. Types of loot
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum LootType {
    // Currencies
    Scrap(u32),     // Scrap (ordinary resource)
    RareMetal(u32), // Rare metal/Gold (for big upgrades)

    // Buffs (applied immediately)
    HealthPack(i32), // Health recovery
    RapidFireBoost,  // Fast ammo boost (reduced cooldown)
    MagneticAmmo,    // Player shots steer toward enemies
    BigBulletBoost,  // Bigger bullets with more damage
    Invisibility,
    Shield(ShieldKind, u32),

    MysteryBox,
}

impl LootType {
    pub fn lifetime(&self) -> f32 {
        match self {
            Self::Scrap(_) => 30.0,
            Self::HealthPack(_) => 24.0,
            Self::RareMetal(_) => 18.0,
            Self::RapidFireBoost | Self::BigBulletBoost | Self::Shield(ShieldKind::Standard, _) => {
                15.0
            }
            Self::MagneticAmmo | Self::Invisibility => 12.0,
            Self::MysteryBox | Self::Shield(ShieldKind::Golden, _) => 10.0,
        }
    }
}

pub enum MysteryEffect {
    Repair(f32),
    Gold(u32),
    Shield(f32),
    Trap(f32),
}

impl MysteryEffect {
    pub fn from_roll(roll: i32) -> Self {
        match roll {
            0 => Self::Repair(25.0),
            1 => Self::Gold(5),
            2 => Self::Shield(45.0),
            _ => Self::Trap(20.0),
        }
    }
}

#[cfg(test)]
mod loot_tests {
    use super::*;

    #[test]
    fn enemy_pickups_apply_healing_invisibility_and_mystery_effects() {
        let mut power_ups = EnemyPowerUps::default();
        let mut health = 15.0;
        let max_health = 50.0;
        power_ups.pick_up(&LootType::HealthPack(25), &mut health, max_health);
        assert_eq!(health, 40.0);
        power_ups.apply_mystery_effect(MysteryEffect::from_roll(0), &mut health, max_health);
        assert_eq!(health, max_health);

        power_ups.pick_up(&LootType::Invisibility, &mut health, max_health);
        assert!(power_ups.is_invisible());
        power_ups.tick(INVISIBILITY_DURATION);
        assert!(!power_ups.is_invisible());

        power_ups.apply_mystery_effect(MysteryEffect::from_roll(2), &mut health, max_health);
        power_ups.apply_mystery_effect(MysteryEffect::from_roll(3), &mut health, max_health);
        assert_eq!(health, max_health);
        assert_eq!(
            power_ups.shield.as_ref().map(|shield| shield.hp),
            Some(25.0)
        );

        power_ups.tick(SHIELD_DURATION);
        power_ups.apply_mystery_effect(MysteryEffect::from_roll(3), &mut health, max_health);
        assert_eq!(health, 30.0);
    }
}

// 2. The entity of the dropped item
#[derive(Serialize, Deserialize)]
pub struct LootItem {
    pub pos: Vec2,
    #[serde(skip)]
    pub vel: Vec2, // Initial explosion velocity (decays)
    #[serde(skip)]
    pub drift_vel: Vec2, // Slow constant drift in space
    pub item_type: LootType,
    pub time_left: f32,
    pub radius: f32,
    #[serde(skip)]
    pub magnet_active: bool, // Is the magnet active?
    pub rotation: f32, // Rotation angle in radians
    #[serde(skip)]
    pub rotation_speed: f32, // Rotation speed in radians per second (can be negative)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AudioSettings {
    #[serde(default = "default_master_volume")]
    pub master_volume: f32,
    #[serde(default = "default_music_volume")]
    pub music_volume: f32,
    #[serde(default = "default_sfx_volume")]
    pub sfx_volume: f32,
    #[serde(default)]
    pub audio_muted: bool,
}

pub const DEFAULT_PLAYER_NAME: &str = "player1";

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct SaveData {
    pub high_score: u32,
    pub audio: AudioSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_name: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Difficulty {
    Nebula,    // Easy
    Supernova, // Medium
    BlackHole, // Hard
}

impl Asteroid {
    pub fn new_large(bounds: Vec2) -> Self {
        // 10% chance of being rare
        let is_rare = gen_range(0, 100) < 10;
        Self {
            pos: vec2(gen_range(0.0, bounds.x), gen_range(0.0, bounds.y)),
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
    pub fn new_at_side(from_left: bool, y: f32, enemy_type: EnemyType, width: f32) -> Self {
        let x = if from_left { -30.0 } else { width + 30.0 };
        let (max_health, speed, shoot_timer) = match enemy_type {
            EnemyType::Regular => (24.0, 120.0, 1.5),
            EnemyType::Kamikaze => (18.0, 120.0, 1.5),
            EnemyType::Interceptor => (15.0, 210.0, 1.2),
            EnemyType::Heavy => (60.0, 75.0, 2.2),
            EnemyType::Guardian => (105.0, 65.0, 2.6),
        };
        let speed_x = if from_left { speed } else { -speed };
        Self {
            pos: vec2(x, y),
            vel: vec2(speed_x, gen_range(-20.0, 20.0)),
            shoot_timer,
            rotation: 0.0,
            health: max_health,
            max_health,
            enemy_type,
            power_ups: EnemyPowerUps::default(),
        }
    }

    // Returns true if the enemy is destroyed
    pub fn take_damage(&mut self, damage: f32) -> bool {
        self.health -= self.power_ups.absorb_damage(damage);
        self.health <= 0.0
    }
}

impl Boss {
    pub fn new(pos: Vec2, summon_timer: f32) -> Self {
        Self {
            pos,
            vel: vec2(68.0, 24.0),
            rotation: std::f32::consts::FRAC_PI_2,
            health: BOSS_MAX_HEALTH,
            max_health: BOSS_MAX_HEALTH,
            shoot_timer: 1.0,
            rings_left: 0,
            next_attack: BossAttackKind::Cannons,
            pending_attack: None,
            shield_timer: 0.0,
            power_ups: EnemyPowerUps::default(),
            phase: BossPhase::Opening,
            summon_timer,
            pending_summon: None,
        }
    }

    pub fn attack_paths(
        &self,
        kind: BossAttackKind,
        ring: u32,
    ) -> impl Iterator<Item = (Vec2, Vec2)> {
        fn direction(angle: f32) -> Vec2 {
            vec2(angle.cos(), angle.sin())
        }

        let heading = self.rotation;
        let center = self.pos;
        let forward = direction(heading);
        let right = forward.perp();
        let count = match kind {
            BossAttackKind::Cannons => 6,
            BossAttackKind::Missiles => 2,
            BossAttackKind::Rings => 12,
        };
        // Cannon and launcher offsets are measured in fractions of the sprite size.
        (0..count).map(move |shot| match kind {
            BossAttackKind::Cannons => {
                let side = if shot < 3 { -1.0 } else { 1.0 };
                let muzzle = center + (forward * 0.42 + right * side * 0.15) * BOSS_SIZE;
                let offset = (shot % 3) as f32 - 1.0;
                (muzzle, direction(heading + 0.17 * offset + 0.06 * side))
            }
            BossAttackKind::Missiles => {
                let side = if shot == 0 { -1.0 } else { 1.0 };
                let launcher = center + (forward * 0.06 + right * side * 0.32) * BOSS_SIZE;
                (launcher, direction(heading + 0.48 * side))
            }
            BossAttackKind::Rings => {
                let step = std::f32::consts::TAU / count as f32;
                let ray = direction(
                    heading + (shot as f32 + ring as f32 / BOSS_RING_COUNT as f32) * step,
                );
                (center + ray * BOSS_SIZE * 0.48, ray)
            }
        })
    }

    pub fn take_damage(&mut self, damage: f32) -> bool {
        self.health -= self.power_ups.absorb_damage(damage);
        self.health <= 0.0
    }
}

impl Ship {
    pub fn is_invisible(&self) -> bool {
        self.invisibility_timer > 0.0
    }

    // Returns true if the ship was destroyed
    // Damage is first applied to shield if active, then to ship health
    pub fn take_damage(&mut self, damage: f32) -> bool {
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

        self.health <= 0.0
    }

    // Restore health (used by health packs)
    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    // Activate shield with given HP capacity and duration
    pub fn activate_shield(&mut self, kind: ShieldKind, hp: f32, duration: f32) {
        self.shield_kind = kind;
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

impl UpgradeLevels {
    pub fn get(&self, id: UpgradeId) -> u8 {
        match id {
            UpgradeId::ReinforcedHull => self.reinforced_hull,
            UpgradeId::WeaponTuning => self.weapon_tuning,
            UpgradeId::EngineOverdrive => self.engine_overdrive,
            UpgradeId::MagnetArray => self.magnet_array,
            UpgradeId::ShieldCapacitor => self.shield_capacitor,
        }
    }

    pub fn set(&mut self, id: UpgradeId, value: u8) {
        match id {
            UpgradeId::ReinforcedHull => self.reinforced_hull = value,
            UpgradeId::WeaponTuning => self.weapon_tuning = value,
            UpgradeId::EngineOverdrive => self.engine_overdrive = value,
            UpgradeId::MagnetArray => self.magnet_array = value,
            UpgradeId::ShieldCapacitor => self.shield_capacitor = value,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Explosion {
    pub pos: Vec2,
    #[serde(skip)]
    pub timer: f32, // Timer for frame change
    pub frame: usize,      // Current frame (0, 1, 2...)
    pub max_frames: usize, // How many frames in the texture (e.g. 8)
    #[serde(skip)]
    pub frame_time: f32, // Animation speed (e.g. 0.1 sec per frame)
    pub scale: f32,        // Explosion size (for boss large, for enemy small)
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: default_master_volume(),
            music_volume: default_music_volume(),
            sfx_volume: default_sfx_volume(),
            audio_muted: false,
        }
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

impl Difficulty {
    // Damage multiplier (Easy: x0.8, Normal: x1.0, Hard: x1.5)
    pub fn damage_mult(&self) -> f32 {
        match self {
            Difficulty::Nebula => 0.8,
            Difficulty::Supernova => 1.0,
            Difficulty::BlackHole => 1.5,
        }
    }

    // Spawn rate multiplier (Easy: less frequent, Hard: more frequent)
    // We will divide the spawn interval by this number.
    pub fn spawn_rate_mult(&self) -> f32 {
        match self {
            Difficulty::Nebula => 0.8,
            Difficulty::Supernova => 1.0,
            Difficulty::BlackHole => 1.3,
        }
    }

    // Loot chance (Bonus to roll)
    // Negative means less chances
    pub fn loot_luck_modifier(&self) -> i32 {
        match self {
            Difficulty::Nebula => 10, // +10% to luck
            Difficulty::Supernova => 0,
            Difficulty::BlackHole => -15, // -15% to luck
        }
    }
}
