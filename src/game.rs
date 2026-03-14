use crate::components::*;
use crate::draw::*;
use crate::resources::Resources;
use crate::systems::{generate_loot, get_mission, load_score, wrap_around, LootSource};
use macroquad::prelude::*;
use std::collections::HashSet;

// Game constants
pub const ROTATION_SPEED: f32 = 200.0;
pub const ACCELERATION: f32 = 150.0;
pub const BULLET_SPEED: f32 = 400.0;
pub const BULLET_LIFETIME: f32 = 2.0;
pub const SHOOT_COOLDOWN: f32 = 0.3;
pub const PLAYER_BULLET_DAMAGE: f32 = 15.0;
pub const PLAYER_BULLET_RADIUS: f32 = 6.0;
pub const BIG_BULLET_DAMAGE: f32 = 30.0;
pub const BIG_BULLET_RADIUS: f32 = 12.0;
pub const ENEMY_BULLET_DAMAGE: f32 = 15.0;
pub const BASE_ASTEROID_DAMAGE: f32 = 5.0;
pub const BASE_KAMIKAZE_DAMAGE: f32 = 30.0; // Base explosion damage for kamikaze
pub const SCORE_PER_ENEMY_HP: u32 = 10;
pub const ENEMY_BORDER_MARGIN: f32 = 40.0;
/// Delay in seconds after an enemy spawns before the direction hint is shown (Easy only).
pub const ENEMY_HINT_DELAY: f64 = 2.5;
pub const BASE_SHIP_HP: f32 = 150.0;
pub const BASE_LOOT_MAGNET_RADIUS: f32 = 150.0;
pub const HULL_HP_PER_LEVEL: f32 = 20.0;
pub const WEAPON_DAMAGE_BONUS_PER_LEVEL: f32 = 0.10;
pub const ENGINE_ACCEL_BONUS_PER_LEVEL: f32 = 0.08;
pub const MAGNET_RADIUS_PER_LEVEL: f32 = 25.0;
pub const SHIELD_CAPACITOR_HP: f32 = 45.0;
pub const SHIELD_CAPACITOR_DURATION: f32 = 20.0;

pub const UPGRADABLE_IDS: [UpgradeId; 5] = [
    UpgradeId::ReinforcedHull,
    UpgradeId::WeaponTuning,
    UpgradeId::EngineOverdrive,
    UpgradeId::MagnetArray,
    UpgradeId::ShieldCapacitor,
];

pub struct UpgradeDefinition {
    pub name_key: &'static str,
    pub desc_key: &'static str,
    pub max_level: u8,
}

pub enum BuyUpgradeResult {
    Bought,
    Maxed,
    NotEnoughResources,
}

pub fn upgrade_definition(id: UpgradeId) -> UpgradeDefinition {
    match id {
        UpgradeId::ReinforcedHull => UpgradeDefinition {
            name_key: "upgrade_hull_name",
            desc_key: "upgrade_hull_desc",
            max_level: 3,
        },
        UpgradeId::WeaponTuning => UpgradeDefinition {
            name_key: "upgrade_weapon_name",
            desc_key: "upgrade_weapon_desc",
            max_level: 3,
        },
        UpgradeId::EngineOverdrive => UpgradeDefinition {
            name_key: "upgrade_engine_name",
            desc_key: "upgrade_engine_desc",
            max_level: 3,
        },
        UpgradeId::MagnetArray => UpgradeDefinition {
            name_key: "upgrade_magnet_name",
            desc_key: "upgrade_magnet_desc",
            max_level: 2,
        },
        UpgradeId::ShieldCapacitor => UpgradeDefinition {
            name_key: "upgrade_shield_name",
            desc_key: "upgrade_shield_desc",
            max_level: 1,
        },
    }
}

pub fn upgrade_cost(id: UpgradeId, next_level: u8) -> UpgradeCost {
    match id {
        UpgradeId::ReinforcedHull => UpgradeCost {
            scrap: 20 + 25 * (next_level as u32 - 1),
            rare_metal: 0,
        },
        UpgradeId::WeaponTuning => UpgradeCost {
            scrap: 30 + 25 * (next_level as u32 - 1),
            rare_metal: if next_level >= 3 { 2 } else { 0 },
        },
        UpgradeId::EngineOverdrive => UpgradeCost {
            scrap: 25 + 20 * (next_level as u32 - 1),
            rare_metal: 0,
        },
        UpgradeId::MagnetArray => UpgradeCost {
            scrap: 20 + 20 * (next_level as u32 - 1),
            rare_metal: next_level as u32 - 1,
        },
        UpgradeId::ShieldCapacitor => UpgradeCost {
            scrap: 0,
            rare_metal: 3,
        },
    }
}

pub struct Game {
    pub ship: Ship,
    pub bullets: Vec<Bullet>,
    pub asteroids: Vec<Asteroid>,
    pub enemy_ships: Vec<EnemyShip>,
    pub loot_items: Vec<LootItem>,
    pub explosions: Vec<Explosion>,
    pub score: u32,
    pub current_level_idx: u32,
    pub current_mission: Mission,
    pub mission_kills: u32,
    pub mission_scrap_collected: u32,
    pub mission_rare_metal_collected: u32,
    pub enemy_spawn_timer: f32,
    pub difficulty: Difficulty,
    pub menu_selection: MenuItem,
    /// On Easy: hint shows first, then ship spawns after ENEMY_HINT_DELAY.
    /// (from_left, spawn_y, hint_shown_at_time) — ship spawns at that position when delay has passed.
    pub pending_enemy_hint: Option<(bool, f32, f64)>,
    pub boss: Option<Boss>,
    pub upgrade_levels: UpgradeLevels,
    pub upgrade_selection: usize,
}

