use crate::audio::AudioCue;
use crate::components::*;
use crate::game::constants::*;
use crate::game::{nearest_pilot, Game, Pilot};
use crate::systems::{generate_loot, spawn_loot, spawn_shield_loot, wrap_around, LootSource};
use macroquad::prelude::*;
use macroquad::rand::gen_range;

const ASTEROID_MISSION_POINTS: f32 = 10.0;

pub fn update_timers(game: &mut Game, dt: f32) {
    game.mission_progress.elapsed_seconds += f64::from(dt);
    for pilot in game
        .pilots
        .iter_mut()
        .flatten()
        .filter(|pilot| pilot.is_alive())
    {
        let ship = &mut pilot.ship;
        ship.shoot_timer -= dt;
        ship.rapid_fire_timer -= dt;
        ship.big_bullet_timer -= dt;
        ship.magnetic_ammo_timer -= dt;
        ship.invisibility_timer = (ship.invisibility_timer - dt).max(0.0);

        if ship.shield_timer > 0.0 {
            ship.shield_timer -= dt;
            if ship.shield_timer <= 0.0 {
                ship.shield_hp = 0.0;
            }
        }
    }

    game.explosions.retain_mut(|e| {
        e.timer += dt;
        if e.timer >= e.frame_time {
            e.timer = 0.0;
            e.frame += 1;
        }
        e.frame < e.max_frames
    });
}

fn homing_velocity(
    pos: Vec2,
    vel: Vec2,
    targets: impl Iterator<Item = Vec2>,
    max_turn: f32,
) -> Vec2 {
    let heading = vel.normalize_or_zero();
    let cone_dot = HOMING_HALF_ANGLE.to_radians().cos();
    let target = targets
        .map(|target| target - pos)
        .filter(|offset| {
            offset.length_squared() <= HOMING_RANGE * HOMING_RANGE
                && heading.dot(offset.normalize_or_zero()) >= cone_dot
        })
        .min_by(|a, b| a.length_squared().total_cmp(&b.length_squared()));
    let Some(offset) = target else {
        return vel;
    };

    let angle = heading.perp_dot(offset).atan2(heading.dot(offset));
    let (sin, cos) = angle.clamp(-max_turn, max_turn).sin_cos();
    vec2(vel.x * cos - vel.y * sin, vel.x * sin + vel.y * cos)
}

pub fn update_physics(game: &mut Game, dt: f32) {
    for b in &mut game.bullets {
        if b.homing {
            match b.style {
                BulletStyle::Player => {
                    let targets = game
                        .enemy_ships
                        .iter()
                        .filter(|enemy| !enemy.power_ups.is_invisible())
                        .map(|enemy| enemy.pos)
                        .chain(
                            game.boss
                                .iter()
                                .filter(|boss| !boss.power_ups.is_invisible())
                                .map(|boss| boss.pos),
                        );
                    b.vel = homing_velocity(b.pos, b.vel, targets, HOMING_TURN_SPEED * dt);
                }
                BulletStyle::Enemy | BulletStyle::Boss | BulletStyle::BossMissile => {
                    let targets = game
                        .pilots
                        .iter()
                        .flatten()
                        .filter(|pilot| pilot.is_alive() && !pilot.ship.is_invisible())
                        .map(|pilot| pilot.ship.pos);
                    let turn_speed = if b.style == BulletStyle::BossMissile {
                        BOSS_MISSILE_TURN_SPEED
                    } else {
                        HOMING_TURN_SPEED
                    };
                    b.vel = homing_velocity(b.pos, b.vel, targets, turn_speed * dt);
                }
            }
        }
        b.pos += b.vel * dt;
        b.life_time -= dt;
    }
    game.bullets.retain(|b| b.life_time > 0.0);

    for a in game.asteroids.iter_mut() {
        a.pos += a.vel * dt;
        wrap_around(&mut a.pos, game.bounds);
    }

    // Ramming can exhaust the field without earning credit for the objective.
    if game.asteroids.is_empty()
        && game.current_mission.objectives.iter().any(|&objective| {
            matches!(objective, MissionObjective::DestroyAsteroids(_))
                && !game.mission_progress.meets(objective)
        })
    {
        let mut asteroid = Asteroid::new_large(game.bounds);
        if game
            .pilots
            .iter()
            .flatten()
            .find(|pilot| pilot.is_alive())
            .is_some_and(|pilot| pilot.ship.pos.x < game.bounds.x / 2.0)
        {
            asteroid.pos.x = game.bounds.x;
            asteroid.vel.x = -asteroid.vel.x.abs();
        } else {
            asteroid.pos.x = 0.0;
            asteroid.vel.x = asteroid.vel.x.abs();
        }
        game.asteroids.push(asteroid);
    }
}

