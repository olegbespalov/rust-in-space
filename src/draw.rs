use crate::components::{EnemyShip, Engine, Ship};
use macroquad::prelude::*;
use macroquad::rand::gen_range;

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

pub fn draw_background(texture: &Texture2D) {
    let screen_w = screen_width();
    let screen_h = screen_height();
    let tex_w = texture.width();
    let tex_h = texture.height();

    // calculate the scale to ensure the texture covers the screen on the smaller side
    let scale_x = screen_w / tex_w;
    let scale_y = screen_h / tex_h;
    // select the maximum scale to avoid black bars
    let scale = scale_x.max(scale_y);

    let final_w = tex_w * scale;
    let final_h = tex_h * scale;

    // center the texture on the screen
    let x = (screen_w - final_w) / 2.0;
    let y = (screen_h - final_h) / 2.0;

    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(final_w, final_h)),
            ..Default::default()
        },
    );
}

pub fn draw_ship(ship: &Ship, body_tex: &Texture2D, flame_tex: &Texture2D) {
    let r_rad = ship.rotation.to_radians();

    draw_engine(&ship.engine, ship.pos, r_rad, flame_tex);

    let ship_size = 72.0;

    draw_texture_ex(
        body_tex,
        ship.pos.x - ship_size / 2.0,
        ship.pos.y - ship_size / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(ship_size, ship_size)),
            rotation: r_rad + std::f32::consts::FRAC_PI_2,
            ..Default::default()
        },
    );
}

pub fn draw_engine(engine: &Engine, ship_pos: Vec2, ship_rotation_rad: f32, texture: &Texture2D) {
    if engine.current_thrust <= 0.05 {
        return;
    }

    let dir_vec = vec2(ship_rotation_rad.cos(), ship_rotation_rad.sin());

    let max_flame_w = 22.0;
    let max_flame_h = 52.0;

    let current_w = max_flame_w * engine.current_thrust;
    let flicker = gen_range(-3.0, 3.0) * engine.current_thrust; // bigger flicker
    let current_h = max_flame_h * engine.current_thrust + flicker;

    let flame_pos = ship_pos - (dir_vec * engine.offset);

    draw_texture_ex(
        texture,
        flame_pos.x - current_w / 2.0,
        flame_pos.y - current_h / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(current_w, current_h)),
            rotation: ship_rotation_rad + std::f32::consts::FRAC_PI_2,
            ..Default::default()
        },
    );
}

pub fn draw_enemy(enemy: &EnemyShip, texture: &Texture2D) {
    let sprite_size = vec2(60.0, 60.0);

    draw_texture_ex(
        texture,
        enemy.pos.x - sprite_size.x / 2.0,
        enemy.pos.y - sprite_size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(sprite_size),
            rotation: 0.0, // Если захочешь вращать врагов: std::f32::consts::PI
            ..Default::default()
        },
    );
}