impl Game {
    pub fn new() -> Self {
        Self {
            ship: create_ship(),
            bullets: Vec::new(),
            asteroids: Vec::new(),
            enemy_ships: Vec::new(),
            loot_items: Vec::new(),
            explosions: Vec::new(),
            score: 0,
            current_level_idx: 1,
            current_mission: get_mission(1),
            mission_kills: 0,
            mission_scrap_collected: 0,
            mission_rare_metal_collected: 0,
            enemy_spawn_timer: 0.0,
            difficulty: Difficulty::Supernova,
            menu_selection: MenuItem::Start,
            pending_enemy_hint: None,
            boss: None,
            upgrade_levels: UpgradeLevels::default(),
            upgrade_selection: 0,
        }
    }

    pub fn reset(&mut self) {
        let saved_diff = self.difficulty;
        self.bullets.clear();
        self.asteroids = (0..5).map(|_| Asteroid::new_large()).collect();
        self.loot_items.clear();
        self.enemy_ships.clear();
        self.boss = None;
        self.score = 0;
        self.current_level_idx = 1;
        self.current_mission = get_mission(self.current_level_idx);
        self.ship = create_ship();
        self.difficulty = saved_diff;
        self.pending_enemy_hint = None;
        self.upgrade_levels = UpgradeLevels::default();
        self.upgrade_selection = 0;
        self.apply_permanent_upgrades();
    }

    pub fn start_mission(&mut self) {
        self.bullets.clear();
        self.enemy_ships.clear();
        self.loot_items.clear();
        self.boss = None;

        if self.current_mission.is_boss_level {
            self.asteroids = Vec::new();
            self.boss = Some(Boss::new());
        } else {
            self.asteroids = (0..self.current_mission.asteroid_count)
                .map(|_| Asteroid::new_large())
                .collect();
        }

        self.mission_kills = 0;
        self.mission_scrap_collected = 0;
        self.mission_rare_metal_collected = 0;
        self.enemy_spawn_timer = self.current_mission.enemy_spawn_interval;
        self.pending_enemy_hint = None;

        // Reset ship position and movement, restore health to full
        self.ship.pos = vec2(screen_width() / 2.0, screen_height() / 2.0);
        self.ship.vel = vec2(0.0, 0.0);
        self.ship.rotation = 0.0;
        self.ship.engine.current_thrust = 0.0;
        self.apply_permanent_upgrades();
        // Restore health to maximum HP for the mission
        self.ship.health = self.ship.max_health;

        if self.upgrade_levels.get(UpgradeId::ShieldCapacitor) > 0 && !self.ship.has_shield() {
            self.ship
                .activate_shield(SHIELD_CAPACITOR_HP, SHIELD_CAPACITOR_DURATION);
        }
        // Note: scrap, rare_metal, and temporary boost timers are preserved between missions
    }

    pub fn next_mission(&mut self) {
        self.current_level_idx += 1;
        self.current_mission = get_mission(self.current_level_idx);
    }

    pub fn is_mission_complete(&self) -> bool {
        if self.current_mission.is_boss_level {
            self.boss.is_none()
        } else {
            self.mission_kills >= self.current_mission.target_kills
                && self.mission_scrap_collected >= self.current_mission.target_scrap
                && self.mission_rare_metal_collected >= self.current_mission.target_rare_metal
        }
    }

    pub fn cycle_difficulty(&mut self) {
        self.difficulty = match self.difficulty {
            Difficulty::Nebula => Difficulty::Supernova,
            Difficulty::Supernova => Difficulty::BlackHole,
            Difficulty::BlackHole => Difficulty::Nebula,
        };
    }

    pub fn selected_upgrade(&self) -> Option<UpgradeId> {
        UPGRADABLE_IDS.get(self.upgrade_selection).copied()
    }

    pub fn is_continue_selected(&self) -> bool {
        self.upgrade_selection >= UPGRADABLE_IDS.len()
    }

    pub fn next_upgrade_selection(&mut self) {
        let total_items = UPGRADABLE_IDS.len() + 1;
        self.upgrade_selection = (self.upgrade_selection + 1) % total_items;
    }

    pub fn prev_upgrade_selection(&mut self) {
        let total_items = UPGRADABLE_IDS.len() + 1;
        self.upgrade_selection = if self.upgrade_selection == 0 {
            total_items - 1
        } else {
            self.upgrade_selection - 1
        };
    }

    pub fn upgrade_level(&self, id: UpgradeId) -> u8 {
        self.upgrade_levels.get(id)
    }

    pub fn can_buy_upgrade(&self, id: UpgradeId) -> bool {
        let level = self.upgrade_level(id);
        let def = upgrade_definition(id);
        if level >= def.max_level {
            return false;
        }

        let next_level = level + 1;
        let cost = upgrade_cost(id, next_level);
        self.ship.scrap >= cost.scrap && self.ship.rare_metal >= cost.rare_metal
    }

    pub fn buy_upgrade(&mut self, id: UpgradeId) -> BuyUpgradeResult {
        let current_level = self.upgrade_level(id);
        let def = upgrade_definition(id);
        if current_level >= def.max_level {
            return BuyUpgradeResult::Maxed;
        }

        let next_level = current_level + 1;
        let cost = upgrade_cost(id, next_level);
        if self.ship.scrap < cost.scrap || self.ship.rare_metal < cost.rare_metal {
            return BuyUpgradeResult::NotEnoughResources;
        }

        self.ship.scrap -= cost.scrap;
        self.ship.rare_metal -= cost.rare_metal;
        self.upgrade_levels.set(id, next_level);
        self.apply_permanent_upgrades();
        BuyUpgradeResult::Bought
    }

