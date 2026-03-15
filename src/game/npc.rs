use crate::components::*;
use crate::game::constants::*;
use crate::game::Game;
use macroquad::prelude::*;

pub fn update_enemies(game: &mut Game, dt: f32) {
    use macroquad::rand::gen_range;

    if game.current_mission.is_boss_level {
        return;
    }

    // On Easy: show hint first, then spawn ship after ENEMY_HINT_DELAY
    if let Some((from_left, spawn_y, hint_at)) = game.pending_enemy_hint {
        if get_time() - hint_at >= ENEMY_HINT_DELAY {
            game.enemy_ships
                .push(EnemyShip::new_at_side(from_left, spawn_y));
            game.pending_enemy_hint = None;
            let base_interval = game.current_mission.enemy_spawn_interval;
            game.enemy_spawn_timer = base_interval / game.difficulty.spawn_rate_mult();
        }
        // While pending, don't decrement timer or start a new spawn
    } else {
        game.enemy_spawn_timer -= dt;
        if game.enemy_spawn_timer <= 0.0 {
            if game.difficulty == Difficulty::Nebula {
                let from_left = gen_range(0, 2) == 0;
                let y = gen_range(50.0, screen_height() - 50.0);
                game.pending_enemy_hint = Some((from_left, y, get_time()));
            } else {
                game.enemy_ships.push(EnemyShip::new());
                let base_interval = game.current_mission.enemy_spawn_interval;
                game.enemy_spawn_timer = base_interval / game.difficulty.spawn_rate_mult();
            }
        }
    }

    for e in game.enemy_ships.iter_mut() {
        let diff = game.ship.pos - e.pos;
        e.rotation = diff.y.atan2(diff.x);

        match e.enemy_type {
            EnemyType::Regular => {
                // Regular enemy: move horizontally and shoot
                e.pos += e.vel * dt;
                e.shoot_timer -= dt;

                if e.shoot_timer <= 0.0 {
                    let bullet_vel = vec2(e.rotation.cos(), e.rotation.sin()) * 250.0;

                    game.bullets.push(Bullet {
                        pos: e.pos,
                        vel: bullet_vel,
                        life_time: 4.0,
                        style: BulletStyle::Enemy,
                        damage: ENEMY_BULLET_DAMAGE,
                        radius: 9.0,
                    });
                    e.shoot_timer = 2.0;
                    game.audio_cues.push(crate::audio::AudioCue::EnemyShot);
                }
            }
            EnemyType::Kamikaze => {
                // Kamikaze enemy: fly directly toward player
                let dir = diff.normalize();
                let kamikaze_speed = 180.0; // Speed toward player
                e.vel = dir * kamikaze_speed;
                e.pos += e.vel * dt;
            }
        }

        keep_enemy_in_bounds(e);
    }
    game.enemy_ships
        .retain(|e| e.pos.x > -100.0 && e.pos.x < screen_width() + 100.0);
}

fn keep_enemy_in_bounds(e: &mut EnemyShip) {
    let margin = ENEMY_BORDER_MARGIN;
    let w = screen_width();
    let h = screen_height();

    if e.pos.x < margin {
        e.vel.x = e.vel.x.max(60.0);
        e.pos.x = e.pos.x.max(margin);
    }
    if e.pos.x > w - margin {
        e.vel.x = e.vel.x.min(-60.0);
        e.pos.x = e.pos.x.min(w - margin);
    }
    if e.pos.y < margin {
        e.vel.y = e.vel.y.max(40.0);
        e.pos.y = e.pos.y.max(margin);
    }
    if e.pos.y > h - margin {
        e.vel.y = e.vel.y.min(-40.0);
        e.pos.y = e.pos.y.min(h - margin);
    }
}

pub fn update_boss(game: &mut Game, dt: f32) {
    let mut spawn_volley = false;
    let mut volley_pos = vec2(0.0, 0.0);
    if let Some(ref mut boss) = game.boss {
        boss.pos += boss.vel * dt;
        boss.rotation += BOSS_ROTATION_SPEED * dt * std::f32::consts::PI / 180.0;
        keep_boss_in_bounds(boss);
        boss.shoot_timer -= dt;
        if boss.shoot_timer <= 0.0 {
            if boss.burst_shots_left == 0 {
                boss.burst_shots_left = BOSS_BURST_SIZE;
            }
            spawn_volley = true;
            volley_pos = boss.pos;
            boss.burst_shots_left -= 1;
            boss.shoot_timer = if boss.burst_shots_left > 0 {
                BOSS_BURST_DELAY
            } else {
                BOSS_SHOOT_INTERVAL
            };
        }
    }
    if spawn_volley {
        game.audio_cues.push(crate::audio::AudioCue::EnemyShot);
        for (dx, dy) in BOSS_VOLLEY_DIRS {
            game.bullets.push(Bullet {
                pos: volley_pos,
                vel: vec2(dx * BOSS_BULLET_SPEED, dy * BOSS_BULLET_SPEED),
                life_time: 4.0,
                style: BulletStyle::Enemy,
                damage: ENEMY_BULLET_DAMAGE,
                radius: 9.0,
            });
        }
    }
}

fn keep_boss_in_bounds(boss: &mut Boss) {
    let margin = BOSS_BORDER_MARGIN;
    let w = screen_width();
    let h = screen_height();
    if boss.pos.x < margin {
        boss.vel.x = boss.vel.x.abs();
        boss.pos.x = margin;
    }
    if boss.pos.x > w - margin {
        boss.vel.x = -boss.vel.x.abs();
        boss.pos.x = w - margin;
    }
    if boss.pos.y < margin {
        boss.vel.y = boss.vel.y.abs();
        boss.pos.y = margin;
    }
    if boss.pos.y > h - margin {
        boss.vel.y = -boss.vel.y.abs();
        boss.pos.y = h - margin;
    }
}
