use crate::components::*;
use crate::game::constants::*;
use crate::game::Game;
use crate::systems::wrap_around;
use macroquad::prelude::*;

pub fn create_ship() -> Ship {
    Ship {
        pos: vec2(screen_width() / 2.0, screen_height() / 2.0),
        vel: vec2(0.0, 0.0),
        rotation: 0.0,
        health: BASE_SHIP_HP,
        max_health: BASE_SHIP_HP,
        shoot_timer: 0.0,
        rapid_fire_timer: 0.0,
        engine: Engine::basic(),
        scrap: 0,
        rare_metal: 0,
        shield_hp: 0.0,
        shield_max_hp: 0.0,
        shield_timer: 0.0,
        big_bullet_timer: 0.0,
    }
}

pub fn update_ship_movement(game: &mut Game, dt: f32) {
    if is_key_down(KeyCode::Left) {
        game.ship.rotation -= ROTATION_SPEED * dt;
    }
    if is_key_down(KeyCode::Right) {
        game.ship.rotation += ROTATION_SPEED * dt;
    }

    let rotation_rad = game.ship.rotation.to_radians();
    let ship_dir = vec2(rotation_rad.cos(), rotation_rad.sin());

    let is_gas_pedal_down = is_key_down(KeyCode::Up);
    game.ship.engine.update(dt, is_gas_pedal_down);
    if game.ship.engine.current_thrust > 0.0 {
        let thrust_force = game.ship.engine.current_thrust * game.player_acceleration();
        game.ship.vel += ship_dir * thrust_force * dt;
    }

    game.ship.pos += game.ship.vel * dt;
    wrap_around(&mut game.ship.pos);
}

pub fn update_ship_shooting(game: &mut Game) {
    let current_cooldown = if game.ship.rapid_fire_timer > 0.0 {
        SHOOT_COOLDOWN / 3.0
    } else {
        SHOOT_COOLDOWN
    };

    if is_key_down(KeyCode::Space) && game.ship.shoot_timer <= 0.0 {
        let rotation_rad = game.ship.rotation.to_radians();
        let ship_dir = vec2(rotation_rad.cos(), rotation_rad.sin());

        let damage_mult = game.player_damage_mult();
        let (damage, radius) = if game.ship.big_bullet_timer > 0.0 {
            (BIG_BULLET_DAMAGE, BIG_BULLET_RADIUS)
        } else {
            (PLAYER_BULLET_DAMAGE, PLAYER_BULLET_RADIUS)
        };

        game.bullets.push(Bullet {
            pos: game.ship.pos,
            vel: ship_dir * BULLET_SPEED + game.ship.vel,
            life_time: BULLET_LIFETIME,
            style: BulletStyle::Player,
            damage: damage * damage_mult,
            radius,
        });
        game.ship.shoot_timer = current_cooldown;
    }
}
