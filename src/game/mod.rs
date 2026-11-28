pub mod constants;
mod npc;
mod physics;
mod player;

use crate::audio::AudioCue;
use crate::components::*;
use crate::systems::get_mission;
use constants::*;
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

pub(crate) use player::advance_ship_movement;
use player::create_ship;
pub use player::PilotInput;

pub const MAX_PILOTS: usize = 4;
const SPAWN_OFFSETS: [Vec2; MAX_PILOTS] = [
    vec2(-60.0, 0.0),
    vec2(60.0, 0.0),
    vec2(0.0, -60.0),
    vec2(0.0, 60.0),
];

pub const UPGRADABLE_IDS: [UpgradeId; 6] = [
    UpgradeId::ReinforcedHull,
    UpgradeId::WeaponTuning,
    UpgradeId::DualCannons,
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
        UpgradeId::DualCannons => UpgradeDefinition {
            name_key: "upgrade_cannon_name",
            desc_key: "upgrade_cannon_desc",
            max_level: 1,
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
            scrap: 0,
            rare_metal: 12 + 15 * (next_level as u32 - 1),
        },
        UpgradeId::WeaponTuning => UpgradeCost {
            scrap: 0,
            rare_metal: 18 + 15 * (next_level as u32 - 1),
        },
        UpgradeId::DualCannons => UpgradeCost {
            scrap: 0,
            rare_metal: 30,
        },
        UpgradeId::EngineOverdrive => UpgradeCost {
            scrap: 0,
            rare_metal: 15 + 12 * (next_level as u32 - 1),
        },
        UpgradeId::MagnetArray => UpgradeCost {
            scrap: 0,
            rare_metal: 12 + 12 * (next_level as u32 - 1),
        },
        UpgradeId::ShieldCapacitor => UpgradeCost {
            scrap: 0,
            rare_metal: 36,
        },
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct MissionProgress {
    pub enemies_defeated: u32,
    pub asteroids_destroyed: u32,
    pub scrap_collected: u32,
    pub elapsed_seconds: f64,
    pub boss_defeated: bool,
    interceptors_defeated: u32,
    heavy_ships_defeated: u32,
}

impl MissionProgress {
    pub fn record_enemy_defeat(&mut self, enemy_type: EnemyType) {
        self.enemies_defeated += 1;
        match enemy_type {
            EnemyType::Interceptor => self.interceptors_defeated += 1,
            EnemyType::Heavy => self.heavy_ships_defeated += 1,
            EnemyType::Regular | EnemyType::Kamikaze | EnemyType::Guardian => {}
        }
    }

    pub fn amount(&self, objective: MissionObjective) -> u32 {
        match objective {
            MissionObjective::DefeatEnemies(_) => self.enemies_defeated,
            MissionObjective::DestroyAsteroids(_) => self.asteroids_destroyed,
            MissionObjective::DefeatInterceptors(_) => self.interceptors_defeated,
            MissionObjective::DefeatHeavyShips(_) => self.heavy_ships_defeated,
            MissionObjective::CollectScrap(_) => self.scrap_collected,
            MissionObjective::Survive(_) => self.elapsed_seconds as u32,
            MissionObjective::DefeatBoss => u32::from(self.boss_defeated),
        }
    }

    pub fn meets(&self, objective: MissionObjective) -> bool {
        self.amount(objective) >= objective.target()
    }

    fn completes(&self, mission: &Mission) -> bool {
        mission
            .objectives
            .iter()
            .all(|&objective| self.meets(objective))
    }
}

// LAN guests use this state for display. Simulation-only fields can be skipped
// here and on its entity types.
#[derive(Serialize, Deserialize)]
pub struct Game {
    pub pilots: [Option<Pilot>; MAX_PILOTS],
    pub bounds: Vec2,
    pub bullets: Vec<Bullet>,
    pub asteroids: Vec<Asteroid>,
    pub enemy_ships: Vec<EnemyShip>,
    pub loot_items: Vec<LootItem>,
    pub explosions: Vec<Explosion>,
    pub score: u32,
    pub current_mission: Mission,
    pub mission_progress: MissionProgress,
    #[serde(skip)]
    enemy_spawn_timer: f32,
    #[serde(skip)]
    enemy_spawn_count: u32,
    #[serde(skip)]
    enemy_hint_delay: f64,
    pub difficulty: Difficulty,
    pub pending_enemy_hint: Option<(bool, f32, f64)>,
    pub boss: Option<Boss>,
    #[serde(skip)]
    pub audio_cues: Vec<AudioCue>,
}

pub enum BattleOutcome {
    Playing,
    Defeat,
    Success,
}

impl Game {
    pub fn new(bounds: Vec2) -> Self {
        Self {
            pilots: std::array::from_fn(|slot| {
                (slot == 0).then(|| Pilot::new(ShipKind::Alpha, bounds / 2.0))
            }),
            bounds,
            bullets: Vec::new(),
            asteroids: Vec::new(),
            enemy_ships: Vec::new(),
            loot_items: Vec::new(),
            explosions: Vec::new(),
            score: 0,
            current_mission: get_mission(1),
            mission_progress: MissionProgress::default(),
            enemy_spawn_timer: 0.0,
            enemy_spawn_count: 0,
            enemy_hint_delay: ENEMY_HINT_DELAY,
            difficulty: Difficulty::Supernova,
            pending_enemy_hint: None,
            boss: None,
            audio_cues: Vec::new(),
        }
    }

    pub fn drain_audio_cues(&mut self) -> Vec<AudioCue> {
        std::mem::take(&mut self.audio_cues)
    }

    pub fn reset(&mut self, ship_kind: ShipKind, bounds: Vec2) {
        let difficulty = self.difficulty;
        *self = Self::new(bounds);
        self.difficulty = difficulty;
        self.set_pilot(0, ship_kind);
    }

    pub fn set_pilot(&mut self, slot: usize, kind: ShipKind) {
        self.pilots[slot] = Some(Pilot::new(kind, self.bounds / 2.0));
    }

    pub fn solo_pilot(&self) -> &Pilot {
        self.pilots[0].as_ref().expect("solo pilot exists")
    }

    pub fn solo_pilot_mut(&mut self) -> &mut Pilot {
        self.pilots[0].as_mut().expect("solo pilot exists")
    }

    pub fn start_mission(&mut self) {
        self.bullets.clear();
        self.enemy_ships.clear();
        self.loot_items.clear();
        let cooperative = self.pilot_count() > 1;
        self.boss = self
            .current_mission
            .objectives
            .contains(&MissionObjective::DefeatBoss)
            .then(|| {
                let mut boss = Boss::new(
                    vec2(self.bounds.x / 2.0, BOSS_SIZE / 2.0),
                    npc::boss_summon_delay(BossPhase::Opening, self.difficulty),
                );
                if cooperative {
                    boss.max_health *= 1.5;
                    boss.health = boss.max_health;
                }
                boss
            });
        self.mission_progress = MissionProgress::default();
        self.enemy_spawn_timer = self.current_mission.enemy_spawn_interval;
        self.enemy_spawn_count = 0;
        self.enemy_hint_delay = if cooperative {
            ENEMY_HINT_DELAY / f64::from(COOP_ENEMY_SPAWN_RATE)
        } else {
            ENEMY_HINT_DELAY
        };
        self.pending_enemy_hint = None;

        for (pilot, offset) in self.pilots.iter_mut().zip(SPAWN_OFFSETS) {
            if let Some(pilot) = pilot {
                let offset = if cooperative { offset } else { Vec2::ZERO };
                pilot.start_mission(self.bounds / 2.0 + offset);
            }
        }

        let pilot_positions: Vec<Vec2> = self
            .pilots
            .iter()
            .flatten()
            .filter(|pilot| pilot.is_alive())
            .map(|pilot| pilot.ship.pos)
            .collect();
        self.asteroids = (0..self.current_mission.asteroid_count)
            .map(|_| {
                let mut asteroid = Asteroid::new_large(self.bounds);
                for _ in 0..32 {
                    if pilot_positions
                        .iter()
                        .all(|&pos| (asteroid.pos - pos).length() >= asteroid.radius + 80.0)
                    {
                        break;
                    }
                    asteroid = Asteroid::new_large(self.bounds);
                }
                asteroid
            })
            .collect();
    }

    pub fn next_mission(&mut self) {
        self.current_mission = get_mission(self.current_mission.level_id + 1);
    }

    pub fn prepare_lan_mission(&mut self) {
        self.current_mission = lan_mission(
            get_mission(self.current_mission.level_id),
            self.pilot_count() > 1,
        );
    }

    pub fn step(
        &mut self,
        inputs: [PilotInput; MAX_PILOTS],
        bounds: Vec2,
        dt: f32,
        now: f64,
    ) -> BattleOutcome {
        self.bounds = bounds;
        physics::update_timers(self, dt);
        player::update_pilots(self, inputs, dt);
        npc::update_enemies(self, dt, now);
        npc::update_boss(self, dt);
        physics::update_loot(self, dt);
        if self.has_survivors() {
            physics::update_physics(self, dt);
            physics::update_collisions(self, dt);
        }
        if !self.has_survivors() {
            return BattleOutcome::Defeat;
        }
        npc::update_boss_encounter(self, dt);

        if self.mission_progress.completes(&self.current_mission) {
            BattleOutcome::Success
        } else {
            BattleOutcome::Playing
        }
    }

    pub fn has_survivors(&self) -> bool {
        self.pilots.iter().flatten().any(Pilot::is_alive)
    }

    pub fn pilot_count(&self) -> usize {
        self.pilots.iter().flatten().count()
    }

    pub fn cycle_difficulty(&mut self) {
        self.difficulty = match self.difficulty {
            Difficulty::Nebula => Difficulty::Supernova,
            Difficulty::Supernova => Difficulty::BlackHole,
            Difficulty::BlackHole => Difficulty::Nebula,
        };
    }
}

fn nearest_pilot(
    pilots: &[Option<Pilot>; MAX_PILOTS],
    pos: Vec2,
    eligible: impl Fn(&Pilot) -> bool,
) -> Option<usize> {
    pilots
        .iter()
        .enumerate()
        .filter_map(|(slot, pilot)| {
            let pilot = pilot
                .as_ref()
                .filter(|pilot| pilot.is_alive() && eligible(pilot))?;
            Some((slot, pilot.ship.pos.distance_squared(pos)))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)))
        .map(|(slot, _)| slot)
}

