use crate::audio::AudioCue;
use crate::components::*;
use crate::game::constants::*;
use crate::game::Game;
use crate::systems::{generate_loot, spawn_loot, spawn_shield_loot, wrap_around, LootSource};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::collections::HashSet;

pub fn update_timers(game: &mut Game, dt: f32) {
    game.mission_progress.elapsed_seconds += f64::from(dt);
    game.ship.shoot_timer -= dt;
    game.ship.rapid_fire_timer -= dt;
    game.ship.big_bullet_timer -= dt;
    game.ship.magnetic_ammo_timer -= dt;
    game.ship.invisibility_timer = (game.ship.invisibility_timer - dt).max(0.0);

    if game.ship.shield_timer > 0.0 {
        game.ship.shield_timer -= dt;
        if game.ship.shield_timer <= 0.0 {
            game.ship.shield_hp = 0.0;
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

fn homing_velocity(pos: Vec2, vel: Vec2, targets: impl Iterator<Item = Vec2>, dt: f32) -> Vec2 {
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
    let max_turn = HOMING_TURN_SPEED * dt;
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
                    b.vel = homing_velocity(b.pos, b.vel, targets, dt);
                }
                BulletStyle::Enemy if !game.ship.is_invisible() => {
                    b.vel = homing_velocity(b.pos, b.vel, std::iter::once(game.ship.pos), dt);
                }
                BulletStyle::Enemy => {}
            }
        }
        b.pos += b.vel * dt;
        b.life_time -= dt;
    }
    game.bullets.retain(|b| b.life_time > 0.0);

    for a in game.asteroids.iter_mut() {
        a.pos += a.vel * dt;
        wrap_around(&mut a.pos);
    }

    // Ramming can exhaust the field without earning credit for the objective.
    if game.asteroids.is_empty()
        && game.current_mission.objectives.iter().any(|&objective| {
            matches!(objective, MissionObjective::DestroyAsteroids(_))
                && !game.mission_progress.meets(objective)
        })
    {
        let mut asteroid = Asteroid::new_large();
        if game.ship.pos.x < screen_width() / 2.0 {
            asteroid.pos.x = screen_width();
            asteroid.vel.x = -asteroid.vel.x.abs();
        } else {
            asteroid.pos.x = 0.0;
            asteroid.vel.x = asteroid.vel.x.abs();
        }
        game.asteroids.push(asteroid);
    }
}

