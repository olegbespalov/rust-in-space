use crate::components::Mission;
use crate::components::SaveData;
use crate::components::{LootItem, LootType};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::fs;

// Where does the item drop from?
pub enum LootSource {
    Asteroid,
    RareAsteroid,
    EnemySmall,
    // EnemyBoss, // For future
}

pub fn wrap_around(pos: &mut Vec2) {
    if pos.x < -20.0 {
        pos.x = screen_width() + 20.0;
    } else if pos.x > screen_width() + 20.0 {
        pos.x = -20.0;
    }

    if pos.y < -20.0 {
        pos.y = screen_height() + 20.0;
    } else if pos.y > screen_height() + 20.0 {
        pos.y = -20.0;
    }
}

const SAVE_FILE: &str = "highscore.json";

pub fn save_score(score: u32) {
    let current_data = load_score();
    if score > current_data.high_score {
        let new_data = SaveData { high_score: score };
        if let Ok(json) = serde_json::to_string(&new_data) {
            let _ = fs::write(SAVE_FILE, json);
        }
    }
}

pub fn load_score() -> SaveData {
    if let Ok(content) = fs::read_to_string(SAVE_FILE) {
        if let Ok(data) = serde_json::from_str::<SaveData>(&content) {
            return data;
        }
    }
    SaveData::new()
}

pub fn get_mission(level: u32) -> Mission {
    match level {
        1 => Mission {
            level_id: 1,
            title: "Operation: Dust".to_string(),
            description: "Destroy 3 scouts and collect resources.".to_string(),
            target_kills: 3,
            target_scrap: 1,
            target_rare_metal: 0,
            enemy_spawn_interval: 10.0, // enemies spawn rarely
            asteroid_count: 5,
        },
        2 => Mission {
            level_id: 2,
            title: "Into the Void".to_string(),
            description: "Enemy activity rising. Kill 10 enemies.".to_string(),
            target_kills: 10,
            target_scrap: 0, // scrap is not important
            target_rare_metal: 0,
            enemy_spawn_interval: 2.0,
            asteroid_count: 8,
        },
        3 => Mission {
            level_id: 3,
            title: "Scrap Yard".to_string(),
            description: "Collect 20 rust piles and 3 gold for upgrades.".to_string(),
            target_kills: 5,
            target_scrap: 20,
            target_rare_metal: 3,
            enemy_spawn_interval: 2.5,
            asteroid_count: 12,
        },
        _ => Mission {
            // generate infinite levels after the 3rd one
            level_id: level,
            title: format!("Deep Space sector {level}"),
            description: "Survive.".to_string(),
            target_kills: 10 + level,
            target_scrap: 10 + (level / 2),
            target_rare_metal: 2 + (level / 3),
            enemy_spawn_interval: (1.5 - (level as f32 * 0.1)).max(0.5),
            asteroid_count: 10 + level as usize,
        },
    }
}

pub fn generate_loot(pos: Vec2, source: LootSource) -> Option<LootItem> {
    let roll = gen_range(0, 100);

    let (item_type, radius) = match source {
        LootSource::Asteroid => {
            if roll < 55 {
                (LootType::Scrap(gen_range(1, 4)), 10.0)
            }
            // 55% chance of scrap (increased from 40%)
            else if roll < 65 {
                (LootType::RareMetal(1), 12.0)
            }
            // 10% chance of rare metal (increased from 5%)
            else {
                return None;
            } // 35% chance of nothing (decreased from 55%)
        }
        LootSource::RareAsteroid => {
            // Rare asteroids always drop loot (100% chance)
            if roll < 30 {
                (LootType::RareMetal(gen_range(2, 5)), 12.0)
            }
            // 30% chance of rare metal
            else if roll < 45 {
                (LootType::Scrap(gen_range(5, 10)), 10.0)
            }
            // 15% chance of scrap
            else if roll < 70 {
                (LootType::HealthPack(25), 15.0)
            }
            // 25% chance of health pack
            else if roll < 88 {
                (LootType::RapidFireBoost, 15.0)
            }
            // 18% chance of rapid fire boost
            else {
                // 12% chance of big bullet boost (roll 88-99)
                (LootType::BigBulletBoost, 15.0)
            }
        }
        LootSource::EnemySmall => {
            if roll < 30 {
                (LootType::Scrap(gen_range(5, 10)), 10.0)
            }
            // 30% chance of scrap
            else if roll < 55 {
                (LootType::HealthPack(25), 15.0)
            }
            // 25% health pack
            else if roll < 73 {
                (LootType::RapidFireBoost, 15.0)
            }
            // 18% rapid fire boost
            else if roll < 85 {
                (LootType::BigBulletBoost, 15.0)
            }
            // 12% big bullet boost
            else if roll < 93 {
                // 8% chance of shield with varying HP (30-100 HP)
                (LootType::Shield(gen_range(30, 101)), 15.0)
            } else {
                return None;
            }
            // 7% chance of nothing
        } // LootSource::EnemyBoss => {
          //     // Something always drops from the boss
          //     (LootType::RareMetal(gen_range(10, 50)), 20.0)
          // }
    };

    // Random slow drift velocity (super slow, like floating in space)
    let drift_speed = gen_range(5.0, 15.0);
    let drift_angle = gen_range(0.0, std::f32::consts::PI * 2.0);
    let drift_vel = vec2(drift_angle.cos(), drift_angle.sin()) * drift_speed;

    // Random rotation speed (can be positive or negative for random direction)
    let rotation_speed = gen_range(-1.5, 1.5);

    Some(LootItem {
        pos,
        vel: vec2(gen_range(-50.0, 50.0), gen_range(-50.0, 50.0)), // Fly apart on explosion
        drift_vel,
        item_type,
        radius,
        magnet_active: false,
        rotation: gen_range(0.0, std::f32::consts::PI * 2.0), // Random initial rotation
        rotation_speed,
    })
}