    pub fn apply_permanent_upgrades(&mut self) {
        let hull_bonus = self.upgrade_level(UpgradeId::ReinforcedHull) as f32 * HULL_HP_PER_LEVEL;
        self.ship.max_health = BASE_SHIP_HP + hull_bonus;
        self.ship.health = self.ship.health.min(self.ship.max_health);
    }

    pub fn player_damage_mult(&self) -> f32 {
        1.0 + (self.upgrade_level(UpgradeId::WeaponTuning) as f32 * WEAPON_DAMAGE_BONUS_PER_LEVEL)
    }

    pub fn player_acceleration(&self) -> f32 {
        let mult = 1.0
            + (self.upgrade_level(UpgradeId::EngineOverdrive) as f32
                * ENGINE_ACCEL_BONUS_PER_LEVEL);
        ACCELERATION * mult
    }

    pub fn loot_magnet_radius(&self) -> f32 {
        BASE_LOOT_MAGNET_RADIUS
            + (self.upgrade_level(UpgradeId::MagnetArray) as f32 * MAGNET_RADIUS_PER_LEVEL)
    }
}

pub fn create_ship() -> Ship {
    Ship {
        pos: vec2(screen_width() / 2.0, screen_height() / 2.0),
        vel: vec2(0.0, 0.0),
        rotation: 0.0,
        health: BASE_SHIP_HP,
        max_health: BASE_SHIP_HP,
        shoot_timer: 0.0,
        rapid_fire_timer: 0.0,
        engine: Engine::basic(),
        scrap: 0,
        rare_metal: 0,
        shield_hp: 0.0,
        shield_max_hp: 0.0,
        shield_timer: 0.0,
        big_bullet_timer: 0.0,
    }
}

pub fn update_timers(game: &mut Game, dt: f32) {
    game.ship.shoot_timer -= dt;
    game.ship.rapid_fire_timer -= dt;
    game.ship.big_bullet_timer -= dt;

    if game.ship.shield_timer > 0.0 {
        game.ship.shield_timer -= dt;
        if game.ship.shield_timer <= 0.0 {
            game.ship.shield_hp = 0.0;
        }
    }

    // Enemy spawning is handled in update_enemies (so last_enemy_spawn_* is set for the Easy hint).

    game.explosions.retain_mut(|e| {
        e.timer += dt;
        if e.timer >= e.frame_time {
            e.timer = 0.0;
            e.frame += 1;
        }
        e.frame < e.max_frames
    });
}

pub fn update_ship_movement(game: &mut Game, dt: f32) {
    if is_key_down(KeyCode::Left) {
        game.ship.rotation -= ROTATION_SPEED * dt;
    }
    if is_key_down(KeyCode::Right) {
        game.ship.rotation += ROTATION_SPEED * dt;
    }

    let rotation_rad = game.ship.rotation.to_radians();
    let ship_dir = vec2(rotation_rad.cos(), rotation_rad.sin());

    let is_gas_pedal_down = is_key_down(KeyCode::Up);
    game.ship.engine.update(dt, is_gas_pedal_down);
    if game.ship.engine.current_thrust > 0.0 {
        let thrust_force = game.ship.engine.current_thrust * game.player_acceleration();
        game.ship.vel += ship_dir * thrust_force * dt;
    }

    game.ship.pos += game.ship.vel * dt;
    wrap_around(&mut game.ship.pos);
}

pub fn update_ship_shooting(game: &mut Game) {
    let current_cooldown = if game.ship.rapid_fire_timer > 0.0 {
        SHOOT_COOLDOWN / 3.0
    } else {
        SHOOT_COOLDOWN
    };

    if is_key_down(KeyCode::Space) && game.ship.shoot_timer <= 0.0 {
        let rotation_rad = game.ship.rotation.to_radians();
        let ship_dir = vec2(rotation_rad.cos(), rotation_rad.sin());

        let damage_mult = game.player_damage_mult();
        let (damage, radius) = if game.ship.big_bullet_timer > 0.0 {
            (BIG_BULLET_DAMAGE, BIG_BULLET_RADIUS)
        } else {
            (PLAYER_BULLET_DAMAGE, PLAYER_BULLET_RADIUS)
        };

        game.bullets.push(Bullet {
            pos: game.ship.pos,
            vel: ship_dir * BULLET_SPEED + game.ship.vel,
            life_time: BULLET_LIFETIME,
            style: BulletStyle::Player,
            damage: damage * damage_mult,
            radius,
        });
        game.ship.shoot_timer = current_cooldown;
    }
}

fn keep_enemy_in_bounds(e: &mut EnemyShip) {
    let margin = ENEMY_BORDER_MARGIN;
    let w = screen_width();
    let h = screen_height();

    if e.pos.x < margin {
        e.vel.x = e.vel.x.max(60.0);
        e.pos.x = e.pos.x.max(margin);
    }
    if e.pos.x > w - margin {
        e.vel.x = e.vel.x.min(-60.0);
        e.pos.x = e.pos.x.min(w - margin);
    }
    if e.pos.y < margin {
        e.vel.y = e.vel.y.max(40.0);
        e.pos.y = e.pos.y.max(margin);
    }
    if e.pos.y > h - margin {
        e.vel.y = e.vel.y.min(-40.0);
        e.pos.y = e.pos.y.min(h - margin);
    }
}

