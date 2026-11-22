use crate::components::*;
use crate::game::constants::*;
use crate::game::Game;
use macroquad::prelude::*;

const TARGET_ENEMY_SPAWN_PERIOD: u32 = 4;

pub fn update_enemies(game: &mut Game, dt: f32) {
    if game.boss.is_none() {
        update_scheduled_enemy_spawn(game, dt);
    }

    let player_visible = !game.ship.is_invisible();
    for e in game.enemy_ships.iter_mut() {
        e.power_ups.tick(dt);
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
                    game.bullets.push(enemy_bullet(
                        e.pos,
                        vec2(angle.cos(), angle.sin()) * speed,
                        ENEMY_BULLET_DAMAGE,
                        &e.power_ups,
                    ));
                }
                e.shoot_timer = enemy_shot_delay(interval, &e.power_ups);
                game.audio_cues.push(crate::audio::AudioCue::EnemyShot);
            }
        }

        keep_enemy_in_bounds(e);
    }
    game.enemy_ships
        .retain(|e| e.pos.x > -100.0 && e.pos.x < screen_width() + 100.0);
}

fn update_scheduled_enemy_spawn(game: &mut Game, dt: f32) {
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
}

fn enemy_bullet(pos: Vec2, vel: Vec2, base_damage: f32, power_ups: &EnemyPowerUps) -> Bullet {
    const BASE_RADIUS: f32 = 9.0;
    let (damage, radius) = if power_ups.big_bullet_timer > 0.0 {
        (
            base_damage * (BIG_BULLET_DAMAGE / PLAYER_BULLET_DAMAGE),
            BASE_RADIUS * (BIG_BULLET_RADIUS / PLAYER_BULLET_RADIUS),
        )
    } else {
        (base_damage, BASE_RADIUS)
    };
    Bullet {
        pos,
        vel,
        life_time: 4.0,
        style: BulletStyle::Enemy,
        homing: power_ups.magnetic_ammo_timer > 0.0,
        damage,
        radius,
    }
}

