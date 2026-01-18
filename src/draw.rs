use crate::components::{Asteroid, EnemyShip, Engine, Explosion, LootItem, LootType, Ship};
use crate::resources::Resources;
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

pub fn draw_ship(
    ship: &Ship,
    body_tex: &Texture2D,
    flame_tex: &Texture2D,
    shield_tex: Option<&Texture2D>,
) {
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

    // Draw shield if active
    if ship.has_shield() {
        if let Some(shield_texture) = shield_tex {
            // Shield is a round energetic circle that covers the ship
            // Make it larger than the ship
            let shield_size = ship_size * 1.8;
            // Calculate opacity based on remaining HP (fade as shield weakens)
            let hp_ratio = ship.shield_hp / ship.shield_max_hp;
            let alpha = (hp_ratio * 0.7 + 0.3).min(1.0); // Between 0.3 and 1.0
            let shield_color = Color::new(1.0, 1.0, 1.0, alpha);

            draw_texture_ex(
                shield_texture,
                ship.pos.x - shield_size / 2.0,
                ship.pos.y - shield_size / 2.0,
                shield_color,
                DrawTextureParams {
                    dest_size: Some(vec2(shield_size, shield_size)),
                    ..Default::default()
                },
            );
        }
    }
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

pub fn draw_enemy(enemy: &EnemyShip, res: &Resources) {
    let size = vec2(60.0, 60.0);
    draw_texture_ex(
        &res.enemy_small, // TODO: different enemy textures
        enemy.pos.x - size.x / 2.0,
        enemy.pos.y - size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            rotation: enemy.rotation + std::f32::consts::FRAC_PI_2,
            ..Default::default()
        },
    );
}

pub fn draw_loot(item: &LootItem, res: &Resources) {
    let texture = match item.item_type {
        LootType::Scrap(_) => &res.loot_scrap,
        LootType::RareMetal(_) => &res.loot_rare,
        LootType::HealthPack(_) => &res.loot_health,
        LootType::RapidFireBoost => &res.loot_rapid_fire,
        LootType::BigBulletBoost => &res.loot_big_bullet,
        LootType::Shield(_) => &res.loot_shield,
    };

    // Increase size for better visibility
    let size = vec2(item.radius * 4.5, item.radius * 4.5);
    draw_texture_ex(
        texture,
        item.pos.x - size.x / 2.0,
        item.pos.y - size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            rotation: item.rotation,
            ..Default::default()
        },
    );
}

pub fn draw_asteroid(asteroid: &Asteroid, res: &Resources) {
    let texture = if asteroid.is_rare {
        &res.rare_asteroid
    } else {
        &res.asteroid
    };

    let size = vec2(asteroid.radius * 2.0, asteroid.radius * 2.0);
    draw_texture_ex(
        texture,
        asteroid.pos.x - size.x / 2.0,
        asteroid.pos.y - size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(size),
            ..Default::default()
        },
    );
}

pub fn draw_explosion(expl: &Explosion, res: &Resources) {
    let texture = &res.explosion;

    // Calculate the width of one frame
    // If you have 8 frames in a row, the frame width = texture width / 8
    let frame_width = texture.width() / expl.max_frames as f32;
    let frame_height = texture.height(); // The height is one

    // Select the needed piece of the texture
    let source_rect = Rect::new(
        expl.frame as f32 * frame_width, // X shift
        0.0,                             // Y is always 0
        frame_width,
        frame_height,
    );

    // Draw
    let draw_size = vec2(frame_width * expl.scale, frame_height * expl.scale);

    draw_texture_ex(
        texture,
        expl.pos.x - draw_size.x / 2.0, // Center
        expl.pos.y - draw_size.y / 2.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(draw_size),
            source: Some(source_rect), // <--- Magic here
            ..Default::default()
        },
    );
}