pub fn update_enemies(game: &mut Game, dt: f32) {
    use macroquad::rand::gen_range;

    if game.current_mission.is_boss_level {
        return;
    }

    // On Easy: show hint first, then spawn ship after ENEMY_HINT_DELAY
    if let Some((from_left, spawn_y, hint_at)) = game.pending_enemy_hint {
        if get_time() - hint_at >= ENEMY_HINT_DELAY {
            game.enemy_ships
                .push(EnemyShip::new_at_side(from_left, spawn_y));
            game.pending_enemy_hint = None;
            let base_interval = game.current_mission.enemy_spawn_interval;
            game.enemy_spawn_timer = base_interval / game.difficulty.spawn_rate_mult();
        }
        // While pending, don't decrement timer or start a new spawn
    } else {
        game.enemy_spawn_timer -= dt;
        if game.enemy_spawn_timer <= 0.0 {
            if game.difficulty == Difficulty::Nebula {
                let from_left = gen_range(0, 2) == 0;
                let y = gen_range(50.0, screen_height() - 50.0);
                game.pending_enemy_hint = Some((from_left, y, get_time()));
            } else {
                game.enemy_ships.push(EnemyShip::new());
                let base_interval = game.current_mission.enemy_spawn_interval;
                game.enemy_spawn_timer = base_interval / game.difficulty.spawn_rate_mult();
            }
        }
    }

    for e in game.enemy_ships.iter_mut() {
        let diff = game.ship.pos - e.pos;
        e.rotation = diff.y.atan2(diff.x);

        match e.enemy_type {
            EnemyType::Regular => {
                // Regular enemy: move horizontally and shoot
                e.pos += e.vel * dt;
                e.shoot_timer -= dt;

                if e.shoot_timer <= 0.0 {
                    let bullet_vel = vec2(e.rotation.cos(), e.rotation.sin()) * 250.0;

                    game.bullets.push(Bullet {
                        pos: e.pos,
                        vel: bullet_vel,
                        life_time: 4.0,
                        style: BulletStyle::Enemy,
                        damage: ENEMY_BULLET_DAMAGE,
                        radius: 9.0,
                    });
                    e.shoot_timer = 2.0;
                }
            }
            EnemyType::Kamikaze => {
                // Kamikaze enemy: fly directly toward player
                let dir = diff.normalize();
                let kamikaze_speed = 180.0; // Speed toward player
                e.vel = dir * kamikaze_speed;
                e.pos += e.vel * dt;
            }
        }

        keep_enemy_in_bounds(e);
    }
    game.enemy_ships
        .retain(|e| e.pos.x > -100.0 && e.pos.x < screen_width() + 100.0);
}

const BOSS_SHOOT_INTERVAL: f32 = 3.0;
const BOSS_BURST_SIZE: u32 = 3;
const BOSS_BURST_DELAY: f32 = 0.12;
const BOSS_BULLET_SPEED: f32 = 250.0;
const BOSS_BORDER_MARGIN: f32 = 70.0;
const BOSS_ROTATION_SPEED: f32 = 35.0;

fn keep_boss_in_bounds(boss: &mut Boss) {
    let margin = BOSS_BORDER_MARGIN;
    let w = screen_width();
    let h = screen_height();
    if boss.pos.x < margin {
        boss.vel.x = boss.vel.x.abs();
        boss.pos.x = margin;
    }
    if boss.pos.x > w - margin {
        boss.vel.x = -boss.vel.x.abs();
        boss.pos.x = w - margin;
    }
    if boss.pos.y < margin {
        boss.vel.y = boss.vel.y.abs();
        boss.pos.y = margin;
    }
    if boss.pos.y > h - margin {
        boss.vel.y = -boss.vel.y.abs();
        boss.pos.y = h - margin;
    }
}

const BOSS_VOLLEY_DIRS: [(f32, f32); 4] = [
    (0.0, -1.0), // up
    (0.0, 1.0),  // down
    (-1.0, 0.0), // left
    (1.0, 0.0),  // right
];

pub fn update_boss(game: &mut Game, dt: f32) {
    let mut spawn_volley = false;
    let mut volley_pos = vec2(0.0, 0.0);
    if let Some(ref mut boss) = game.boss {
        boss.pos += boss.vel * dt;
        boss.rotation += BOSS_ROTATION_SPEED * dt * std::f32::consts::PI / 180.0;
        keep_boss_in_bounds(boss);
        boss.shoot_timer -= dt;
        if boss.shoot_timer <= 0.0 {
            if boss.burst_shots_left == 0 {
                boss.burst_shots_left = BOSS_BURST_SIZE;
            }
            spawn_volley = true;
            volley_pos = boss.pos;
            boss.burst_shots_left -= 1;
            boss.shoot_timer = if boss.burst_shots_left > 0 {
                BOSS_BURST_DELAY
            } else {
                BOSS_SHOOT_INTERVAL
            };
        }
    }
    if spawn_volley {
        for (dx, dy) in BOSS_VOLLEY_DIRS {
            game.bullets.push(Bullet {
                pos: volley_pos,
                vel: vec2(dx * BOSS_BULLET_SPEED, dy * BOSS_BULLET_SPEED),
                life_time: 4.0,
                style: BulletStyle::Enemy,
                damage: ENEMY_BULLET_DAMAGE,
                radius: 9.0,
            });
        }
    }
}

