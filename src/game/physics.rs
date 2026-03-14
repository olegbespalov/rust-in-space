use crate::components::*;
use crate::game::constants::*;
use crate::game::Game;
use crate::systems::{generate_loot, wrap_around, LootSource};
use macroquad::prelude::*;
use std::collections::HashSet;

pub fn update_timers(game: &mut Game, dt: f32) {
    game.ship.shoot_timer -= dt;
    game.ship.rapid_fire_timer -= dt;
    game.ship.big_bullet_timer -= dt;

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

pub fn update_physics(game: &mut Game, dt: f32) {
    game.bullets.iter_mut().for_each(|b| {
        b.pos += b.vel * dt;
        b.life_time -= dt;
    });
    game.bullets.retain(|b| b.life_time > 0.0);

    for a in game.asteroids.iter_mut() {
        a.pos += a.vel * dt;
        wrap_around(&mut a.pos);
    }
}

pub fn update_loot(game: &mut Game, dt: f32) {
    let mut items_to_remove = Vec::new();
    let magnet_radius = game.loot_magnet_radius();

    for (i, item) in game.loot_items.iter_mut().enumerate() {
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
            let dir = (game.ship.pos - item.pos).normalize();
            item.pos += dir * LOOT_MAGNET_SPEED * dt;
        }

        if dist_to_ship < (SHIP_RADIUS + item.radius) {
            match item.item_type {
                LootType::Scrap(amount) => {
                    game.ship.scrap += amount;
                    game.mission_scrap_collected += amount;
                }
                LootType::RareMetal(amount) => {
                    game.ship.rare_metal += amount;
                    game.mission_rare_metal_collected += amount;
                }
                LootType::HealthPack(hp) => {
                    game.ship.heal(hp as f32);
                }
                LootType::RapidFireBoost => {
                    game.ship.rapid_fire_timer = RAPID_FIRE_DURATION;
                }
                LootType::BigBulletBoost => {
                    game.ship.big_bullet_timer = BIG_BULLET_DURATION;
                }
                LootType::Shield(hp) => {
                    game.ship.activate_shield(hp as f32, SHIELD_DURATION);
                }
            }
            items_to_remove.push(i);
        }
    }

    for &i in items_to_remove.iter().rev() {
        game.loot_items.remove(i);
    }
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
                if (b.pos - e.pos).length() < ENEMY_SMALL_RADIUS + b.radius {
                    hit = true;
                    if e.take_damage(b.damage) {
                        game.score += (e.max_health as u32) * SCORE_PER_ENEMY_HP;
                        if let Some(loot) =
                            generate_loot(e.pos, LootSource::EnemySmall, game.difficulty)
                        {
                            game.loot_items.push(loot);
                        }
                        game.mission_kills += 1;
                        game.explosions.push(Explosion::new(e.pos, 0.4));
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
        game.boss = None;
        game.score += (max_health as u32) * SCORE_PER_ENEMY_HP;
        game.explosions.push(Explosion::new(pos, 0.8));
        if let Some(loot) = generate_loot(pos, LootSource::EnemyBoss, game.difficulty) {
            game.loot_items.push(loot);
        }
    }
    game.asteroids.extend(new_asteroids);
}

fn handle_enemy_bullet_collisions(game: &mut Game) -> bool {
    let mut game_over = false;
    game.bullets.retain(|b| {
        if b.style == BulletStyle::Enemy
            && (b.pos - game.ship.pos).length() < SHIP_RADIUS + b.radius
        {
            game.explosions.push(Explosion::new(game.ship.pos, 0.5));
            let damage = b.damage * game.difficulty.damage_mult();
            if game.ship.take_damage(damage, game.score) {
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
            if game.ship.take_damage(asteroid_damage, game.score) {
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
            if game.ship.take_damage(damage, game.score) {
                game_over = true;
            }
            game.score += (e.max_health as u32) * SCORE_PER_ENEMY_HP;
            if let Some(loot) = generate_loot(e.pos, LootSource::EnemySmall, game.difficulty) {
                game.loot_items.push(loot);
            }
            game.mission_kills += 1;
            return false;
        }
        true
    });

    game_over
}
