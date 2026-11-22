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

pub(crate) fn advance_ship_movement(
    ship: &mut Ship,
    acceleration: f32,
    input: PilotInput,
    bounds: Vec2,
    dt: f32,
) {
    if input.left {
        ship.rotation -= ROTATION_SPEED * dt;
    }
    if input.right {
        ship.rotation += ROTATION_SPEED * dt;
    }

    let rotation_rad = ship.rotation.to_radians();
    let ship_dir = vec2(rotation_rad.cos(), rotation_rad.sin());

    ship.engine.update(dt, input.thrust);
    if ship.engine.current_thrust > 0.0 {
        let thrust_force = ship.engine.current_thrust * acceleration;
        ship.vel += ship_dir * thrust_force * dt;
    }

    ship.pos += ship.vel * dt;
    wrap_around(&mut ship.pos, bounds);
}

fn fire_player_volley(pilot: &mut Pilot, slot: usize, bullets: &mut Vec<Bullet>) {
    let damage_mult = pilot.damage_mult();
    let dual_cannons = pilot.upgrade_level(UpgradeId::DualCannons) > 0;
    let ship = &mut pilot.ship;
    ship.shoot_timer = if ship.rapid_fire_timer > 0.0 {
        SHOOT_COOLDOWN / 3.0
    } else {
        SHOOT_COOLDOWN
    };
    let forward = Vec2::from_angle(ship.rotation.to_radians());
    let (damage, radius) = if ship.big_bullet_timer > 0.0 {
        (BIG_BULLET_DAMAGE, BIG_BULLET_RADIUS)
    } else {
        (PLAYER_BULLET_DAMAGE, PLAYER_BULLET_RADIUS)
    };
    let dual_mounts = ship.kind.dual_cannon_mounts();
    let mounts = if dual_cannons {
        &dual_mounts[..]
    } else {
        &ShipKind::SINGLE_CANNON[..]
    };

    for mount in mounts {
        let offset = forward.perp() * mount.offset.x - forward * mount.offset.y;
        let heading = if mount.rear_facing { -forward } else { forward };
        bullets.push(Bullet {
            pos: ship.pos + offset * PLAYER_SHIP_DRAW_SIZE,
            vel: heading * BULLET_SPEED + ship.vel,
            owner: Some(slot),
            life_time: BULLET_LIFETIME,
            style: BulletStyle::Player,
            homing: ship.magnetic_ammo_timer > 0.0,
            damage: damage * damage_mult,
            radius,
        });
    }
}

pub(super) fn update_pilots(game: &mut super::Game, inputs: [PilotInput; MAX_PILOTS], dt: f32) {
    for (slot, (pilot, input)) in game.pilots.iter_mut().zip(inputs).enumerate() {
        if let Some(pilot) = pilot.as_mut().filter(|pilot| pilot.is_alive()) {
            let acceleration = pilot.acceleration();
            advance_ship_movement(&mut pilot.ship, acceleration, input, game.bounds, dt);
            if input.fire && pilot.ship.shoot_timer <= 0.0 {
                fire_player_volley(pilot, slot, &mut game.bullets);
                game.audio_cues.push(AudioCue::PlayerShot);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{BuyUpgradeResult, Game};

    #[test]
    fn buying_dual_cannons_changes_the_volley_for_each_ship() {
        let mut game = Game::new(vec2(800.0, 600.0));
        let mut inputs = [PilotInput::default(); MAX_PILOTS];
        inputs[0].fire = true;

        for kind in ShipKind::ALL {
            game.reset(kind, game.bounds);
            let pos = game.solo_pilot().ship.pos;
            let upgrade = UpgradeId::DualCannons;
            assert!(matches!(
                game.solo_pilot_mut().buy_upgrade(upgrade),
                BuyUpgradeResult::NotEnoughResources
            ));
            update_pilots(&mut game, inputs, 0.0);
            assert_eq!(game.bullets.len(), 1);
            assert_eq!(game.bullets[0].pos, pos);
            assert_eq!(game.bullets[0].vel, Vec2::X * BULLET_SPEED);
            game.bullets.clear();
            game.audio_cues.clear();

            let pilot = game.solo_pilot_mut();
            pilot.ship.rare_metal = 100;
            assert!(matches!(
                pilot.buy_upgrade(upgrade),
                BuyUpgradeResult::Bought
            ));
            assert!(matches!(
                pilot.buy_upgrade(upgrade),
                BuyUpgradeResult::Maxed
            ));
            assert_eq!(pilot.ship.rare_metal, 70);
            pilot.ship.shoot_timer = 0.0;
            pilot.ship.rapid_fire_timer = 1.0;
            pilot.ship.big_bullet_timer = 1.0;
            pilot.ship.magnetic_ammo_timer = 1.0;
            pilot.ship.rotation = 90.0;
            pilot.ship.vel = vec2(20.0, 40.0);

            update_pilots(&mut game, inputs, 0.0);
            update_pilots(&mut game, inputs, 0.0);
            assert_eq!(game.bullets.len(), 2);
            assert_eq!(game.audio_cues, [AudioCue::PlayerShot]);
            let ship = &game.solo_pilot().ship;
            assert_eq!(ship.shoot_timer, SHOOT_COOLDOWN / 3.0);
            let first = &game.bullets[0];
            let second = &game.bullets[1];
            assert!((first.vel - ship.vel - Vec2::Y * BULLET_SPEED).length() < 0.001);
            if matches!(kind, ShipKind::Alpha | ShipKind::Specter) {
                assert!((first.vel - second.vel).length() < 0.001);
                assert!(first.pos.x > pos.x && second.pos.x < pos.x);
            } else {
                assert!((second.vel - ship.vel + Vec2::Y * BULLET_SPEED).length() < 0.001);
                assert!(first.pos.y > pos.y && second.pos.y < pos.y);
            }
            assert!(game.bullets.iter().all(|bullet| {
                bullet.owner == Some(0)
                    && bullet.homing
                    && bullet.damage == BIG_BULLET_DAMAGE
                    && bullet.radius == BIG_BULLET_RADIUS
            }));
        }
    }
}
