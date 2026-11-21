use crate::components::Mission;
use crate::components::SaveData;
use crate::components::{Difficulty, LootItem, LootType};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

pub fn save_file() -> Option<PathBuf> {
    let absolute_dir = |name| {
        std::env::var_os(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    let directory = match std::env::consts::OS {
        "linux" => absolute_dir("XDG_DATA_HOME")
            .or_else(|| absolute_dir("HOME").map(|home| home.join(".local/share")))?
            .join("rust-in-space"),
        "macos" => absolute_dir("HOME")?.join("Library/Application Support/Rust in Space"),
        "windows" => absolute_dir("LOCALAPPDATA")?.join("Rust in Space"),
        _ => return None,
    };
    Some(directory.join(SAVE_FILE))
}

pub fn save_data(path: &Path, data: &SaveData) {
    if let Err(error) = write_save(path, data) {
        eprintln!("Failed to write {}: {error}", path.display());
    }
}

fn write_save(path: &Path, data: &SaveData) -> io::Result<()> {
    let json = serde_json::to_vec(data)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Replace the previous snapshot only after the new file is fully written.
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let result = fs::write(&temporary, json).and_then(|()| fs::rename(&temporary, path));
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

pub fn load_save(path: Option<&Path>) -> SaveData {
    path.map_or_else(SaveData::default, |path| {
        load_or_migrate(path, Path::new(SAVE_FILE))
    })
}

fn load_or_migrate(path: &Path, legacy_path: &Path) -> SaveData {
    match fs::read(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let legacy = read_save(legacy_path, fs::read(legacy_path));
            if let Some(data) = legacy {
                save_data(path, &data);
                data
            } else {
                SaveData::default()
            }
        }
        result => read_save(path, result).unwrap_or_default(),
    }
}

fn read_save(path: &Path, contents: io::Result<Vec<u8>>) -> Option<SaveData> {
    match contents.and_then(|bytes| serde_json::from_slice(&bytes).map_err(io::Error::from)) {
        Ok(data) => Some(data),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            eprintln!("Failed to read {}: {error}", path.display());
            None
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::localization::Language;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn migrates_legacy_save_and_restores_updates_from_the_user_directory() {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let directory = std::env::temp_dir().join(format!(
            "rust-in-space-save-{}-{}",
            std::process::id(),
            nonce.as_nanos()
        ));
        fs::create_dir(&directory).unwrap();
        let legacy_path = directory.join(SAVE_FILE);
        let save_path = directory.join("user-data").join(SAVE_FILE);

        let missing = load_or_migrate(&save_path, &legacy_path);
        assert_eq!(missing.high_score, 0);
        assert_eq!(missing.language, None);
        assert!(!save_path.exists());

        let legacy = r#"{"high_score":450,"audio":{"master_volume":0.5,"audio_muted":true}}"#;
        fs::write(&legacy_path, legacy).unwrap();
        let mut data = load_or_migrate(&save_path, &legacy_path);
        assert_eq!(data.high_score, 450);
        assert_eq!(data.language, None);
        assert!(data.audio.audio_muted);
        assert!(!fs::read_to_string(&save_path).unwrap().contains("language"));

        data.high_score = 600;
        data.language = Some(Language::German);
        write_save(&save_path, &data).unwrap();

        let restored = load_or_migrate(&save_path, &legacy_path);
        assert_eq!(restored.high_score, 600);
        assert_eq!(restored.language, Some(Language::German));
        assert_eq!(restored.audio.master_volume, 0.5);
        assert!(restored.audio.audio_muted);
        assert_eq!(fs::read_to_string(&legacy_path).unwrap(), legacy);

        fs::write(&save_path, b"invalid JSON").unwrap();
        let invalid = load_or_migrate(&save_path, &legacy_path);
        assert_eq!(invalid.high_score, 0);
        assert_eq!(invalid.language, None);

        fs::remove_file(&save_path).unwrap();
        fs::create_dir(&save_path).unwrap();
        let unreadable = load_or_migrate(&save_path, &legacy_path);
        assert_eq!(unreadable.high_score, 0);
        assert_eq!(unreadable.language, None);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn unknown_language_keeps_the_saved_score_and_defaults_to_english() {
        let data: SaveData =
            serde_json::from_str(r#"{"high_score":900,"language":"unknown"}"#).unwrap();
        assert_eq!(data.high_score, 900);
        assert_eq!(data.language, Some(Language::English));
    }
}