pub fn update_loot(game: &mut Game, dt: f32) {
    let mut items_to_remove = Vec::new();
    let magnet_radius = game.loot_magnet_radius();

    for (i, item) in game.loot_items.iter_mut().enumerate() {
        item.vel *= 0.95;
        item.pos += item.vel * dt;
        item.pos += item.drift_vel * dt;
        wrap_around(&mut item.pos);

        item.rotation += item.rotation_speed * dt;
        if item.rotation > std::f32::consts::PI * 2.0 {
            item.rotation -= std::f32::consts::PI * 2.0;
        } else if item.rotation < 0.0 {
            item.rotation += std::f32::consts::PI * 2.0;
        }

        let dist_to_ship = (game.ship.pos - item.pos).length();

        if dist_to_ship < magnet_radius {
            item.magnet_active = true;
        }

        if item.magnet_active {
            let dir = (game.ship.pos - item.pos).normalize();
            let magnet_speed = 300.0;
            item.pos += dir * magnet_speed * dt;
        }

        if dist_to_ship < (72.0 / 2.0 + item.radius) {
            match item.item_type {
                LootType::Scrap(amount) => {
                    game.ship.scrap += amount;
                    game.mission_scrap_collected += amount;
                }
                LootType::RareMetal(amount) => {
                    game.ship.rare_metal += amount;
                    game.mission_rare_metal_collected += amount;
                }
                LootType::HealthPack(hp) => {
                    game.ship.heal(hp as f32);
                }
                LootType::RapidFireBoost => {
                    game.ship.rapid_fire_timer = 10.0;
                }
                LootType::BigBulletBoost => {
                    game.ship.big_bullet_timer = 15.0;
                }
                LootType::Shield(hp) => {
                    game.ship.activate_shield(hp as f32, 30.0);
                }
            }
            items_to_remove.push(i);
        }
    }

    for &i in items_to_remove.iter().rev() {
        game.loot_items.remove(i);
    }
}

pub fn update_physics(game: &mut Game, dt: f32) {
    game.bullets.iter_mut().for_each(|b| {
        b.pos += b.vel * dt;
        b.life_time -= dt;
    });
    game.bullets.retain(|b| b.life_time > 0.0);

    for a in game.asteroids.iter_mut() {
        a.pos += a.vel * dt;
        wrap_around(&mut a.pos);
    }
}

const BOSS_HITBOX_RADIUS: f32 = 55.0;

