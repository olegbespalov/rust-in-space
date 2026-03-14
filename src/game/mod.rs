pub mod constants;
pub mod npc;
pub mod physics;
pub mod player;

use crate::components::*;
use crate::systems::get_mission;
use constants::*;
use macroquad::prelude::*;

pub use npc::{update_boss, update_enemies};
pub use physics::{update_collisions, update_loot, update_physics, update_timers};
pub use player::create_ship;
pub use player::{update_ship_movement, update_ship_shooting};

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

        self.ship.pos = vec2(screen_width() / 2.0, screen_height() / 2.0);
        self.ship.vel = vec2(0.0, 0.0);
        self.ship.rotation = 0.0;
        self.ship.engine.current_thrust = 0.0;
        self.apply_permanent_upgrades();
        self.ship.health = self.ship.max_health;

        if self.upgrade_levels.get(UpgradeId::ShieldCapacitor) > 0 && !self.ship.has_shield() {
            self.ship
                .activate_shield(SHIELD_CAPACITOR_HP, SHIELD_CAPACITOR_DURATION);
        }
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

        let cost = upgrade_cost(id, level + 1);
        self.ship.scrap >= cost.scrap && self.ship.rare_metal >= cost.rare_metal
    }

    pub fn buy_upgrade(&mut self, id: UpgradeId) -> BuyUpgradeResult {
        let current_level = self.upgrade_level(id);
        let def = upgrade_definition(id);
        if current_level >= def.max_level {
            return BuyUpgradeResult::Maxed;
        }

        let cost = upgrade_cost(id, current_level + 1);
        if self.ship.scrap < cost.scrap || self.ship.rare_metal < cost.rare_metal {
            return BuyUpgradeResult::NotEnoughResources;
        }

        self.ship.scrap -= cost.scrap;
        self.ship.rare_metal -= cost.rare_metal;
        self.upgrade_levels.set(id, current_level + 1);
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