fn enemy_shot_delay(interval: f32, power_ups: &EnemyPowerUps) -> f32 {
    if power_ups.rapid_fire_timer > 0.0 {
        interval / 3.0
    } else {
        interval
    }
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

    fn advance(boss: &mut Boss, enemy_count: usize, dt: f32) -> Option<BossSummon> {
        advance_boss_encounter(boss, Difficulty::Supernova, enemy_count, dt, || {
            (true, 160.0)
        })
    }

    fn opening_boss() -> Boss {
        Boss::new(
            Vec2::ZERO,
            boss_summon_delay(BossPhase::Opening, Difficulty::Supernova),
        )
    }

    #[test]
    fn boss_escalates_once_at_half_health_and_replaces_the_pending_wave() {
        let mut boss = opening_boss();
        boss.health = 226.0;
        assert!(advance(&mut boss, 0, 5.0).is_none());
        assert_eq!(boss.phase, BossPhase::Opening);
        assert!(boss.pending_summon.is_some());

        boss.take_damage(1.0);
        boss.shoot_timer = BossPhase::Opening.shot_interval();
        assert!(advance(&mut boss, 0, 1.0).is_none());
        assert_eq!(boss.phase, BossPhase::Escalated);
        assert_eq!(boss.shoot_timer, 2.0);
        assert_eq!(boss.summon_timer, 3.5);
        assert!(boss.pending_summon.is_none());

        boss.health = boss.max_health;
        advance(&mut boss, 0, 1.0);
        boss.take_damage(boss.max_health / 2.0);
        advance(&mut boss, 0, 1.0);
        assert_eq!(boss.phase, BossPhase::Escalated);
        assert_eq!(boss.summon_timer, 1.5);
    }

    #[test]
    fn boss_warns_before_each_wave_and_only_announces_waves_that_fit() {
        let mut boss = opening_boss();
        assert!(advance(&mut boss, 0, 4.75).is_none());
        assert!(boss.pending_summon.is_none());
        assert!(advance(&mut boss, 0, 0.25).is_none());
        assert_eq!(boss.pending_summon.unwrap().remaining, 1.0);
        assert!(advance(&mut boss, 0, 0.75).is_none());
        let first_wave = advance(&mut boss, 0, 0.25).unwrap();
        assert!(first_wave
            .arrivals(boss.phase)
            .eq([(true, 160.0, EnemyType::Regular)]));
        assert_eq!(boss.summon_timer, 5.0);

        boss.health = boss.max_health / 2.0;
        advance(&mut boss, 3, 0.0);
        assert!(advance(&mut boss, 3, 3.5).is_none());
        assert!(boss.pending_summon.is_none());
        assert!(advance(&mut boss, 2, 0.0).is_none());
        assert_eq!(boss.pending_summon.unwrap().remaining, 1.0);
        let second_wave = advance(&mut boss, 2, 1.0).unwrap();
        assert!(second_wave.arrivals(boss.phase).eq([
            (true, 160.0, EnemyType::Interceptor),
            (false, 160.0, EnemyType::Kamikaze),
        ]));
        assert_eq!(boss.summon_timer, 3.5);
    }

    #[test]
    fn boss_summon_interval_scales_with_difficulty_before_the_fixed_warning() {
        assert_eq!(
            boss_summon_delay(BossPhase::Opening, Difficulty::Nebula),
            6.5
        );
        assert_eq!(
            boss_summon_delay(BossPhase::Escalated, Difficulty::BlackHole),
            4.5 / 1.3 - 1.0
        );
    }

    #[test]
    fn lethal_damage_cancels_the_pending_boss_wave() {
        let mut boss = opening_boss();
        advance(&mut boss, 0, 5.0);
        assert!(boss.pending_summon.is_some());
        assert!(boss.take_damage(boss.max_health));
        assert!(advance(&mut boss, 0, 1.0).is_none());
        assert!(boss.pending_summon.is_none());
        assert_eq!(boss.phase, BossPhase::Opening);
    }

    #[test]
    fn enemy_weapon_pickups_change_new_shots_until_the_boosts_expire() {
        let mut power_ups = EnemyPowerUps::default();
        let mut health = 30.0;
        for item in [
            LootType::BigBulletBoost,
            LootType::RapidFireBoost,
            LootType::MagneticAmmo,
        ] {
            power_ups.pick_up(&item, &mut health, 30.0);
        }

        let boosted = enemy_bullet(Vec2::ZERO, Vec2::X, ENEMY_BULLET_DAMAGE, &power_ups);
        assert_eq!(boosted.damage, ENEMY_BULLET_DAMAGE * 2.0);
        assert_eq!(boosted.radius, 18.0);
        assert!(boosted.homing);
        assert_eq!(enemy_shot_delay(3.0, &power_ups), 1.0);

        power_ups.tick(BIG_BULLET_DURATION);
        let normal = enemy_bullet(Vec2::ZERO, Vec2::X, ENEMY_BULLET_DAMAGE, &power_ups);
        assert_eq!(normal.damage, ENEMY_BULLET_DAMAGE);
        assert_eq!(normal.radius, 9.0);
        assert!(!normal.homing);
        assert_eq!(enemy_shot_delay(3.0, &power_ups), 3.0);
    }

    #[test]
    fn targeted_ships_are_scheduled_while_their_objective_is_incomplete() {
        for (level, target, quota) in [(2, EnemyType::Interceptor, 5), (4, EnemyType::Heavy, 3)] {
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
    if let Some(ref mut boss) = game.boss {
        boss.power_ups.tick(dt);
        boss.pos += boss.vel * dt;
        boss.rotation += BOSS_ROTATION_SPEED * dt * std::f32::consts::PI / 180.0;
        keep_boss_in_bounds(boss);
        boss.shoot_timer -= dt;
        if boss.shoot_timer <= 0.0 {
            if boss.burst_shots_left == 0 {
                boss.burst_shots_left = BOSS_BURST_SIZE;
            }
            boss.burst_shots_left -= 1;
            boss.shoot_timer = if boss.burst_shots_left > 0 {
                BOSS_BURST_DELAY
            } else {
                enemy_shot_delay(boss.phase.shot_interval(), &boss.power_ups)
            };
            game.audio_cues.push(crate::audio::AudioCue::EnemyShot);
            for (dx, dy) in BOSS_VOLLEY_DIRS {
                game.bullets.push(enemy_bullet(
                    boss.pos,
                    vec2(dx, dy) * BOSS_BULLET_SPEED,
                    BOSS_BULLET_DAMAGE,
                    &boss.power_ups,
                ));
            }
        }
    }
}

impl BossPhase {
    fn shot_interval(self) -> f32 {
        match self {
            Self::Opening => BOSS_SHOOT_INTERVAL,
            Self::Escalated => BOSS_ESCALATED_SHOOT_INTERVAL,
        }
    }

    fn summon_types(self) -> &'static [EnemyType] {
        match self {
            Self::Opening => &[EnemyType::Regular],
            Self::Escalated => &[EnemyType::Interceptor, EnemyType::Kamikaze],
        }
    }
}

impl BossSummon {
    pub fn arrivals(self, phase: BossPhase) -> impl Iterator<Item = (bool, f32, EnemyType)> {
        phase
            .summon_types()
            .iter()
            .enumerate()
            .map(move |(index, &enemy_type)| {
                let from_left = if index == 0 {
                    self.from_left
                } else {
                    !self.from_left
                };
                (from_left, self.y, enemy_type)
            })
    }
}

pub(super) fn boss_summon_delay(phase: BossPhase, difficulty: Difficulty) -> f32 {
    let interval = match phase {
        BossPhase::Opening => BOSS_SUMMON_INTERVAL,
        BossPhase::Escalated => BOSS_ESCALATED_SUMMON_INTERVAL,
    };
    interval / difficulty.spawn_rate_mult() - BOSS_SUMMON_WARNING_DURATION
}

pub fn update_boss_encounter(game: &mut Game, dt: f32) {
    let Some(boss) = game.boss.as_mut() else {
        return;
    };
    let wave = advance_boss_encounter(boss, game.difficulty, game.enemy_ships.len(), dt, || {
        use macroquad::rand::gen_range;
        let height = screen_height();
        let margin = 50.0_f32.min(height / 2.0);
        let y = if height > 2.0 * margin {
            gen_range(margin, height - margin)
        } else {
            height / 2.0
        };
        (gen_range(0, 2) == 0, y)
    });
    if let Some(wave) = wave {
        game.enemy_ships
            .extend(wave.arrivals(boss.phase).map(|(from_left, y, enemy_type)| {
                EnemyShip::new_at_side(from_left, y, enemy_type)
            }));
    }
}

fn advance_boss_encounter(
    boss: &mut Boss,
    difficulty: Difficulty,
    enemy_count: usize,
    dt: f32,
    choose_entry: impl FnOnce() -> (bool, f32),
) -> Option<BossSummon> {
    if boss.health <= 0.0 {
        boss.pending_summon = None;
        return None;
    }

    if boss.phase == BossPhase::Opening
        && boss.health <= boss.max_health * BOSS_ESCALATION_HEALTH_RATIO
    {
        boss.phase = BossPhase::Escalated;
        if boss.burst_shots_left == 0 {
            let new_delay = enemy_shot_delay(boss.phase.shot_interval(), &boss.power_ups);
            boss.shoot_timer = boss.shoot_timer.min(new_delay);
        }
        boss.pending_summon = None;
        boss.summon_timer = boss_summon_delay(boss.phase, difficulty);
        return None;
    }

    if let Some(wave) = boss.pending_summon.as_mut() {
        wave.remaining -= dt;
        if wave.remaining <= 0.0 {
            boss.summon_timer = boss_summon_delay(boss.phase, difficulty) + wave.remaining;
            return boss.pending_summon.take();
        }
    } else {
        boss.summon_timer = (boss.summon_timer - dt).max(0.0);
        if boss.summon_timer == 0.0
            && enemy_count + boss.phase.summon_types().len() <= BOSS_REINFORCEMENT_LIMIT
        {
            let (from_left, y) = choose_entry();
            boss.pending_summon = Some(BossSummon {
                from_left,
                y,
                remaining: BOSS_SUMMON_WARNING_DURATION,
            });
        }
    }
    None
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