pub fn update_loot(game: &mut Game, dt: f32) {
    let scrap_target = game
        .current_mission
        .objectives
        .iter()
        .find_map(|objective| match objective {
            MissionObjective::CollectScrap(target) => Some(*target),
            _ => None,
        })
        .unwrap_or_default();
    game.loot_items.retain_mut(|item| {
        if !game.pilots.iter().flatten().any(super::Pilot::is_alive) {
            return true;
        }
        item.time_left -= dt;
        if item.time_left <= 0.0 {
            return false;
        }

        item.vel *= LOOT_VEL_DECAY;
        item.pos += item.vel * dt;
        item.pos += item.drift_vel * dt;
        wrap_around(&mut item.pos, game.bounds);

        item.rotation += item.rotation_speed * dt;
        if item.rotation > std::f32::consts::PI * 2.0 {
            item.rotation -= std::f32::consts::PI * 2.0;
        } else if item.rotation < 0.0 {
            item.rotation += std::f32::consts::PI * 2.0;
        }

        let pickup_pos = item.pos;
        if let Some(pilot) = nearest_pilot(&game.pilots, item.pos, |pilot| {
            item.magnet_active
                || pilot.ship.pos.distance_squared(item.pos) < pilot.loot_magnet_radius().powi(2)
        })
        .and_then(|slot| game.pilots[slot].as_ref())
        {
            item.magnet_active = true;
            let dir = (pilot.ship.pos - item.pos).normalize_or_zero();
            item.pos += dir * LOOT_MAGNET_SPEED * dt;
        }
        // Pickup tests use the position before magnet motion, as in solo play.
        if let Some(pilot) = nearest_pilot(&game.pilots, pickup_pos, |pilot| {
            pilot.ship.pos.distance_squared(pickup_pos) < (SHIP_RADIUS + item.radius).powi(2)
        })
        .and_then(|slot| game.pilots[slot].as_mut())
        {
            apply_pickup(
                pilot,
                &item.item_type,
                &mut game.mission_progress,
                scrap_target,
                &mut game.audio_cues,
            );
            return false;
        }
        !enemy_picks_up(item, &mut game.enemy_ships, game.boss.as_mut())
    });
    // A mystery box can destroy its collector. Resolve those deaths before collisions.
    game.enemy_ships.retain(|enemy| {
        if enemy.health > 0.0 {
            return true;
        }
        game.mission_progress.record_enemy_defeat(enemy.enemy_type);
        game.explosions.push(Explosion::new(enemy.pos, 0.4));
        game.audio_cues.push(AudioCue::ExplosionBig);
        false
    });
    if let Some(boss) = &game.boss {
        if boss.health <= 0.0 {
            finish_boss_defeat(game, boss.pos, boss.max_health);
        }
    }
}

fn apply_pickup(
    pilot: &mut Pilot,
    kind: &LootType,
    progress: &mut super::MissionProgress,
    scrap_target: u32,
    cues: &mut Vec<AudioCue>,
) {
    let ship = &mut pilot.ship;
    match *kind {
        LootType::Scrap(amount) | LootType::MediumScrap(amount) | LootType::LargeScrap(amount) => {
            let credited = amount.min(scrap_target.saturating_sub(progress.scrap_collected));
            pilot.mission_points += credited as f32;
            ship.scrap += amount;
            progress.scrap_collected += amount;
            cues.push(AudioCue::PickupScrap);
        }
        LootType::RareMetal(amount) => {
            ship.rare_metal += amount;
            cues.push(AudioCue::PickupScrap);
        }
        LootType::HealthPack(hp) => {
            ship.heal(hp as f32);
            cues.push(AudioCue::PickupHealth);
        }
        LootType::RapidFireBoost => {
            ship.rapid_fire_timer = RAPID_FIRE_DURATION;
            cues.push(AudioCue::PickupScrap);
        }
        LootType::BigBulletBoost => {
            ship.big_bullet_timer = BIG_BULLET_DURATION;
            cues.push(AudioCue::PickupScrap);
        }
        LootType::MagneticAmmo => {
            ship.magnetic_ammo_timer = MAGNETIC_AMMO_DURATION;
            cues.push(AudioCue::PickupScrap);
        }
        LootType::Invisibility => {
            ship.invisibility_timer = INVISIBILITY_DURATION;
            cues.push(AudioCue::PickupScrap);
        }
        LootType::Shield(kind, hp) => {
            ship.activate_shield(kind, hp as f32, kind.duration());
            cues.push(AudioCue::ShieldOn);
        }
        LootType::MysteryBox => cues.push(open_mystery_box(ship, gen_range(0, 4))),
    }
}