fn lan_mission(mut mission: Mission, cooperative: bool) -> Mission {
    use MissionObjective::*;
    if cooperative {
        let scale = |n: u32| n.saturating_add(n.div_ceil(2));
        for objective in &mut mission.objectives {
            *objective = match *objective {
                DefeatEnemies(n) => DefeatEnemies(scale(n)),
                DestroyAsteroids(n) => DestroyAsteroids(scale(n)),
                DefeatInterceptors(n) => DefeatInterceptors(scale(n)),
                DefeatHeavyShips(n) => DefeatHeavyShips(scale(n)),
                CollectScrap(n) => CollectScrap(scale(n)),
                Survive(_) | DefeatBoss => *objective,
            };
        }
        mission.asteroid_count = mission
            .asteroid_count
            .saturating_add(mission.asteroid_count.div_ceil(2));
        mission.enemy_spawn_interval /= COOP_ENEMY_SPAWN_RATE;
    }
    mission
}

#[derive(Default)]
pub struct UpgradeSelection(pub usize);

impl UpgradeSelection {
    pub const SELL_SCRAP_INDEX: usize = UPGRADABLE_IDS.len();
    pub const CONTINUE_INDEX: usize = UPGRADABLE_IDS.len() + 1;

    pub fn selected(&self) -> Option<UpgradeId> {
        UPGRADABLE_IDS.get(self.0).copied()
    }

