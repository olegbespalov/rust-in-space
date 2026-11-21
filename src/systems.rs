use crate::components::Mission;
use crate::components::SaveData;
use crate::components::{Difficulty, LootItem, LootType};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::fs;
use std::path::PathBuf;

// Where does the item drop from?
pub enum LootSource {
    Asteroid,
    RareAsteroid,
    EnemySmall,
    EnemyBoss,
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

fn save_file() -> PathBuf {
    #[cfg(target_os = "macos")]
    if std::env::current_exe()
        .ok()
        .and_then(|executable| crate::resources::macos_bundle_resources(&executable))
        .is_some()
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library/Application Support/Rust in Space/highscore.json");
        }
    }

    PathBuf::from(SAVE_FILE)
}

pub fn save_data(data: &SaveData) {
    if let Ok(json) = serde_json::to_string(data) {
        let path = save_file();
        let result = match path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            Some(parent) => fs::create_dir_all(parent),
            None => Ok(()),
        }
        .and_then(|()| fs::write(&path, json));
        if let Err(error) = result {
            eprintln!("Failed to write {}: {error}", path.display());
        }
    }
}

pub fn save_score(score: u32) {
    let current_data = load_score();
    if score > current_data.high_score {
        let new_data = SaveData {
            high_score: score,
            audio: current_data.audio,
        };
        save_data(&new_data);
    }
}

pub fn save_audio_settings(audio: crate::components::AudioSettings) {
    let mut current_data = load_score();
    current_data.audio = audio;
    save_data(&current_data);
}

pub fn load_score() -> SaveData {
    match fs::read_to_string(save_file()) {
        Ok(content) => match serde_json::from_str::<SaveData>(&content) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to parse highscore file: {}", e);
                SaveData::new()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => SaveData::new(),
        Err(e) => {
            eprintln!("Failed to read highscore file: {}", e);
            SaveData::new()
        }
    }
}

pub fn get_mission(level: u32) -> Mission {
    use crate::components::MissionObjective::*;

    match level {
        1 => Mission {
            level_id: 1,
            title: "Operation: Dust".to_string(),
            objectives: vec![DefeatEnemies(5), DestroyAsteroids(8)],
            enemy_spawn_interval: 5.5,
            asteroid_count: 7,
        },
        2 => Mission {
            level_id: 2,
            title: "Into the Void".to_string(),
            objectives: vec![Survive(45), DefeatInterceptors(3)],
            enemy_spawn_interval: 2.0,
            asteroid_count: 8,
        },
        3 => Mission {
            level_id: 3,
            title: "Boss: Guardian".to_string(),
            objectives: vec![DefeatBoss, DefeatEnemies(4)],
            enemy_spawn_interval: 6.0,
            asteroid_count: 0,
        },
        4 => Mission {
            level_id: 4,
            title: "Scrap Yard".to_string(),
            objectives: vec![CollectScrap(24), DefeatHeavyShips(2)],
            enemy_spawn_interval: 2.5,
            asteroid_count: 12,
        },
        _ => Mission {
            level_id: level,
            title: format!("Deep Space sector {level}"),
            objectives: match level % 3 {
                2 => vec![Survive(50 + level.min(30)), DefeatHeavyShips(3)],
                0 => vec![
                    DefeatEnemies(level.saturating_add(10)),
                    DefeatInterceptors(4),
                ],
                _ => vec![CollectScrap(25 + level / 2), DefeatHeavyShips(3)],
            },
            enemy_spawn_interval: (1.5 - ((level - 1) as f32 * 0.1)).max(0.5),
            asteroid_count: 10 + (level - 1) as usize,
        },
    }
}

pub fn generate_loot(pos: Vec2, source: LootSource, difficulty: Difficulty) -> Option<LootItem> {
    // Base roll (0-100) + Difficulty modifier
    // On easy (Nebula) roll will be 10-110 (more chances on rare)
    // On hard (Black Hole) roll will be -15-85 (less chances)
    let roll = gen_range(0, 100) + difficulty.loot_luck_modifier();

    let (item_type, radius) = match source {
        LootSource::Asteroid => {
            if roll < 40 {
                (LootType::Scrap(gen_range(1, 4)), 10.0)
            }
            // If roll became less due to difficulty, RareMetal will drop less
            else if roll < 45 {
                (LootType::RareMetal(1), 12.0)
            } else {
                return None;
            }
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
        }
        LootSource::EnemyBoss => {
            // Something always drops from the boss
            (LootType::RareMetal(gen_range(10, 50)), 20.0)
        }
    };

    Some(spawn_loot(pos, item_type, radius))
}

pub fn spawn_loot(pos: Vec2, item_type: LootType, radius: f32) -> LootItem {
    // Random slow drift velocity (super slow, like floating in space)
    let drift_speed = gen_range(5.0, 15.0);
    let drift_angle = gen_range(0.0, std::f32::consts::PI * 2.0);
    let drift_vel = vec2(drift_angle.cos(), drift_angle.sin()) * drift_speed;

    // Random rotation speed (can be positive or negative for random direction)
    let rotation_speed = gen_range(-1.5, 1.5);

    LootItem {
        pos,
        vel: vec2(gen_range(-50.0, 50.0), gen_range(-50.0, 50.0)), // Fly apart on explosion
        drift_vel,
        item_type,
        radius,
        magnet_active: false,
        rotation: gen_range(0.0, std::f32::consts::PI * 2.0), // Random initial rotation
        rotation_speed,
    }
}