fn enemy_picks_up(item: &LootItem, enemies: &mut [EnemyShip], boss: Option<&mut Boss>) -> bool {
    if let Some(enemy) = enemies.iter_mut().find(|enemy| {
        enemy.health > 0.0
            && item.pos.distance(enemy.pos)
                < item.radius + enemy.enemy_type.projectile_hitbox_radius()
    }) {
        enemy
            .power_ups
            .pick_up(&item.item_type, &mut enemy.health, enemy.max_health);
        return true;
    }

    if let Some(boss) = boss {
        if boss.health > 0.0 && item.pos.distance(boss.pos) < item.radius + BOSS_HITBOX_RADIUS {
            boss.power_ups
                .pick_up(&item.item_type, &mut boss.health, boss.max_health);
            return true;
        }
    }

    false
}

fn open_mystery_box(ship: &mut Ship, roll: i32) -> AudioCue {
    match MysteryEffect::from_roll(roll) {
        MysteryEffect::Repair(hp) => {
            ship.heal(hp);
            AudioCue::PickupHealth
        }
        MysteryEffect::Gold(amount) => {
            ship.rare_metal += amount;
            AudioCue::PickupScrap
        }
        MysteryEffect::Shield(hp) => {
            let (kind, duration) = if ship.has_shield() {
                (ship.shield_kind, ship.shield_timer.max(SHIELD_DURATION))
            } else {
                (ShieldKind::Standard, SHIELD_DURATION)
            };
            if ship.shield_hp < hp {
                ship.activate_shield(kind, hp, duration);
            } else {
                ship.shield_timer = duration;
            }
            AudioCue::ShieldOn
        }
        MysteryEffect::Trap(damage) => {
            let cue = if ship.has_shield() {
                AudioCue::ShieldHit
            } else {
                AudioCue::ShipHit
            };
            ship.take_damage(damage);
            cue
        }
    }
}

pub fn update_collisions(game: &mut Game, dt: f32) {
    handle_bullet_bullet_collisions(game);
    handle_player_bullet_collisions(game, dt);
    handle_enemy_bullet_collisions(game, dt);
    handle_ship_entity_collisions(game);
}

fn handle_bullet_bullet_collisions(game: &mut Game) {
    if !game.bullets.iter().any(|b| b.style == BulletStyle::Player) {
        return;
    }
    let mut hostile_bullets: Vec<_> = game
        .bullets
        .iter()
        .enumerate()
        .filter(|(_, bullet)| bullet.style != BulletStyle::Player)
        .map(|(index, bullet)| (index, Some(bullet)))
        .collect();
    if hostile_bullets.is_empty() {
        return;
    }

    let mut intercepted = Vec::new();
    for (player_index, player) in game.bullets.iter().enumerate() {
        if player.style != BulletStyle::Player {
            continue;
        }
        for (hostile_index, hostile) in &mut hostile_bullets {
            let Some(bullet) = *hostile else {
                continue;
            };
            let radius = player.radius + bullet.radius;
            if player.pos.distance_squared(bullet.pos) < radius * radius {
                let collision_pos = (player.pos + bullet.pos) * 0.5;
                game.explosions.push(Explosion::new(collision_pos, 0.5));
                game.audio_cues.push(AudioCue::ExplosionSmall);
                *hostile = None;
                if intercepted.is_empty() {
                    intercepted.resize(game.bullets.len(), false);
                }
                intercepted[player_index] = true;
                intercepted[*hostile_index] = true;
                break;
            }
        }
    }

    if !intercepted.is_empty() {
        let mut index = 0;
        game.bullets.retain(|_| {
            let keep = !intercepted[index];
            index += 1;
            keep
        });
    }
}

fn drop_small_enemy_loot(items: &mut Vec<LootItem>, pos: Vec2, difficulty: Difficulty) {
    items.extend(generate_loot(pos, LootSource::EnemySmall, difficulty));
    if gen_range(0, 100) < MAGNETIC_AMMO_DROP_PERCENT {
        items.push(spawn_loot(pos, LootType::MagneticAmmo, 15.0));
    }
    if gen_range(0, 100) < INVISIBILITY_DROP_PERCENT {
        items.push(spawn_loot(pos, LootType::Invisibility, 15.0));
    }
    if gen_range(0, 100) < MYSTERY_BOX_DROP_PERCENT {
        items.push(spawn_loot(pos, LootType::MysteryBox, 15.0));
    }
    if gen_range(0, 100) < GOLDEN_SHIELD_DROP_PERCENT {
        items.push(spawn_shield_loot(pos, ShieldKind::Golden));
    }
}