pub fn update_collisions(game: &mut Game) -> bool {
    let mut new_asteroids = Vec::new();
    let mut game_over = false;
    let mut boss_died_at: Option<(Vec2, f32)> = None;

    // Player bullets vs Enemy bullets (bullets explode each other)
    // Check this FIRST before other collisions
    let mut bullets_to_remove = HashSet::new();
    for (i, player_bullet) in game.bullets.iter().enumerate() {
        if player_bullet.style != BulletStyle::Player {
            continue;
        }
        if bullets_to_remove.contains(&i) {
            continue; // Already marked for removal
        }
        for (j, enemy_bullet) in game.bullets.iter().enumerate().skip(i + 1) {
            if enemy_bullet.style != BulletStyle::Enemy {
                continue;
            }
            if bullets_to_remove.contains(&j) {
                continue; // Already marked for removal
            }
            let distance = (player_bullet.pos - enemy_bullet.pos).length();
            if distance < player_bullet.radius + enemy_bullet.radius {
                // Bullets collide - create explosion at midpoint
                let collision_pos = (player_bullet.pos + enemy_bullet.pos) * 0.5;
                game.explosions.push(Explosion::new(collision_pos, 0.5));
                bullets_to_remove.insert(i);
                bullets_to_remove.insert(j);
                break; // This player bullet is destroyed, move to next
            }
        }
    }
    // Remove collided bullets (sort in reverse to maintain indices)
    let mut sorted_indices: Vec<usize> = bullets_to_remove.into_iter().collect();
    sorted_indices.sort_unstable_by(|a, b| b.cmp(a));
    for &idx in &sorted_indices {
        if idx < game.bullets.len() {
            game.bullets.remove(idx);
        }
    }

    // Player bullets vs asteroids and enemies
    game.bullets.retain(|b| {
        if b.style != BulletStyle::Player {
            return true;
        }

        let mut hit = false;

        // Check asteroid collisions
        for i in (0..game.asteroids.len()).rev() {
            if (b.pos - game.asteroids[i].pos).length() < game.asteroids[i].radius + b.radius {
                game.score += 100;
                let is_rare = game.asteroids[i].is_rare;
                let asteroid_pos = game.asteroids[i].pos;

                if is_rare {
                    if let Some(loot) = generate_loot(
                        asteroid_pos,
                        crate::systems::LootSource::RareAsteroid,
                        game.difficulty,
                    ) {
                        game.loot_items.push(loot);
                    }
                } else if let Some(loot) = generate_loot(
                    asteroid_pos,
                    crate::systems::LootSource::Asteroid,
                    game.difficulty,
                ) {
                    game.loot_items.push(loot);
                }

                let old = game.asteroids.remove(i);
                if old.radius > 15.0 {
                    new_asteroids.push(Asteroid::new_fragment(old.pos, old.radius));
                    new_asteroids.push(Asteroid::new_fragment(old.pos, old.radius));
                }
                hit = true;
                break;
            }
        }

        // Check enemy collisions
        game.enemy_ships.retain_mut(|e| {
            if (b.pos - e.pos).length() < 30.0 + b.radius {
                hit = true;
                if e.take_damage(b.damage) {
                    let score_gain = (e.max_health as u32) * SCORE_PER_ENEMY_HP;
                    game.score += score_gain;
                    if let Some(loot) = generate_loot(
                        e.pos,
                        crate::systems::LootSource::EnemySmall,
                        game.difficulty,
                    ) {
                        game.loot_items.push(loot);
                    }
                    game.mission_kills += 1;
                    game.explosions.push(Explosion::new(e.pos, 0.4));
                    false
                } else {
                    true
                }
            } else {
                true
            }
        });

        // Check boss collision
        if let Some(ref mut boss) = game.boss {
            if (b.pos - boss.pos).length() < BOSS_HITBOX_RADIUS + b.radius {
                hit = true;
                let pos = boss.pos;
                let max_health = boss.max_health;
                if boss.take_damage(b.damage) {
                    boss_died_at = Some((pos, max_health));
                }
            }
        }

        !hit
    });
    if let Some((pos, max_health)) = boss_died_at {
        game.boss = None;
        game.score += (max_health as u32) * SCORE_PER_ENEMY_HP;
        game.explosions.push(Explosion::new(pos, 0.8));
        if let Some(loot) = generate_loot(pos, LootSource::EnemyBoss, game.difficulty) {
            game.loot_items.push(loot);
        }
    }
    game.asteroids.extend(new_asteroids);

    // Enemy bullets vs player
    game.bullets.retain(|b| {
        if b.style == BulletStyle::Enemy && (b.pos - game.ship.pos).length() < 20.0 + b.radius {
            game.explosions.push(Explosion::new(game.ship.pos, 0.5));
            let damage = b.damage * game.difficulty.damage_mult();
            if game.ship.take_damage(damage, game.score) {
                game_over = true;
            }
            false
        } else {
            true
        }
    });

    // Ship vs asteroids
    for i in (0..game.asteroids.len()).rev() {
        if (game.ship.pos - game.asteroids[i].pos).length() < game.asteroids[i].radius + 10.0 {
            let base_asteroid_damage = (game.asteroids[i].radius / 10.0) * BASE_ASTEROID_DAMAGE;
            let asteroid_damage = base_asteroid_damage * game.difficulty.damage_mult();
            let asteroid_radius = game.asteroids[i].radius;
            game.asteroids.remove(i);
            let explosion_scale = (asteroid_radius / 40.0).clamp(0.3, 0.8);
            game.explosions
                .push(Explosion::new(game.ship.pos, explosion_scale));
            if game.ship.take_damage(asteroid_damage, game.score) {
                game_over = true;
            }
        }
    }

    // Ship vs kamikaze enemies (explode on contact)
    game.enemy_ships.retain_mut(|e| {
        if e.enemy_type == EnemyType::Kamikaze {
            let distance = (game.ship.pos - e.pos).length();
            let enemy_radius = 22.5; // Smaller radius for kamikaze (45.0 size / 2)
            let ship_radius = 10.0; // Ship radius approximation
            if distance < enemy_radius + ship_radius {
                // Kamikaze explodes on contact
                let kamikaze_damage = BASE_KAMIKAZE_DAMAGE * game.difficulty.damage_mult();
                game.explosions.push(Explosion::new(e.pos, 0.6));
                if game.ship.take_damage(kamikaze_damage, game.score) {
                    game_over = true;
                }
                // Award score for destroying kamikaze
                let score_gain = (e.max_health as u32) * SCORE_PER_ENEMY_HP;
                game.score += score_gain;
                if let Some(loot) = generate_loot(
                    e.pos,
                    crate::systems::LootSource::EnemySmall,
                    game.difficulty,
                ) {
                    game.loot_items.push(loot);
                }
                game.mission_kills += 1;
                return false; // Remove the kamikaze enemy
            }
        }
        true // Keep the enemy
    });

    game_over
}

pub fn render_game(game: &Game, resources: &Resources) {
    for item in &game.loot_items {
        draw_loot(item, resources);
    }

    for b in &game.bullets {
        let texture = match b.style {
            BulletStyle::Player => &resources.bullet,
            BulletStyle::Enemy => &resources.enemy_bullet,
        };

        let rotation = b.vel.y.atan2(b.vel.x) + std::f32::consts::FRAC_PI_2;
        let size = b.radius * 2.0;

        draw_texture_ex(
            texture,
            b.pos.x - size / 2.0,
            b.pos.y - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                rotation,
                ..Default::default()
            },
        );
    }

    for a in &game.asteroids {
        draw_asteroid(a, resources);
    }

    for e in &game.enemy_ships {
        draw_enemy(e, resources);
    }

    if let Some(ref boss) = game.boss {
        draw_boss(boss, resources);
    }

    for ex in &game.explosions {
        draw_explosion(ex, resources);
    }

    draw_ship(
        &game.ship,
        &resources.ship_body,
        &resources.ship_flame,
        Some(&resources.shield_active),
    );

    let mut status_text = format!(
        "{} {}  {} {:.0}/{:.0}",
        resources.lang.t("score"),
        game.score,
        resources.lang.t("hp"),
        game.ship.health,
        game.ship.max_health
    );
    if game.ship.has_shield() {
        status_text.push_str(&format!(
            "  {} {:.0}/{:.0}",
            resources.lang.t("shield"),
            game.ship.shield_hp,
            game.ship.shield_max_hp
        ));
    }
    crate::draw::draw_text_with_font(&status_text, 20.0, 30.0, 24.0, WHITE, resources);

    let status = if game.current_mission.is_boss_level {
        match &game.boss {
            Some(boss) => format!(
                "{} {:.0}/{:.0}",
                resources.lang.t("boss_hp"),
                boss.health,
                boss.max_health
            ),
            None => resources.lang.t("boss_defeated").to_string(),
        }
    } else {
        format!(
            "{} {}/{}  {} {}/{}  {} {}/{}",
            resources.lang.t("defeated"),
            game.mission_kills,
            game.current_mission.target_kills,
            resources.lang.t("rust"),
            game.mission_scrap_collected,
            game.current_mission.target_scrap,
            resources.lang.t("gold"),
            game.mission_rare_metal_collected,
            game.current_mission.target_rare_metal
        )
    };
    crate::draw::draw_text_with_font(
        &status,
        20.0,
        screen_height() - 30.0,
        24.0,
        WHITE,
        resources,
    );

    let inventory = format!(
        "{} {} {} | {} {}",
        resources.lang.t("resources"),
        resources.lang.t("rust"),
        game.ship.scrap,
        resources.lang.t("gold"),
        game.ship.rare_metal
    );
    crate::draw::draw_text_with_font(
        &inventory,
        20.0,
        screen_height() - 60.0,
        20.0,
        GRAY,
        resources,
    );

    // Easy difficulty: signal shows first, then ship spawns after ENEMY_HINT_DELAY
    if game.difficulty == Difficulty::Nebula {
        if let Some((from_left, spawn_y, _)) = game.pending_enemy_hint {
            crate::draw::draw_enemy_direction_hint(from_left, spawn_y);
        }
    }
}

