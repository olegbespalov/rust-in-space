use crate::components::{EnemyShip, Ship};
use macroquad::prelude::*;

pub fn draw_text_centered(text: &str, y_offset: f32, size: u16, color: Color) {
    let dims = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        screen_width() / 2.0 - dims.width / 2.0,
        screen_height() / 2.0 - dims.height / 2.0 + y_offset,
        size as f32,
        color,
    );
}

pub fn draw_ship(ship: &Ship, texture: &Texture2D) {
    let r = ship.rotation.to_radians();
    draw_texture_ex(
        texture,
        ship.pos.x - 20.0,
        ship.pos.y - 20.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(40.0, 40.0)),
            rotation: r + std::f32::consts::FRAC_PI_2, // turn the ship nose up
            ..Default::default()
        },
    );
}

pub fn draw_enemy(enemy: &EnemyShip, texture: &Texture2D) {
    let sprite_size = vec2(40.0, 40.0);

    draw_texture_ex(
        texture,
        enemy.pos.x - sprite_size.x / 2.0,
        enemy.pos.y - sprite_size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(sprite_size),
            rotation: 0.0, // no rotation for now
            ..Default::default()
        },
    );
}