fn handle_player_bullet_collisions(game: &mut Game, dt: f32) {
    let mut new_asteroids = Vec::new();
    let mut boss_died_at: Option<(Vec2, f32)> = None;

    game.bullets.retain(|b| {
        if b.style != BulletStyle::Player {
            return true;
        }
        let mut hit = false;
        let mut points = 0.0;

        // Asteroids
        for i in (0..game.asteroids.len()).rev() {
            let asteroid = &game.asteroids[i];
            let radius = asteroid.radius + b.radius;
            if b.pos.distance_squared(asteroid.pos) < radius * radius {
                game.score += 100;
                points += ASTEROID_MISSION_POINTS;
                let asteroid = game.asteroids.remove(i);
                game.mission_progress.asteroids_destroyed += 1;
                game.audio_cues.push(crate::audio::AudioCue::ExplosionSmall);

                let source = if asteroid.is_rare {
                    LootSource::RareAsteroid
                } else {
                    LootSource::Asteroid
                };
                if let Some(loot) = generate_loot(asteroid.pos, source, game.difficulty) {
                    game.loot_items.push(loot);
                }

                if asteroid.radius > MIN_ASTEROID_FOR_FRAGMENTATION {
                    new_asteroids.push(Asteroid::new_fragment(asteroid.pos, asteroid.radius));
                    new_asteroids.push(Asteroid::new_fragment(asteroid.pos, asteroid.radius));
                }
                hit = true;
                break;
            }
        }

        // Enemy ships
        if !hit {
            let target = game
                .enemy_ships
                .iter()
                .enumerate()
                .find_map(|(index, enemy)| {
                    let shield_size = enemy
                        .power_ups
                        .shield
                        .as_ref()
                        .map(|_| enemy.enemy_type.draw_size());
                    projectile_impact(
                        b,
                        dt,
                        enemy.pos,
                        enemy.enemy_type.projectile_hitbox_radius(),
                        shield_size,
                    )
                    .map(|impact| (index, impact))
                });
            if let Some((index, impact)) = target {
                hit = true;
                let enemy = &mut game.enemy_ships[index];
                if enemy.power_ups.shield.is_some() {
                    game.explosions.push(Explosion::new(impact, 0.5));
                    game.audio_cues.push(AudioCue::ShieldHit);
                }
                points += effective_damage(b.damage, enemy.health, &enemy.power_ups);
                if enemy.take_damage(b.damage) {
                    let enemy = game.enemy_ships.remove(index);
                    game.score += (enemy.max_health as u32) * SCORE_PER_ENEMY_HP;
                    drop_small_enemy_loot(&mut game.loot_items, enemy.pos, game.difficulty);
                    game.mission_progress.record_enemy_defeat(enemy.enemy_type);
                    game.explosions.push(Explosion::new(enemy.pos, 0.4));
                    game.audio_cues.push(AudioCue::ExplosionBig);
                }
            }
        }

        // Boss
        if !hit {
            if let Some(ref mut boss) = game.boss {
                let shielded = boss.power_ups.shield.is_some();
                if let Some(impact) = projectile_impact(
                    b,
                    dt,
                    boss.pos,
                    BOSS_HITBOX_RADIUS,
                    shielded.then_some(BOSS_SIZE),
                ) {
                    hit = true;
                    if shielded {
                        game.explosions.push(Explosion::new(impact, 0.5));
                        game.audio_cues.push(AudioCue::ShieldHit);
                    }
                    points += effective_damage(b.damage, boss.health, &boss.power_ups);
                    let pos = boss.pos;
                    let max_health = boss.max_health;
                    if boss.take_damage(b.damage) {
                        boss_died_at = Some((pos, max_health));
                    }
                }
            }
        }

        if let Some(pilot) = b.owner.and_then(|slot| game.pilots[slot].as_mut()) {
            pilot.mission_points += points;
        }
        !hit
    });

    if let Some((pos, max_health)) = boss_died_at {
        finish_boss_defeat(game, pos, max_health);
    }
    game.asteroids.extend(new_asteroids);
}

fn effective_damage(damage: f32, health: f32, power_ups: &EnemyPowerUps) -> f32 {
    damage.min(health.max(0.0) + power_ups.shield.as_ref().map_or(0.0, |shield| shield.hp))
}

fn finish_boss_defeat(game: &mut Game, pos: Vec2, max_health: f32) {
    game.boss = None;
    game.mission_progress.boss_defeated = true;
    game.score += (max_health as u32) * SCORE_PER_ENEMY_HP;
    game.explosions.push(Explosion::new(pos, 0.8));
    game.audio_cues.push(AudioCue::ExplosionBig);
    if let Some(loot) = generate_loot(pos, LootSource::EnemyBoss, game.difficulty) {
        game.loot_items.push(loot);
    }
}