pub fn render_menu(game: &Game, res: &Resources) {
    draw_background(&res.background);

    // 1. Logo rendering - smaller and at top
    let time = get_time();
    let pulse = 1.0 + (time * 2.0).sin() as f32 * 0.05;

    let target_width = screen_width() * 0.3;
    let aspect_ratio = res.logo.height() / res.logo.width();
    let target_height = target_width * aspect_ratio;

    let logo_w = target_width * pulse;
    let logo_h = target_height * pulse;

    let logo_x = screen_width() / 2.0 - logo_w / 2.0;
    let logo_y = screen_height() / 2.0 - logo_h / 2.0 - 200.0;

    draw_texture_ex(
        &res.logo,
        logo_x,
        logo_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(logo_w, logo_h)),
            ..Default::default()
        },
    );

    // 2. Menu items - centered, with selection highlighting
    let base_y = logo_h / 2.0 - 50.0;
    let item_spacing = 60.0;

    // Font sizes: Start is bigger, others smaller
    let start_font_size = 32;
    let start_selected_font_size = 36;
    let other_font_size = 16;
    let other_selected_font_size = 18;

    // Start menu item - biggest font
    let start_y = base_y + 40.0;
    let is_selected = game.menu_selection == MenuItem::Start;
    let start_color = if is_selected { YELLOW } else { WHITE };
    let start_size = if is_selected {
        start_selected_font_size
    } else {
        start_font_size
    };
    let start_prefix = if is_selected { "> " } else { "  " };
    draw_text_centered(
        &format!("{}{}", start_prefix, res.lang.t("menu_start")),
        start_y,
        start_size,
        start_color,
        res,
    );

    // Difficulty menu item - smaller font, no label
    let diff_y = start_y + item_spacing;
    let is_selected = game.menu_selection == MenuItem::Difficulty;
    let diff_color = if is_selected { YELLOW } else { WHITE };
    let diff_size = if is_selected {
        other_selected_font_size
    } else {
        other_font_size
    };
    let diff_prefix = if is_selected { "> " } else { "  " };

    let diff_key = match game.difficulty {
        Difficulty::Nebula => "diff_nebula",
        Difficulty::Supernova => "diff_supernova",
        Difficulty::BlackHole => "diff_blackhole",
    };
    let diff_text = res.lang.t(diff_key);

    // Use difficulty color when selected, otherwise white
    let final_diff_color = if is_selected {
        match game.difficulty {
            Difficulty::Nebula => GREEN,
            Difficulty::Supernova => YELLOW,
            Difficulty::BlackHole => RED,
        }
    } else {
        diff_color
    };

    draw_text_centered(
        &format!("{diff_prefix}< {diff_text} >"),
        diff_y,
        diff_size,
        final_diff_color,
        res,
    );

    // Language menu item - smaller font, no label
    let lang_y = diff_y + item_spacing;
    let is_selected = game.menu_selection == MenuItem::Language;
    let lang_color = if is_selected { YELLOW } else { WHITE };
    let lang_size = if is_selected {
        other_selected_font_size
    } else {
        other_font_size
    };
    let lang_prefix = if is_selected { "> " } else { "  " };

    let lang_text = match res.lang.current_lang {
        crate::localization::Language::English => res.lang.t("lang_english"),
        crate::localization::Language::Russian => res.lang.t("lang_russian"),
        crate::localization::Language::German => res.lang.t("lang_german"),
    };

    draw_text_centered(
        &format!("{lang_prefix}< {lang_text} >"),
        lang_y,
        lang_size,
        lang_color,
        res,
    );

    // Instructions at bottom
    draw_text_centered(
        res.lang.t("menu_instructions"),
        base_y + 250.0,
        14,
        GRAY,
        res,
    );
}