    pub fn next(&mut self) {
        self.0 = (self.0 + 1) % (UPGRADABLE_IDS.len() + 2);
    }

    pub fn prev(&mut self) {
        self.0 = (self.0 + UPGRADABLE_IDS.len() + 1) % (UPGRADABLE_IDS.len() + 2);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Pilot {
    pub ship: Ship,
    pub upgrade_levels: UpgradeLevels,
    #[serde(skip)]
    mission_points: f32,
}

impl Pilot {
    fn new(kind: ShipKind, pos: Vec2) -> Self {
        Self {
            ship: create_ship(kind, pos),
            upgrade_levels: UpgradeLevels::default(),
            mission_points: 0.0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.ship.health > 0.0
    }

    pub fn mission_points(&self) -> u32 {
        self.mission_points.round() as u32
    }

    fn start_mission(&mut self, pos: Vec2) {
        self.mission_points = 0.0;
        self.ship.pos = pos;
        self.ship.vel = Vec2::ZERO;
        self.ship.rotation = 0.0;
        self.ship.engine.current_thrust = 0.0;
        self.apply_permanent_upgrades();
        self.ship.health = self.ship.max_health;
        if self.upgrade_levels.get(UpgradeId::ShieldCapacitor) > 0 && !self.ship.has_shield() {
            self.ship.activate_shield(
                ShieldKind::Standard,
                SHIELD_CAPACITOR_HP,
                SHIELD_CAPACITOR_DURATION,
            );
        }
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

    pub fn sell_scrap(&mut self) -> u32 {
        const SCRAP_PER_GOLD: u32 = 7;
        let gold = self.ship.scrap / SCRAP_PER_GOLD;
        if gold > 0 {
            self.ship.scrap -= gold * SCRAP_PER_GOLD;
            self.ship.rare_metal += gold;
        }
        gold
    }

    fn apply_permanent_upgrades(&mut self) {
        let hull_bonus = self.upgrade_level(UpgradeId::ReinforcedHull) as f32 * HULL_HP_PER_LEVEL;
        self.ship.max_health = BASE_SHIP_HP + hull_bonus;
        self.ship.health = self.ship.health.min(self.ship.max_health);
    }

    fn damage_mult(&self) -> f32 {
        1.0 + (self.upgrade_level(UpgradeId::WeaponTuning) as f32 * WEAPON_DAMAGE_BONUS_PER_LEVEL)
    }

    pub(crate) fn acceleration(&self) -> f32 {
        let mult = 1.0
            + (self.upgrade_level(UpgradeId::EngineOverdrive) as f32
                * ENGINE_ACCEL_BONUS_PER_LEVEL);
        ACCELERATION * mult
    }

    fn loot_magnet_radius(&self) -> f32 {
        BASE_LOOT_MAGNET_RADIUS
            + (self.upgrade_level(UpgradeId::MagnetArray) as f32 * MAGNET_RADIUS_PER_LEVEL)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_objectives_are_reachable_in_solo_and_coop() {
        let bounds = vec2(1280.0, 720.0);
        let asteroid_target = vec2(200.0, 200.0);
        for cooperative in [false, true] {
            for difficulty in [
                Difficulty::Nebula,
                Difficulty::Supernova,
                Difficulty::BlackHole,
            ] {
                let mut game = Game::new(bounds);
                game.difficulty = difficulty;
                if cooperative {
                    game.set_pilot(1, ShipKind::Alpha);
                }

                // Include missions 1–5 and all three generated objective sets (6–8).
                for level in 1..=8 {
                    if cooperative {
                        game.prepare_lan_mission();
                    }
                    game.start_mission();
                    let objectives = game.current_mission.objectives.clone();
                    for &objective in &objectives {
                        if let MissionObjective::CollectScrap(target) = objective {
                            // Exercise scrap pickup without relying on random drops.
                            let pos = game.solo_pilot().ship.pos;
                            let mut scrap =
                                crate::systems::spawn_loot(pos, LootType::Scrap(target), 10.0);
                            scrap.vel = Vec2::ZERO;
                            scrap.drift_vel = Vec2::ZERO;
                            game.loot_items.push(scrap);
                        }
                    }

                    let mut outcome = BattleOutcome::Playing;
                    let dt = 0.25;
                    for tick in 1..=720 {
                        // Remove combat difficulty while retaining the actual spawn and objective logic.
                        for enemy in &mut game.enemy_ships {
                            enemy.health = 0.0;
                        }
                        if let Some(boss) = &mut game.boss {
                            boss.health = 0.0;
                        }
                        for asteroid in &mut game.asteroids {
                            asteroid.pos = asteroid_target;
                            asteroid.vel = Vec2::ZERO;
                        }
                        if !game.asteroids.is_empty()
                            && objectives.iter().any(|&objective| {
                                matches!(objective, MissionObjective::DestroyAsteroids(_))
                                    && !game.mission_progress.meets(objective)
                            })
                        {
                            game.bullets.push(Bullet {
                                pos: asteroid_target,
                                vel: Vec2::ZERO,
                                owner: Some(0),
                                life_time: BULLET_LIFETIME,
                                style: BulletStyle::Player,
                                homing: false,
                                damage: PLAYER_BULLET_DAMAGE,
                                radius: PLAYER_BULLET_RADIUS,
                            });
                        }

                        outcome = game.step(
                            [PilotInput::default(); MAX_PILOTS],
                            bounds,
                            dt,
                            f64::from(tick) * f64::from(dt),
                        );
                        if !matches!(outcome, BattleOutcome::Playing) {
                            break;
                        }
                    }

                    for objective in objectives {
                        let amount = game.mission_progress.amount(objective);
                        assert!(
                            game.mission_progress.meets(objective),
                            "mission {level}, coop={cooperative}: {objective:?} at {amount}/{}",
                            objective.target(),
                        );
                    }
                    assert!(
                        matches!(outcome, BattleOutcome::Success),
                        "mission {level}, coop={cooperative}"
                    );
                    game.next_mission();
                }
            }
        }
    }

    #[test]
    fn coop_scales_every_count_and_preserves_duration_and_boss_objectives() {
        use MissionObjective::*;
        let base = Mission {
            level_id: 1,
            title: String::new(),
            objectives: vec![
                DefeatEnemies(8),
                DestroyAsteroids(3),
                DefeatInterceptors(5),
                DefeatHeavyShips(1),
                CollectScrap(35),
                Survive(60),
                DefeatBoss,
            ],
            enemy_spawn_interval: 5.0,
            asteroid_count: 9,
        };
        let coop = lan_mission(base.clone(), true);
        assert_eq!(
            coop.objectives
                .iter()
                .map(|objective| objective.target())
                .collect::<Vec<_>>(),
            vec![12, 5, 8, 2, 53, 60, 1]
        );
        assert!(coop.objectives.contains(&DefeatBoss));
        assert_eq!(coop.asteroid_count, 14);
        assert_eq!(coop.enemy_spawn_interval, 4.0);
        let solo = lan_mission(base, false);
        assert_eq!(solo.asteroid_count, 9);
        assert_eq!(solo.enemy_spawn_interval, 5.0);
    }

    #[test]
    fn asteroid_clearing_requires_both_objectives() {
        let mission = get_mission(1);
        let mut progress = MissionProgress {
            enemies_defeated: 8,
            asteroids_destroyed: 9,
            ..Default::default()
        };
        assert!(!progress.completes(&mission));
        progress.asteroids_destroyed += 1;
        assert!(progress.completes(&mission));
    }

    #[test]
    fn patrol_requires_full_survival_time_and_interceptor_kills() {
        let mission = get_mission(2);
        let mut progress = MissionProgress {
            elapsed_seconds: 59.99,
            ..Default::default()
        };
        progress.record_enemy_defeat(EnemyType::Interceptor);
        progress.record_enemy_defeat(EnemyType::Interceptor);
        progress.record_enemy_defeat(EnemyType::Regular);
        progress.record_enemy_defeat(EnemyType::Interceptor);
        assert!(!progress.completes(&mission));

        progress.elapsed_seconds = 60.0;
        assert!(!progress.completes(&mission));
        progress.record_enemy_defeat(EnemyType::Interceptor);
        progress.record_enemy_defeat(EnemyType::Interceptor);
        assert!(progress.completes(&mission));
        assert!(!MissionProgress {
            elapsed_seconds: 60.0,
            enemies_defeated: 12,
            ..Default::default()
        }
        .completes(&mission));
    }

    #[test]
    fn dreadnought_and_salvage_each_require_two_objectives() {
        let boss_mission = get_mission(5);
        let mut progress = MissionProgress {
            boss_defeated: true,
            ..Default::default()
        };
        assert!(!progress.completes(&boss_mission));
        for _ in 0..6 {
            progress.record_enemy_defeat(EnemyType::Regular);
        }
        assert!(progress.completes(&boss_mission));

        progress.boss_defeated = false;
        assert!(!progress.completes(&boss_mission));
        progress.boss_defeated = true;
        assert!(progress.completes(&boss_mission));

        let mut progress = MissionProgress {
            scrap_collected: 35,
            ..Default::default()
        };
        progress.record_enemy_defeat(EnemyType::Heavy);
        assert!(!progress.completes(&get_mission(4)));
        progress.record_enemy_defeat(EnemyType::Heavy);
        assert!(!progress.completes(&get_mission(4)));
        progress.record_enemy_defeat(EnemyType::Heavy);
        assert!(progress.completes(&get_mission(4)));
    }

    #[test]
    fn dreadnought_appears_in_mission_five_and_awards_loot_on_defeat() {
        let bounds = vec2(1280.0, 720.0);
        let mut game = Game::new(bounds);
        for level in 1..5 {
            assert_eq!(game.current_mission.level_id, level);
            game.start_mission();
            assert!(game.boss.is_none(), "mission {level}");
            game.next_mission();
        }
        assert_eq!(game.current_mission.level_id, 5);
        game.start_mission();
        for _ in 0..6 {
            game.mission_progress
                .record_enemy_defeat(EnemyType::Regular);
        }

        let boss = game.boss.as_mut().unwrap();
        boss.health = 1.0;
        let pos = boss.pos + vec2(80.0, 24.0);
        game.bullets.push(Bullet {
            pos,
            vel: Vec2::ZERO,
            owner: Some(0),
            life_time: BULLET_LIFETIME,
            style: BulletStyle::Player,
            homing: false,
            damage: PLAYER_BULLET_DAMAGE,
            radius: PLAYER_BULLET_RADIUS,
        });

        let outcome = game.step([PilotInput::default(); MAX_PILOTS], bounds, 0.0, 0.0);
        assert!(matches!(outcome, BattleOutcome::Success));
        assert!(game.boss.is_none());
        assert_eq!(game.score, BOSS_MAX_HEALTH as u32 * SCORE_PER_ENEMY_HP);
        assert_eq!(game.solo_pilot().mission_points(), 1);
        assert!(matches!(
            game.loot_items[0].item_type,
            LootType::RareMetal(10..=49)
        ));

        game.next_mission();
        assert_eq!(game.current_mission.level_id, 6);
        game.start_mission();
        assert!(game.boss.is_none());
        assert_eq!(game.solo_pilot().mission_points(), 0);
    }
}