fn collision_pilot(
    pilots: &mut [Option<super::Pilot>; super::MAX_PILOTS],
    pos: Vec2,
    radius: f32,
) -> Option<&mut Ship> {
    let hit = |pilot: &super::Pilot| {
        pilot.ship.pos.distance_squared(pos) < (SHIP_RADIUS + radius).powi(2)
    };
    // A solo frame still resolves every collision before reporting defeat.
    if pilots[1..].iter().all(Option::is_none) {
        return pilots[0]
            .as_mut()
            .filter(|pilot| hit(pilot))
            .map(|pilot| &mut pilot.ship);
    }
    let slot = nearest_pilot(pilots, pos, hit)?;
    pilots[slot].as_mut().map(|pilot| &mut pilot.ship)
}

fn projectile_impact(
    bullet: &Bullet,
    dt: f32,
    center: Vec2,
    hull_radius: f32,
    shield_hull_size: Option<f32>,
) -> Option<Vec2> {
    let Some(hull_size) = shield_hull_size else {
        return (bullet.pos.distance_squared(center) < (hull_radius + bullet.radius).powi(2))
            .then_some(center);
    };
    let shield_radius = hull_size * SHIELD_DRAW_SCALE * SHIELD_RING_FRACTION / 2.0;
    // Use the same velocity as the movement step, including any homing turn.
    let travel = bullet.vel * dt;
    let start = bullet.pos - center - travel;
    let c = start.length_squared() - (shield_radius + bullet.radius).powi(2);
    let contact = if c <= 0.0 {
        start
    } else {
        let a = travel.length_squared();
        let b = start.dot(travel);
        let discriminant = b * b - a * c;
        if a == 0.0 || b >= 0.0 || discriminant < 0.0 {
            return None;
        }
        let t = (-b - discriminant.sqrt()) / a;
        if t > 1.0 {
            return None;
        }
        start + travel * t
    };
    Some(center + contact.normalize_or(-bullet.vel.normalize_or(Vec2::X)) * shield_radius)
}

fn collision_pilot_projectile<'a>(
    pilots: &'a mut [Option<Pilot>; super::MAX_PILOTS],
    bullet: &Bullet,
    dt: f32,
) -> Option<(&'a mut Ship, Vec2)> {
    let impact = |pilot: &Pilot| {
        projectile_impact(
            bullet,
            dt,
            pilot.ship.pos,
            SHIP_RADIUS,
            pilot.ship.has_shield().then_some(PLAYER_SHIP_DRAW_SIZE),
        )
    };
    let slot = if pilots[1..].iter().all(Option::is_none) {
        0
    } else {
        nearest_pilot(pilots, bullet.pos, |pilot| impact(pilot).is_some())?
    };
    let pilot = pilots[slot].as_mut()?;
    let impact = impact(pilot)?;
    Some((&mut pilot.ship, impact))
}

fn damage_ship(ship: &mut Ship, damage: f32, cues: &mut Vec<AudioCue>) {
    cues.push(if ship.has_shield() {
        AudioCue::ShieldHit
    } else {
        AudioCue::ShipHit
    });
    ship.take_damage(damage);
}

fn handle_enemy_bullet_collisions(game: &mut Game, dt: f32) {
    game.bullets.retain(|bullet| {
        if bullet.style != BulletStyle::Player {
            if let Some((ship, impact)) = collision_pilot_projectile(&mut game.pilots, bullet, dt) {
                game.explosions.push(Explosion::new(impact, 0.5));
                damage_ship(
                    ship,
                    bullet.damage * game.difficulty.damage_mult(),
                    &mut game.audio_cues,
                );
                return false;
            }
        }
        true
    });
}

