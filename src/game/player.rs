use crate::audio::AudioCue;
use crate::components::*;
use crate::game::constants::*;
use crate::game::{Pilot, MAX_PILOTS};
use crate::systems::wrap_around;
use macroquad::prelude::*;

#[derive(Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PilotInput {
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub fire: bool,
}

impl PilotInput {
    pub fn keyboard() -> Self {
        Self {
            left: is_key_down(KeyCode::Left),
            right: is_key_down(KeyCode::Right),
            thrust: is_key_down(KeyCode::Up),
            fire: is_key_down(KeyCode::Space),
        }
    }
}

pub fn create_ship(kind: ShipKind, pos: Vec2) -> Ship {
    Ship {
        kind,
        pos,
        vel: vec2(0.0, 0.0),
        rotation: 0.0,
        health: BASE_SHIP_HP,
        max_health: BASE_SHIP_HP,
        shoot_timer: 0.0,
        rapid_fire_timer: 0.0,
        engine: Engine::basic(),
        scrap: 0,
        rare_metal: 0,
        shield_kind: ShieldKind::Standard,
        shield_hp: 0.0,
        shield_max_hp: 0.0,
        shield_timer: 0.0,
        invisibility_timer: 0.0,
        big_bullet_timer: 0.0,
        magnetic_ammo_timer: 0.0,
    }
}

fn update_ship_movement(pilot: &mut Pilot, input: PilotInput, bounds: Vec2, dt: f32) {
    if input.left {
        pilot.ship.rotation -= ROTATION_SPEED * dt;
    }
    if input.right {
        pilot.ship.rotation += ROTATION_SPEED * dt;
    }

    let rotation_rad = pilot.ship.rotation.to_radians();
    let ship_dir = vec2(rotation_rad.cos(), rotation_rad.sin());

    pilot.ship.engine.update(dt, input.thrust);
    if pilot.ship.engine.current_thrust > 0.0 {
        let thrust_force = pilot.ship.engine.current_thrust * pilot.acceleration();
        pilot.ship.vel += ship_dir * thrust_force * dt;
    }

    pilot.ship.pos += pilot.ship.vel * dt;
    wrap_around(&mut pilot.ship.pos, bounds);
}

fn player_shot(pilot: &mut Pilot, slot: usize, input: PilotInput) -> Option<Bullet> {
    let current_cooldown = if pilot.ship.rapid_fire_timer > 0.0 {
        SHOOT_COOLDOWN / 3.0
    } else {
        SHOOT_COOLDOWN
    };

    if input.fire && pilot.ship.shoot_timer <= 0.0 {
        let rotation_rad = pilot.ship.rotation.to_radians();
        let ship_dir = vec2(rotation_rad.cos(), rotation_rad.sin());

        let damage_mult = pilot.damage_mult();
        let (damage, radius) = if pilot.ship.big_bullet_timer > 0.0 {
            (BIG_BULLET_DAMAGE, BIG_BULLET_RADIUS)
        } else {
            (PLAYER_BULLET_DAMAGE, PLAYER_BULLET_RADIUS)
        };

        pilot.ship.shoot_timer = current_cooldown;
        return Some(Bullet {
            pos: pilot.ship.pos,
            vel: ship_dir * BULLET_SPEED + pilot.ship.vel,
            owner: Some(slot),
            life_time: BULLET_LIFETIME,
            style: BulletStyle::Player,
            homing: pilot.ship.magnetic_ammo_timer > 0.0,
            damage: damage * damage_mult,
            radius,
        });
    }
    None
}

pub(super) fn update_pilots(game: &mut super::Game, inputs: [PilotInput; MAX_PILOTS], dt: f32) {
    for (slot, (pilot, input)) in game.pilots.iter_mut().zip(inputs).enumerate() {
        if let Some(pilot) = pilot.as_mut().filter(|pilot| pilot.is_alive()) {
            update_ship_movement(pilot, input, game.bounds, dt);
            if let Some(bullet) = player_shot(pilot, slot, input) {
                game.bullets.push(bullet);
                game.audio_cues.push(AudioCue::PlayerShot);
            }
        }
    }
}
