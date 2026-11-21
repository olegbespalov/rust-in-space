use crate::components::*;
use crate::game::constants::*;
use crate::game::Game;
use macroquad::prelude::*;

const TARGET_ENEMY_SPAWN_PERIOD: u32 = 4;

pub fn update_enemies(game: &mut Game, dt: f32) {
    use macroquad::rand::gen_range;

    // On Nebula, show the hint before spawning the ship.
    if let Some((from_left, spawn_y, hint_at)) = game.pending_enemy_hint {
        if get_time() - hint_at >= ENEMY_HINT_DELAY {
            spawn_enemy(game, from_left, spawn_y);
            game.pending_enemy_hint = None;
        }
        // While pending, don't decrement timer or start a new spawn
    } else {
        game.enemy_spawn_timer -= dt;
        if game.enemy_spawn_timer <= 0.0 {
            let from_left = gen_range(0, 2) == 0;
            let y = gen_range(50.0, screen_height() - 50.0);
            if game.difficulty == Difficulty::Nebula {
                game.pending_enemy_hint = Some((from_left, y, get_time()));
            } else {
                spawn_enemy(game, from_left, y);
            }
        }
    }

    let player_visible = !game.ship.is_invisible();
    for e in game.enemy_ships.iter_mut() {
        if !player_visible {
            e.pos += e.vel * dt;
            keep_enemy_in_bounds(e);
            continue;
        }

        let diff = game.ship.pos - e.pos;
        e.rotation = diff.y.atan2(diff.x);

        if e.enemy_type == EnemyType::Kamikaze {
            e.vel = diff.normalize_or_zero() * 180.0;
        }
        e.pos += e.vel * dt;

        if let Some((interval, speed, angles)) = enemy_volley(e.enemy_type) {
            e.shoot_timer -= dt;
            if e.shoot_timer <= 0.0 {
                for &offset in angles {
                    let angle = e.rotation + offset;
                    game.bullets.push(Bullet {
                        pos: e.pos,
                        vel: vec2(angle.cos(), angle.sin()) * speed,
                        life_time: 4.0,
                        style: BulletStyle::Enemy,
                        homing: false,
                        damage: ENEMY_BULLET_DAMAGE,
                        radius: 9.0,
                    });
                }
                e.shoot_timer = interval;
                game.audio_cues.push(crate::audio::AudioCue::EnemyShot);
            }
        }

        keep_enemy_in_bounds(e);
    }
    game.enemy_ships
        .retain(|e| e.pos.x > -100.0 && e.pos.x < screen_width() + 100.0);
}

fn spawn_enemy(game: &mut Game, from_left: bool, y: f32) {
    let mission = &game.current_mission;
    game.enemy_spawn_count += 1;
    let enemy_type =
        scheduled_target_enemy(mission, &game.mission_progress, game.enemy_spawn_count)
            .unwrap_or_else(|| {
                enemy_type_for_roll(mission.level_id, macroquad::rand::gen_range(0, 100))
            });
    game.enemy_ships
        .push(EnemyShip::new_at_side(from_left, y, enemy_type));
    game.enemy_spawn_timer = mission.enemy_spawn_interval / game.difficulty.spawn_rate_mult();
}

fn scheduled_target_enemy(
    mission: &Mission,
    progress: &super::MissionProgress,
    spawn_count: u32,
) -> Option<EnemyType> {
    if !spawn_count.is_multiple_of(TARGET_ENEMY_SPAWN_PERIOD) {
        return None;
    }

    mission.objectives.iter().find_map(|&objective| {
        if progress.meets(objective) {
            return None;
        }

        match objective {
            MissionObjective::DefeatInterceptors(_) => Some(EnemyType::Interceptor),
            MissionObjective::DefeatHeavyShips(_) => Some(EnemyType::Heavy),
            _ => None,
        }
    })
}

fn enemy_type_for_roll(level: u32, roll: i32) -> EnemyType {
    match roll {
        0..=29 => EnemyType::Kamikaze,
        30..=49 if level >= 2 => EnemyType::Interceptor,
        50..=64 if level >= 4 => EnemyType::Heavy,
        _ => EnemyType::Regular,
    }
}

fn enemy_volley(enemy_type: EnemyType) -> Option<(f32, f32, &'static [f32])> {
    match enemy_type {
        EnemyType::Regular => Some((2.0, 250.0, &[0.0])),
        EnemyType::Interceptor => Some((2.4, 320.0, &[0.0])),
        EnemyType::Heavy => Some((
            3.2,
            250.0,
            &[-HEAVY_VOLLEY_HALF_ANGLE, 0.0, HEAVY_VOLLEY_HALF_ANGLE],
        )),
        EnemyType::Kamikaze => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::MissionProgress;
    use crate::systems::get_mission;

    #[test]
    fn targeted_ships_are_scheduled_while_their_objective_is_incomplete() {
        for (level, target, quota) in [(2, EnemyType::Interceptor, 3), (4, EnemyType::Heavy, 2)] {
            let mission = get_mission(level);
            let mut progress = MissionProgress::default();
            assert!(scheduled_target_enemy(&mission, &progress, 3).is_none());
            assert!(scheduled_target_enemy(&mission, &progress, 4) == Some(target));

            for _ in 0..quota {
                progress.record_enemy_defeat(target);
            }
            assert!(scheduled_target_enemy(&mission, &progress, 8).is_none());
        }
    }

    #[test]
    fn enemy_variants_unlock_without_changing_kamikaze_frequency() {
        for (level, expected) in [
            (1, [70, 30, 0, 0]),
            (2, [50, 30, 20, 0]),
            (4, [35, 30, 20, 15]),
        ] {
            let mut counts = [0; 4];
            for roll in 0..100 {
                let index = match enemy_type_for_roll(level, roll) {
                    EnemyType::Regular => 0,
                    EnemyType::Kamikaze => 1,
                    EnemyType::Interceptor => 2,
                    EnemyType::Heavy => 3,
                };
                counts[index] += 1;
            }
            assert_eq!(counts, expected, "mission {level}");
        }
    }
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
                homing: false,
                damage: BOSS_BULLET_DAMAGE,
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