fn handle_ship_entity_collisions(game: &mut Game) {
    for i in (0..game.asteroids.len()).rev() {
        let asteroid = &game.asteroids[i];
        if let Some(ship) = collision_pilot(&mut game.pilots, asteroid.pos, asteroid.radius) {
            let asteroid = game.asteroids.remove(i);
            let damage =
                asteroid.radius / 10.0 * BASE_ASTEROID_DAMAGE * game.difficulty.damage_mult();
            game.explosions.push(Explosion::new(
                ship.pos,
                (asteroid.radius / LARGE_ASTEROID_RADIUS).clamp(0.3, 0.8),
            ));
            damage_ship(ship, damage, &mut game.audio_cues);
        }
    }

    game.enemy_ships.retain_mut(|enemy| {
        if enemy.enemy_type == EnemyType::Kamikaze {
            if let Some(ship) = collision_pilot(&mut game.pilots, enemy.pos, KAMIKAZE_RADIUS) {
                let damage = BASE_KAMIKAZE_DAMAGE * game.difficulty.damage_mult();
                game.explosions.push(Explosion::new(enemy.pos, 0.6));
                damage_ship(ship, damage, &mut game.audio_cues);
                game.score += (enemy.max_health as u32) * SCORE_PER_ENEMY_HP;
                drop_small_enemy_loot(&mut game.loot_items, enemy.pos, game.difficulty);
                game.mission_progress.record_enemy_defeat(enemy.enemy_type);
                return false;
            }
        }
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_pilots_collect_once_with_slot_order_breaking_a_tie() {
        let mut game = Game::new(vec2(1280.0, 720.0));
        game.set_pilot(1, ShipKind::Delta);
        game.current_mission.objectives = vec![MissionObjective::CollectScrap(10)];
        let pos = game.solo_pilot().ship.pos;
        game.loot_items
            .push(spawn_loot(pos, LootType::Scrap(15), 15.0));
        update_loot(&mut game, 0.0);
        assert!(game.loot_items.is_empty());
        assert_eq!(game.solo_pilot().ship.scrap, 15);
        assert_eq!(game.pilots[1].as_ref().unwrap().ship.scrap, 0);
        assert_eq!(game.mission_progress.scrap_collected, 15);
        assert_eq!(game.solo_pilot().mission_points(), 10);
        assert_eq!(game.pilots[1].as_ref().unwrap().mission_points(), 0);
    }

    #[test]
    fn enemy_damage_credits_both_pilots_and_caps_overkill() {
        let mut game = Game::new(vec2(1280.0, 720.0));
        game.set_pilot(1, ShipKind::Delta);
        let mut enemy = EnemyShip::new_at_side(true, 100.0, EnemyType::Regular, game.bounds.x);
        enemy.health = 7.0;
        enemy
            .power_ups
            .activate_shield(ShieldKind::Standard, 12.0, 5.0);
        let pos = enemy.pos;
        game.enemy_ships.push(enemy);

        for (owner, damage) in [(0, 12.0), (1, 25.0)] {
            game.bullets.push(Bullet {
                pos,
                vel: Vec2::ZERO,
                owner: Some(owner),
                life_time: 2.0,
                style: BulletStyle::Player,
                homing: false,
                damage,
                radius: PLAYER_BULLET_RADIUS,
            });
        }
        update_collisions(&mut game, 0.0);

        assert!(game.enemy_ships.is_empty());
        assert_eq!(game.solo_pilot().mission_points(), 12);
        assert_eq!(game.pilots[1].as_ref().unwrap().mission_points(), 7);
    }

    #[test]
    fn player_projectile_hits_first_enemy_and_preserves_survivor_order() {
        let mut game = Game::new(vec2(1280.0, 720.0));
        let pos = vec2(350.0, 100.0);
        for kind in [EnemyType::Regular, EnemyType::Kamikaze, EnemyType::Heavy] {
            let mut enemy = EnemyShip::new_at_side(true, pos.y, kind, game.bounds.x);
            enemy.pos = pos;
            enemy.health = 1.0;
            game.enemy_ships.push(enemy);
        }
        game.bullets.push(Bullet {
            pos,
            vel: Vec2::ZERO,
            owner: Some(0),
            life_time: 1.0,
            style: BulletStyle::Player,
            homing: false,
            damage: PLAYER_BULLET_DAMAGE,
            radius: PLAYER_BULLET_RADIUS,
        });

        handle_player_bullet_collisions(&mut game, 0.0);

        assert!(game.bullets.is_empty());
        assert_eq!(game.enemy_ships.len(), 2);
        assert!(matches!(
            game.enemy_ships[0].enemy_type,
            EnemyType::Kamikaze
        ));
        assert!(matches!(game.enemy_ships[1].enemy_type, EnemyType::Heavy));
        assert!(game.enemy_ships.iter().all(|enemy| enemy.health == 1.0));
    }

    #[test]
    fn enemy_projectile_hits_only_the_nearest_surviving_pilot() {
        let mut game = Game::new(vec2(1280.0, 720.0));
        game.set_pilot(1, ShipKind::Delta);
        let pos = game.solo_pilot().ship.pos + vec2(10.0, 0.0);
        game.pilots[1].as_mut().unwrap().ship.pos = pos;
        game.bullets.push(Bullet {
            pos,
            vel: Vec2::ZERO,
            owner: None,
            life_time: 2.0,
            style: BulletStyle::Enemy,
            homing: false,
            damage: ENEMY_BULLET_DAMAGE,
            radius: 9.0,
        });
        update_collisions(&mut game, 0.0);
        assert!(game.bullets.is_empty());
        assert_eq!(game.solo_pilot().ship.health, BASE_SHIP_HP);
        assert_eq!(
            game.pilots[1].as_ref().unwrap().ship.health,
            BASE_SHIP_HP - ENEMY_BULLET_DAMAGE
        );
    }

    #[test]
    fn enemy_projectile_hits_active_shield_before_ship_hull() {
        for style in [
            BulletStyle::Enemy,
            BulletStyle::Boss,
            BulletStyle::BossMissile,
        ] {
            let mut game = Game::new(vec2(1280.0, 720.0));
            let ship = &mut game.solo_pilot_mut().ship;
            ship.activate_shield(ShieldKind::Standard, 30.0, 5.0);
            let center = ship.pos;
            game.bullets.push(Bullet {
                pos: center + vec2(100.0, 0.0),
                vel: vec2(-200.0, 0.0),
                owner: None,
                life_time: 2.0,
                style,
                homing: false,
                damage: ENEMY_BULLET_DAMAGE,
                radius: 9.0,
            });

            update_physics(&mut game, 1.0);
            update_collisions(&mut game, 1.0);

            assert!(game.bullets.is_empty());
            assert_eq!(game.solo_pilot().ship.health, BASE_SHIP_HP);
            assert_eq!(game.solo_pilot().ship.shield_hp, 30.0 - ENEMY_BULLET_DAMAGE);
            assert_eq!(game.audio_cues, [AudioCue::ShieldHit]);
            assert_eq!(game.explosions.len(), 1);
            assert!((game.explosions[0].pos.x - center.x - 76.0).abs() < 1.0);
            assert_eq!(game.explosions[0].pos.y, center.y);
        }
    }

    #[test]
    fn player_projectiles_hit_enemy_and_boss_shield_rims() {
        let mut game = Game::new(vec2(1280.0, 720.0));
        game.asteroids.clear();
        let mut enemy = EnemyShip::new_at_side(true, 100.0, EnemyType::Guardian, game.bounds.x);
        enemy.pos = vec2(350.0, 100.0);
        let mut boss = Boss::new(vec2(350.0, 550.0), 0.0);
        for power_ups in [&mut enemy.power_ups, &mut boss.power_ups] {
            power_ups.activate_shield(ShieldKind::Golden, 30.0, 5.0);
        }
        let targets = [(enemy.pos, 93.0), (boss.pos, 203.0)];
        game.enemy_ships.push(enemy);
        game.boss = Some(boss);

        for (center, rim) in targets {
            game.bullets.push(Bullet {
                pos: center - vec2(rim + 15.0, 0.0),
                vel: vec2(30.0, 0.0),
                owner: Some(0),
                life_time: 2.0,
                style: BulletStyle::Player,
                homing: false,
                damage: PLAYER_BULLET_DAMAGE,
                radius: PLAYER_BULLET_RADIUS,
            });
        }

        update_physics(&mut game, 1.0);
        update_collisions(&mut game, 1.0);

        assert!(game.bullets.is_empty());
        let enemy = &game.enemy_ships[0];
        let boss = game.boss.as_ref().unwrap();
        assert_eq!(enemy.health, enemy.max_health);
        assert_eq!(boss.health, boss.max_health);
        for power_ups in [&enemy.power_ups, &boss.power_ups] {
            assert_eq!(
                power_ups.shield.as_ref().unwrap().hp,
                30.0 - PLAYER_BULLET_DAMAGE
            );
        }
        assert_eq!(game.explosions.len(), 2);
        for (explosion, (center, rim)) in game.explosions.iter().zip(targets) {
            assert!((explosion.pos.x - center.x + rim).abs() < 1.0);
            assert_eq!(explosion.pos.y, center.y);
        }
    }

    #[test]
    fn projectile_interceptions_preserve_hit_and_survivor_order() {
        let mut game = Game::new(vec2(1280.0, 720.0));
        let bullet = |x, style, owner| Bullet {
            pos: vec2(x, 100.0),
            vel: Vec2::ZERO,
            owner,
            life_time: 1.0,
            style,
            homing: false,
            damage: PLAYER_BULLET_DAMAGE,
            radius: PLAYER_BULLET_RADIUS,
        };
        game.bullets = vec![
            bullet(108.0, BulletStyle::Enemy, None),
            bullet(100.0, BulletStyle::Player, Some(0)),
            bullet(200.0, BulletStyle::Boss, None),
            bullet(100.0, BulletStyle::Player, Some(1)),
            bullet(100.0, BulletStyle::BossMissile, None),
            bullet(100.0, BulletStyle::Player, Some(2)),
            bullet(300.0, BulletStyle::Enemy, None),
        ];

        handle_bullet_bullet_collisions(&mut game);

        let explosion_positions: Vec<_> = game.explosions.iter().map(|ex| ex.pos.x).collect();
        assert_eq!(explosion_positions, [104.0, 100.0]);
        assert_eq!(game.audio_cues.len(), 2);
        let survivors: Vec<_> = game.bullets.iter().map(|b| (b.pos.x, b.owner)).collect();
        assert_eq!(survivors, [(200.0, None), (100.0, Some(2)), (300.0, None)]);
    }

    #[test]
    fn a_missile_tracks_the_pilot_and_can_be_intercepted_by_a_later_shot() {
        let mut game = Game::new(vec2(1280.0, 720.0));
        let pos = game.solo_pilot().ship.pos - vec2(180.0, 80.0);
        game.bullets.push(Bullet {
            pos,
            vel: Vec2::X * BOSS_MISSILE_SPEED,
            owner: None,
            life_time: 3.0,
            style: BulletStyle::BossMissile,
            homing: true,
            damage: BOSS_MISSILE_DAMAGE,
            radius: 11.25,
        });
        update_physics(&mut game, 0.1);
        let missile = &game.bullets[0];
        assert!(missile.vel.y > 0.0);
        assert!((missile.vel.y.atan2(missile.vel.x) - BOSS_MISSILE_TURN_SPEED * 0.1).abs() < 1e-5);

        game.bullets.push(Bullet {
            pos: missile.pos,
            vel: Vec2::ZERO,
            owner: Some(0),
            life_time: 1.0,
            style: BulletStyle::Player,
            homing: false,
            damage: PLAYER_BULLET_DAMAGE,
            radius: PLAYER_BULLET_RADIUS,
        });
        update_collisions(&mut game, 0.0);
        assert!(game.bullets.is_empty());
        assert_eq!(game.solo_pilot().ship.health, BASE_SHIP_HP);
    }

    #[test]
    fn enemy_can_collect_a_shield_and_it_absorbs_damage_until_expiry() {
        let mut enemies = [EnemyShip {
            pos: vec2(100.0, 100.0),
            vel: Vec2::ZERO,
            shoot_timer: 0.0,
            rotation: 0.0,
            health: 24.0,
            max_health: 24.0,
            enemy_type: EnemyType::Regular,
            power_ups: EnemyPowerUps::default(),
        }];
        let mut item = spawn_loot(Vec2::ZERO, LootType::Shield(ShieldKind::Golden, 40), 15.0);
        assert!(!enemy_picks_up(&item, &mut enemies, None));

        item.pos = enemies[0].pos;
        assert!(enemy_picks_up(&item, &mut enemies, None));
        let enemy = &mut enemies[0];
        assert!(!enemy.take_damage(15.0));
        assert_eq!(enemy.health, 24.0);
        enemy.power_ups.tick(SHIELD_DURATION);
        assert!(enemy.power_ups.shield.is_some());
        enemy
            .power_ups
            .tick(GOLDEN_SHIELD_DURATION - SHIELD_DURATION);
        assert!(enemy.power_ups.shield.is_none());
        assert!(enemy.take_damage(24.0));
    }

    #[test]
    fn magnetic_shot_curves_into_a_moving_target() {
        let dt = 1.0 / 60.0;
        let mut pos = Vec2::ZERO;
        let mut vel = vec2(BULLET_SPEED, 0.0);
        let mut target = vec2(250.0, 135.0);

        for _ in 0..(BULLET_LIFETIME / dt) as usize {
            target += vec2(0.0, 80.0) * dt;
            vel = homing_velocity(pos, vel, std::iter::once(target), HOMING_TURN_SPEED * dt);
            assert!((vel.length() - BULLET_SPEED).abs() < 0.01);
            pos += vel * dt;
            if pos.distance(target) < ENEMY_SMALL_RADIUS + PLAYER_BULLET_RADIUS {
                return;
            }
        }

        panic!("magnetic shot failed to intercept the target within its lifetime");
    }

    #[test]
    fn magnetic_shot_ignores_targets_behind_it_or_out_of_range() {
        let vel = vec2(BULLET_SPEED, 0.0);
        let targets = [vec2(-20.0, 10.0), vec2(HOMING_RANGE + 1.0, 50.0)];
        assert_eq!(
            homing_velocity(
                Vec2::ZERO,
                vel,
                targets.into_iter(),
                HOMING_TURN_SPEED / 60.0
            ),
            vel
        );
    }
}