pub fn update_loot(game: &mut Game, dt: f32) -> bool {
    let magnet_radius = game.loot_magnet_radius();
    let mut ship_destroyed = false;

    game.loot_items.retain_mut(|item| {
        if ship_destroyed {
            return true;
        }
        item.time_left -= dt;
        if item.time_left <= 0.0 {
            return false;
        }

        item.vel *= LOOT_VEL_DECAY;
        item.pos += item.vel * dt;
        item.pos += item.drift_vel * dt;
        wrap_around(&mut item.pos);

        item.rotation += item.rotation_speed * dt;
        if item.rotation > std::f32::consts::PI * 2.0 {
            item.rotation -= std::f32::consts::PI * 2.0;
        } else if item.rotation < 0.0 {
            item.rotation += std::f32::consts::PI * 2.0;
        }

        let dist_to_ship = (game.ship.pos - item.pos).length();

        if dist_to_ship < magnet_radius {
            item.magnet_active = true;
        }

        if item.magnet_active {
            let dir = (game.ship.pos - item.pos).normalize_or_zero();
            item.pos += dir * LOOT_MAGNET_SPEED * dt;
        }

        if dist_to_ship < SHIP_RADIUS + item.radius {
            match item.item_type {
                LootType::Scrap(amount) => {
                    game.ship.scrap += amount;
                    game.mission_progress.scrap_collected += amount;
                    game.audio_cues.push(AudioCue::PickupScrap);
                }
                LootType::RareMetal(amount) => {
                    game.ship.rare_metal += amount;
                    game.audio_cues.push(AudioCue::PickupScrap);
                }
                LootType::HealthPack(hp) => {
                    game.ship.heal(hp as f32);
                    game.audio_cues.push(AudioCue::PickupHealth);
                }
                LootType::RapidFireBoost => {
                    game.ship.rapid_fire_timer = RAPID_FIRE_DURATION;
                    game.audio_cues.push(AudioCue::PickupScrap);
                }
                LootType::BigBulletBoost => {
                    game.ship.big_bullet_timer = BIG_BULLET_DURATION;
                    game.audio_cues.push(AudioCue::PickupScrap);
                }
                LootType::MagneticAmmo => {
                    game.ship.magnetic_ammo_timer = MAGNETIC_AMMO_DURATION;
                    game.audio_cues.push(AudioCue::PickupScrap);
                }
                LootType::Invisibility => {
                    game.ship.invisibility_timer = INVISIBILITY_DURATION;
                    game.audio_cues.push(AudioCue::PickupScrap);
                }
                LootType::Shield(kind, hp) => {
                    game.ship.activate_shield(kind, hp as f32, kind.duration());
                    game.audio_cues.push(AudioCue::ShieldOn);
                }
                LootType::MysteryBox => {
                    let (cue, lethal) = open_mystery_box(&mut game.ship, gen_range(0, 4));
                    game.audio_cues.push(cue);
                    ship_destroyed = lethal;
                }
            }
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
    ship_destroyed
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

fn open_mystery_box(ship: &mut Ship, roll: i32) -> (AudioCue, bool) {
    let cue = match MysteryEffect::from_roll(roll) {
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
            return (cue, ship.take_damage(damage));
        }
    };
    (cue, false)
}

pub fn update_collisions(game: &mut Game) -> bool {
    handle_bullet_bullet_collisions(game);
    handle_player_bullet_collisions(game);

    let bullet_hit_player = handle_enemy_bullet_collisions(game);
    let ship_hit_entity = handle_ship_entity_collisions(game);

    bullet_hit_player || ship_hit_entity
}

fn handle_bullet_bullet_collisions(game: &mut Game) {
    let mut bullets_to_remove = HashSet::new();
    for (i, b1) in game.bullets.iter().enumerate() {
        if b1.style != BulletStyle::Player {
            continue;
        }
        if bullets_to_remove.contains(&i) {
            continue;
        }
        for (j, b2) in game.bullets.iter().enumerate().skip(i + 1) {
            if b2.style != BulletStyle::Enemy {
                continue;
            }
            if bullets_to_remove.contains(&j) {
                continue;
            }

            if (b1.pos - b2.pos).length() < b1.radius + b2.radius {
                let collision_pos = (b1.pos + b2.pos) * 0.5;
                game.explosions.push(Explosion::new(collision_pos, 0.5));
                game.audio_cues.push(crate::audio::AudioCue::ExplosionSmall);
                bullets_to_remove.insert(i);
                bullets_to_remove.insert(j);
                break;
            }
        }
    }

    let mut sorted_indices: Vec<usize> = bullets_to_remove.into_iter().collect();
    sorted_indices.sort_unstable_by(|a, b| b.cmp(a));
    for &idx in &sorted_indices {
        if idx < game.bullets.len() {
            game.bullets.remove(idx);
        }
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

fn handle_player_bullet_collisions(game: &mut Game) {
    let mut new_asteroids = Vec::new();
    let mut boss_died_at: Option<(Vec2, f32)> = None;

    game.bullets.retain(|b| {
        if b.style != BulletStyle::Player {
            return true;
        }
        let mut hit = false;

        // Asteroids
        for i in (0..game.asteroids.len()).rev() {
            if (b.pos - game.asteroids[i].pos).length() < game.asteroids[i].radius + b.radius {
                game.score += 100;
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
            game.enemy_ships.retain_mut(|e| {
                if (b.pos - e.pos).length() < e.enemy_type.projectile_hitbox_radius() + b.radius {
                    hit = true;
                    if e.take_damage(b.damage) {
                        game.score += (e.max_health as u32) * SCORE_PER_ENEMY_HP;
                        drop_small_enemy_loot(&mut game.loot_items, e.pos, game.difficulty);
                        game.mission_progress.record_enemy_defeat(e.enemy_type);
                        game.explosions.push(Explosion::new(e.pos, 0.4));
                        game.audio_cues.push(crate::audio::AudioCue::ExplosionBig);
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            });
        }

        // Boss
        if !hit {
            if let Some(ref mut boss) = game.boss {
                if (b.pos - boss.pos).length() < BOSS_HITBOX_RADIUS + b.radius {
                    hit = true;
                    let pos = boss.pos;
                    let max_health = boss.max_health;
                    if boss.take_damage(b.damage) {
                        boss_died_at = Some((pos, max_health));
                    }
                }
            }
        }

        !hit
    });

    if let Some((pos, max_health)) = boss_died_at {
        finish_boss_defeat(game, pos, max_health);
    }
    game.asteroids.extend(new_asteroids);
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

fn handle_enemy_bullet_collisions(game: &mut Game) -> bool {
    let mut game_over = false;
    game.bullets.retain(|b| {
        if b.style == BulletStyle::Enemy
            && (b.pos - game.ship.pos).length() < SHIP_RADIUS + b.radius
        {
            game.explosions.push(Explosion::new(game.ship.pos, 0.5));
            if game.ship.has_shield() {
                game.audio_cues.push(crate::audio::AudioCue::ShieldHit);
            } else {
                game.audio_cues.push(crate::audio::AudioCue::ShipHit);
            }
            let damage = b.damage * game.difficulty.damage_mult();
            if game.ship.take_damage(damage) {
                game_over = true;
            }
            false
        } else {
            true
        }
    });
    game_over
}

fn handle_ship_entity_collisions(game: &mut Game) -> bool {
    let mut game_over = false;

    // Ship vs Asteroids
    for i in (0..game.asteroids.len()).rev() {
        if (game.ship.pos - game.asteroids[i].pos).length() < game.asteroids[i].radius + SHIP_RADIUS
        {
            let base_damage = (game.asteroids[i].radius / 10.0) * BASE_ASTEROID_DAMAGE;
            let asteroid_damage = base_damage * game.difficulty.damage_mult();
            let asteroid_radius = game.asteroids[i].radius;
            game.asteroids.remove(i);
            game.explosions.push(Explosion::new(
                game.ship.pos,
                (asteroid_radius / LARGE_ASTEROID_RADIUS).clamp(0.3, 0.8),
            ));
            if game.ship.has_shield() {
                game.audio_cues.push(crate::audio::AudioCue::ShieldHit);
            } else {
                game.audio_cues.push(crate::audio::AudioCue::ShipHit);
            }
            if game.ship.take_damage(asteroid_damage) {
                game_over = true;
            }
        }
    }

    // Ship vs Kamikaze
    game.enemy_ships.retain_mut(|e| {
        if e.enemy_type == EnemyType::Kamikaze
            && (game.ship.pos - e.pos).length() < KAMIKAZE_RADIUS + SHIP_RADIUS
        {
            let damage = BASE_KAMIKAZE_DAMAGE * game.difficulty.damage_mult();
            game.explosions.push(Explosion::new(e.pos, 0.6));
            if game.ship.has_shield() {
                game.audio_cues.push(crate::audio::AudioCue::ShieldHit);
            } else {
                game.audio_cues.push(crate::audio::AudioCue::ShipHit);
            }
            if game.ship.take_damage(damage) {
                game_over = true;
            }
            game.score += (e.max_health as u32) * SCORE_PER_ENEMY_HP;
            drop_small_enemy_loot(&mut game.loot_items, e.pos, game.difficulty);
            game.mission_progress.record_enemy_defeat(e.enemy_type);
            return false;
        }
        true
    });

    game_over
}

#[cfg(test)]
mod tests {
    use super::*;

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
            vel = homing_velocity(pos, vel, std::iter::once(target), dt);
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
            homing_velocity(Vec2::ZERO, vel, targets.into_iter(), 1.0 / 60.0),
            vel
        );
    }
}
