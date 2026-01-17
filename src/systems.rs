use macroquad::prelude::*;
use std::fs;
use crate::components::SaveData;

pub fn wrap_around(pos: &mut Vec2) {
    if pos.x < -20.0 { pos.x = screen_width() + 20.0; } 
    else if pos.x > screen_width() + 20.0 { pos.x = -20.0; }
    
    if pos.y < -20.0 { pos.y = screen_height() + 20.0; } 
    else if pos.y > screen_height() + 20.0 { pos.y = -20.0; }
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