use macroquad::prelude::*;
use macroquad::rand::gen_range;
use serde::{Serialize, Deserialize};

#[derive(PartialEq)]
pub enum PowerupType { Health, RapidFire }

pub enum GameState {
    Menu,
    Playing,
    GameOver(u32),
}

pub struct Bullet { pub pos: Vec2, pub vel: Vec2, pub life_time: f32 }
pub struct EnemyBullet { pub pos: Vec2, pub vel: Vec2, pub life_time: f32 }

pub struct Powerup {
    pub pos: Vec2,
    pub p_type: PowerupType,
    pub radius: f32,
}

pub struct Asteroid {
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub sides: u8,
}

pub struct EnemyShip {
    pub pos: Vec2,
    pub vel: Vec2,
    pub shoot_timer: f32,
}

pub struct Ship {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rotation: f32,
    pub lives: i32,
    pub shoot_timer: f32,
    pub rapid_fire_timer: f32,
}

impl Asteroid {
    pub fn new_large() -> Self {
        Self {
            pos: vec2(gen_range(0.0, screen_width()), gen_range(0.0, screen_height())),
            vel: vec2(gen_range(-80.0, 80.0), gen_range(-80.0, 80.0)),
            radius: 40.0,
            sides: gen_range(8, 12),
        }
    }

    pub fn new_fragment(pos: Vec2, radius: f32) -> Self {
        Self {
            pos,
            vel: vec2(gen_range(-120.0, 120.0), gen_range(-120.0, 120.0)),
            radius: radius / 2.0,
            sides: gen_range(5, 8),
        }
    }
}

impl EnemyShip {
    pub fn new() -> Self {
        let side = gen_range(0, 2);
        let x = if side == 0 { -30.0 } else { screen_width() + 30.0 };
        let y = gen_range(50.0, screen_height() - 50.0);
        let speed_x = if side == 0 { 120.0 } else { -120.0 }; // Используй константу или число
        Self {
            pos: vec2(x, y),
            vel: vec2(speed_x, gen_range(-20.0, 20.0)),
            shoot_timer: 1.5,
        }
    }
}

impl Ship {
    // Returns true if the game is over
    pub fn take_damage(&mut self, score: u32) -> bool {
        self.lives -= 1;
        
        if self.lives <= 0 {
            // Save score immediately using our system
            crate::systems::save_score(score);
            return true; // Game Over
        } else {
            // Reset position for next life
            self.pos = vec2(screen_width() / 2.0, screen_height() / 2.0);
            self.vel = vec2(0.0, 0.0);
            return false; // Still alive
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveData {
    pub high_score: u32,
}

impl SaveData {
    pub fn new() -> Self {
        Self { high_score: 0 }
    }
}