pub fn render_briefing(mission: &Mission, res: &Resources) {
    draw_text_centered(
        &format!("{} {}", res.lang.t("mission"), mission.level_id),
        -100.0,
        32,
        ORANGE,
        res,
    );
    draw_text_centered(&mission.title, -50.0, 48, WHITE, res);
    draw_text_centered(&mission.description, 0.0, 20, GRAY, res);

    draw_text_centered(res.lang.t("objectives"), 20.0, 24, GRAY, res);

    let obj_text = if mission.is_boss_level {
        res.lang.t("obj_destroy_boss").to_string()
    } else {
        let mut objectives = vec![format!(
            "{} {} {}",
            res.lang.t("obj_destroy_prefix"),
            mission.target_kills,
            res.lang.t("obj_enemies")
        )];
        if mission.target_scrap > 0 {
            objectives.push(format!(
                "{} {} {}",
                res.lang.t("obj_scrap_prefix"),
                mission.target_scrap,
                res.lang.t("obj_rust_piles")
            ));
        }
        if mission.target_rare_metal > 0 {
            objectives.push(format!(
                "{} {} {}",
                res.lang.t("obj_gold_prefix"),
                mission.target_rare_metal,
                res.lang.t("obj_gold")
            ));
        }
        objectives.join("\n")
    };
    draw_text_centered(&obj_text, 70.0, 24, WHITE, res);

    draw_text_centered(res.lang.t("press_space"), 200.0, 24, GREEN, res);
}

pub fn render_mission_success(mission: &Mission, res: &Resources) {
    draw_text_centered(res.lang.t("mission_complete"), -50.0, 40, GREEN, res);
    draw_text_centered(
        &format!(
            "{} {} {}",
            res.lang.t("level_cleared_prefix"),
            mission.level_id,
            res.lang.t("level_cleared_suffix")
        ),
        10.0,
        24,
        WHITE,
        res,
    );
    draw_text_centered(res.lang.t("next_mission"), 100.0, 24, YELLOW, res);
}

pub fn render_upgrade_bay(game: &Game, res: &Resources) {
    draw_text_centered(res.lang.t("upgrade_bay_title"), -260.0, 40, ORANGE, res);
    draw_text_centered(res.lang.t("upgrade_bay_subtitle"), -220.0, 18, GRAY, res);

    let resources_text = format!(
        "{} {} {} | {} {}",
        res.lang.t("resources"),
        res.lang.t("rust"),
        game.ship.scrap,
        res.lang.t("gold"),
        game.ship.rare_metal
    );
    draw_text_centered(&resources_text, -175.0, 22, YELLOW, res);

    let start_y = screen_height() * 0.5 - 110.0;
    let line_height = 52.0;

    for (idx, id) in UPGRADABLE_IDS.iter().copied().enumerate() {
        let def = upgrade_definition(id);
        let level = game.upgrade_level(id);
        let max_level = def.max_level;
        let selected = game.upgrade_selection == idx;
        let line_color = if selected { YELLOW } else { WHITE };
        let prefix = if selected { "> " } else { "  " };

        let status_text = if level >= max_level {
            res.lang.t("upgrade_status_maxed").to_string()
        } else {
            let next_level = level + 1;
            let cost = upgrade_cost(id, next_level);
            let can_buy = game.can_buy_upgrade(id);
            let buy_state = if can_buy {
                res.lang.t("upgrade_status_buy")
            } else {
                res.lang.t("upgrade_status_lack")
            };
            format!(
                "{}: {} {} {} {} ({})",
                res.lang.t("upgrade_cost"),
                res.lang.t("rust"),
                cost.scrap,
                res.lang.t("gold"),
                cost.rare_metal,
                buy_state
            )
        };

        let row_text = format!(
            "{prefix}{}  [{} {}/{}]  {}",
            res.lang.t(def.name_key),
            res.lang.t("upgrade_level"),
            level,
            max_level,
            status_text
        );

        draw_text_with_font(
            &row_text,
            screen_width() * 0.14,
            start_y + line_height * idx as f32,
            20.0,
            line_color,
            res,
        );

        let desc_text = res.lang.t(def.desc_key);
        draw_text_with_font(
            desc_text,
            screen_width() * 0.16,
            start_y + line_height * idx as f32 + 20.0,
            14.0,
            GRAY,
            res,
        );
    }

    let continue_idx = UPGRADABLE_IDS.len();
    let continue_selected = game.upgrade_selection == continue_idx;
    let continue_prefix = if continue_selected { "> " } else { "  " };
    let continue_color = if continue_selected { GREEN } else { WHITE };
    draw_text_with_font(
        &format!("{continue_prefix}{}", res.lang.t("upgrade_continue")),
        screen_width() * 0.14,
        start_y + line_height * continue_idx as f32 + 12.0,
        24.0,
        continue_color,
        res,
    );

    draw_text_centered(res.lang.t("upgrade_controls"), 285.0, 16, GRAY, res);
}

pub fn render_game_over(score: u32, res: &Resources) {
    let high_score = load_score().high_score;
    draw_text_centered(res.lang.t("game_over"), -40.0, 48, RED, res);
    draw_text_centered(
        &format!("{} {}", res.lang.t("final_score_prefix"), score),
        10.0,
        32,
        WHITE,
        res,
    );
    draw_text_centered(
        &format!("{} {}", res.lang.t("high_score"), high_score),
        60.0,
        24,
        YELLOW,
        res,
    );
}

pub fn render_pause(res: &Resources) {
    // Draw semi-transparent overlay
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.7),
    );

    // Draw pause text
    draw_text_centered(res.lang.t("paused"), -20.0, 48, YELLOW, res);
    draw_text_centered(res.lang.t("press_esc"), 30.0, 24, WHITE, res);
}
