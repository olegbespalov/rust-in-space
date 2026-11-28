use crate::components::*;
use crate::game::constants::*;
use crate::game::{nearest_pilot, Game};
use macroquad::prelude::*;

const TARGET_ENEMY_SPAWN_PERIOD: u32 = 4;
const GUARDIAN_UNLOCK_LEVEL: u32 = 4;
const GUARDIAN_VOLLEY_ANGLES: [f32; 5] = [-0.42, -0.21, 0.0, 0.21, 0.42];

pub fn update_enemies(game: &mut Game, dt: f32, now: f64) {
    if game.boss.is_none() {
        update_scheduled_enemy_spawn(game, dt, now);
    }

    let bounds = game.bounds;
    let visible_pilots = game
        .pilots
        .iter()
        .flatten()
        .filter(|pilot| pilot.is_alive() && !pilot.ship.is_invisible())
        .map(|pilot| pilot.ship.pos);
    for e in game.enemy_ships.iter_mut() {
        e.power_ups.tick(dt);
        let Some(target) = visible_pilots.clone().min_by(|a, b| {
            a.distance_squared(e.pos)
                .total_cmp(&b.distance_squared(e.pos))
        }) else {
            e.pos += e.vel * dt;
            keep_enemy_in_bounds(e, bounds);
            continue;
        };

        let diff = target - e.pos;
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

        keep_enemy_in_bounds(e, bounds);
    }
    game.enemy_ships
        .retain(|e| e.pos.x > -100.0 && e.pos.x < bounds.x + 100.0);
}

fn update_scheduled_enemy_spawn(game: &mut Game, dt: f32, now: f64) {
    use macroquad::rand::gen_range;

    // On Nebula, show the hint before spawning the ship.
    if let Some((from_left, spawn_y, hint_at)) = game.pending_enemy_hint {
        if now - hint_at >= game.enemy_hint_delay {
            spawn_enemy(game, from_left, spawn_y);
            game.pending_enemy_hint = None;
        }
        // While pending, don't decrement timer or start a new spawn
    } else {
        game.enemy_spawn_timer -= dt;
        if game.enemy_spawn_timer <= 0.0 {
            let from_left = gen_range(0, 2) == 0;
            let y = gen_range(50.0, game.bounds.y - 50.0);
            if game.difficulty == Difficulty::Nebula {
                game.pending_enemy_hint = Some((from_left, y, now));
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
        owner: None,
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
                if mission.level_id == 2 {
                    return EnemyType::Kamikaze;
                }

                let roll = macroquad::rand::gen_range(0, 100);
                let guardian_present = game
                    .enemy_ships
                    .iter()
                    .any(|enemy| enemy.enemy_type == EnemyType::Guardian);
                enemy_type_for_roll(mission.level_id, roll, guardian_present)
            });
    game.enemy_ships.push(EnemyShip::new_at_side(
        from_left,
        y,
        enemy_type,
        game.bounds.x,
    ));
    let elapsed = game.mission_progress.elapsed_seconds as f32;
    let interval = if mission.level_id == 2 {
        (mission.enemy_spawn_interval - elapsed * 0.02).max(0.5)
    } else {
        mission.enemy_spawn_interval
    };
    game.enemy_spawn_timer = interval / game.difficulty.spawn_rate_mult();
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

fn enemy_type_for_roll(level: u32, roll: i32, guardian_present: bool) -> EnemyType {
    match roll {
        0..=29 => EnemyType::Kamikaze,
        30..=49 if level >= 2 => EnemyType::Interceptor,
        50..=64 if level >= 4 => EnemyType::Heavy,
        65..=69 if level >= GUARDIAN_UNLOCK_LEVEL && !guardian_present => EnemyType::Guardian,
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
        EnemyType::Guardian => Some((3.6, 260.0, &GUARDIAN_VOLLEY_ANGLES)),
        EnemyType::Kamikaze => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coop_nebula_scales_the_full_spawn_cycle_for_the_duration_of_the_mission() {
        let mut solo = Game::new(vec2(1280.0, 720.0));
        solo.difficulty = Difficulty::Nebula;
        solo.next_mission();
        solo.start_mission();
        update_scheduled_enemy_spawn(&mut solo, 10.0, 1.0);
        update_scheduled_enemy_spawn(&mut solo, 0.0, 3.0);
        assert!(solo.enemy_ships.is_empty());
        update_scheduled_enemy_spawn(&mut solo, 0.0, 3.5);
        assert_eq!(solo.enemy_ships.len(), 1);

        let mut coop = Game::new(solo.bounds);
        coop.difficulty = Difficulty::Nebula;
        coop.next_mission();
        coop.set_pilot(1, ShipKind::Alpha);
        coop.prepare_lan_mission();
        coop.start_mission();
        coop.pilots[1] = None;
        update_scheduled_enemy_spawn(&mut coop, 10.0, 1.0);
        update_scheduled_enemy_spawn(&mut coop, 0.0, 2.99);
        assert!(coop.enemy_ships.is_empty());
        update_scheduled_enemy_spawn(&mut coop, 0.0, 3.0);
        assert_eq!(coop.enemy_ships.len(), 1);

        let solo_cycle = f64::from(solo.enemy_spawn_timer) + 2.5;
        let coop_cycle = f64::from(coop.enemy_spawn_timer) + 2.0;
        assert!((solo_cycle / coop_cycle - 1.25).abs() < 1e-6);
    }
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
    fn boss_changes_phases_at_health_thresholds_without_reverting_after_repairs() {
        let mut boss = opening_boss();
        let missile_threshold = boss.max_health * BOSS_MISSILE_PHASE_HEALTH_RATIO;
        let escalation_threshold = boss.max_health * BOSS_ESCALATION_HEALTH_RATIO;
        boss.health = missile_threshold + 1.0;
        assert!(advance(&mut boss, 0, 5.0).is_none());
        assert_eq!(boss.phase, BossPhase::Opening);
        assert!(boss.pending_summon.is_some());

        boss.take_damage(1.0);
        boss.shoot_timer = BossPhase::Opening.shot_interval();
        assert!(advance(&mut boss, 0, 1.0).is_none());
        assert_eq!(boss.phase, BossPhase::MissileSalvos);
        assert_eq!(boss.shoot_timer, 1.25);
        assert_eq!(boss.summon_timer, 4.25);
        assert!(boss.pending_summon.is_none());

        boss.health = escalation_threshold + 1.0;
        advance(&mut boss, 0, 0.0);
        assert_eq!(boss.phase, BossPhase::MissileSalvos);
        boss.take_damage(1.0);
        assert!(advance(&mut boss, 0, 1.0).is_none());
        assert_eq!(boss.phase, BossPhase::Escalated);
        assert_eq!(boss.shoot_timer, 1.0);
        assert_eq!(boss.summon_timer, 3.5);
        assert!(boss.pending_summon.is_none());

        boss.health = boss.max_health;
        advance(&mut boss, 0, 1.0);
        boss.take_damage(boss.max_health * (1.0 - BOSS_ESCALATION_HEALTH_RATIO));
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

        boss.health = boss.max_health * BOSS_ESCALATION_HEALTH_RATIO;
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
    fn boss_locks_its_aim_and_unlocks_missiles_then_rings() {
        use BossAttackKind::*;

        let mut boss = opening_boss();
        assert!(advance_boss_weapons(&mut boss, Some(0.5), 1.0).is_none());
        assert_eq!(boss.pending_attack.unwrap().kind, Cannons);
        assert!(advance_boss_weapons(&mut boss, Some(1.0), 0.25).is_none());
        assert_eq!(
            advance_boss_weapons(&mut boss, Some(1.0), Cannons.warning_duration()),
            Some((Cannons, 0))
        );
        assert_eq!(boss.rotation, 0.5);
        assert_eq!(boss.next_attack, Cannons);

        boss.health = boss.max_health * BOSS_MISSILE_PHASE_HEALTH_RATIO;
        advance(&mut boss, 0, 0.0);
        assert_eq!(boss.phase, BossPhase::MissileSalvos);
        assert!(
            advance_boss_weapons(&mut boss, Some(1.0), BOSS_MISSILE_PHASE_SHOOT_INTERVAL,)
                .is_none()
        );
        assert_eq!(
            advance_boss_weapons(&mut boss, Some(1.0), Missiles.warning_duration()),
            Some((Missiles, 0))
        );
        assert_eq!(boss.next_attack, Cannons);
        boss.health = boss.max_health * BOSS_ESCALATION_HEALTH_RATIO;
        advance(&mut boss, 0, 0.0);
        assert!(advance_boss_weapons(&mut boss, None, BOSS_ESCALATED_SHOOT_INTERVAL).is_none());
        assert_eq!(boss.pending_attack.unwrap().kind, Rings);
        assert_eq!(
            advance_boss_weapons(&mut boss, None, Rings.warning_duration()),
            Some((Rings, 0))
        );
        let (_, first_ray) = boss.attack_paths(Rings, 0).next().unwrap();
        for ring in 1..BOSS_RING_COUNT {
            assert_eq!(
                advance_boss_weapons(&mut boss, None, BOSS_RING_DELAY),
                Some((Rings, ring))
            );
            assert_ne!(boss.attack_paths(Rings, ring).next().unwrap().1, first_ray);
        }
        assert!(advance_boss_weapons(&mut boss, None, 0.0).is_none());
        assert_eq!(boss.rings_left, 0);
    }

    #[test]
    fn boss_shields_block_damage_and_recharge_faster_in_the_final_phase() {
        let mut boss = opening_boss();
        assert!(!advance_boss_shield(&mut boss, BOSS_SHIELD_INTERVAL));

        boss.health = boss.max_health * BOSS_MISSILE_PHASE_HEALTH_RATIO;
        advance(&mut boss, 0, 0.0);
        assert!(!advance_boss_shield(
            &mut boss,
            BOSS_FIRST_SHIELD_DELAY - 1.0
        ));
        assert!(advance_boss_shield(&mut boss, 1.0));
        assert_eq!(
            boss.power_ups.shield.as_ref().unwrap().hp,
            BOSS_SHIELD_HEALTH
        );
        let health = boss.health;
        boss.take_damage(15.0);
        assert_eq!(boss.health, health);
        assert_eq!(
            boss.power_ups.shield.as_ref().unwrap().hp,
            BOSS_SHIELD_HEALTH - 15.0
        );

        boss.power_ups.tick(BOSS_SHIELD_DURATION);
        assert!(boss.power_ups.shield.is_none());
        assert!(!advance_boss_shield(&mut boss, BOSS_SHIELD_DURATION));
        assert!(advance_boss_shield(
            &mut boss,
            BOSS_SHIELD_INTERVAL - BOSS_SHIELD_DURATION
        ));

        boss.health = boss.max_health * BOSS_ESCALATION_HEALTH_RATIO;
        advance(&mut boss, 0, 0.0);
        assert!(!advance_boss_shield(&mut boss, BOSS_FIRST_SHIELD_DELAY));
        assert_eq!(
            boss.power_ups.shield.as_ref().unwrap().hp,
            BOSS_SHIELD_HEALTH
        );
        boss.power_ups.tick(BOSS_SHIELD_DURATION);
        assert!(advance_boss_shield(&mut boss, 0.0));
        assert_eq!(
            boss.power_ups.shield.as_ref().unwrap().hp,
            BOSS_ESCALATED_SHIELD_HEALTH
        );
        assert_eq!(boss.shield_timer, BOSS_ESCALATED_SHIELD_INTERVAL);
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
            (1, [70, 30, 0, 0, 0]),
            (2, [50, 30, 20, 0, 0]),
            (3, [50, 30, 20, 0, 0]),
            (4, [30, 30, 20, 15, 5]),
        ] {
            let mut counts = [0; 5];
            for roll in 0..100 {
                let index = match enemy_type_for_roll(level, roll, false) {
                    EnemyType::Regular => 0,
                    EnemyType::Kamikaze => 1,
                    EnemyType::Interceptor => 2,
                    EnemyType::Heavy => 3,
                    EnemyType::Guardian => 4,
                };
                counts[index] += 1;
            }
            assert_eq!(counts, expected, "mission {level}");
        }
        assert!(enemy_type_for_roll(4, 65, true) == EnemyType::Regular);
    }
}

fn keep_enemy_in_bounds(e: &mut EnemyShip, bounds: Vec2) {
    let margin = ENEMY_BORDER_MARGIN;
    let w = bounds.x;
    let h = bounds.y;

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
    let Some(boss) = game.boss.as_mut() else {
        return;
    };
    boss.power_ups.tick(dt);
    if advance_boss_shield(boss, dt) {
        game.audio_cues.push(crate::audio::AudioCue::ShieldOn);
    }
    if boss.pending_attack.is_none() && boss.rings_left == 0 {
        boss.pos += boss.vel * dt;
    }
    keep_boss_in_bounds(boss, game.bounds);
    let aim = nearest_pilot(&game.pilots, boss.pos, |pilot| !pilot.ship.is_invisible())
        .and_then(|slot| game.pilots[slot].as_ref())
        .map(|pilot| {
            let offset = pilot.ship.pos - boss.pos;
            offset.y.atan2(offset.x)
        });
    let Some((kind, ring)) = advance_boss_weapons(boss, aim, dt) else {
        return;
    };

    game.audio_cues.push(crate::audio::AudioCue::EnemyShot);
    game.bullets
        .extend(boss.attack_paths(kind, ring).map(|(pos, direction)| {
            let missile = kind == BossAttackKind::Missiles;
            let (speed, damage) = if missile {
                (BOSS_MISSILE_SPEED, BOSS_MISSILE_DAMAGE)
            } else {
                (BOSS_BULLET_SPEED, BOSS_BULLET_DAMAGE)
            };
            let mut bullet = enemy_bullet(pos, direction * speed, damage, &boss.power_ups);
            bullet.style = if missile {
                bullet.homing = true;
                bullet.radius *= 1.25;
                BulletStyle::BossMissile
            } else {
                BulletStyle::Boss
            };
            bullet
        }));
}

fn advance_boss_shield(boss: &mut Boss, dt: f32) -> bool {
    let (interval, hp) = match boss.phase {
        BossPhase::Opening => return false,
        BossPhase::MissileSalvos => (BOSS_SHIELD_INTERVAL, BOSS_SHIELD_HEALTH),
        BossPhase::Escalated => (BOSS_ESCALATED_SHIELD_INTERVAL, BOSS_ESCALATED_SHIELD_HEALTH),
    };
    boss.shield_timer = (boss.shield_timer - dt).max(0.0);
    if boss.shield_timer > 0.0 || boss.power_ups.shield.is_some() {
        return false;
    }
    boss.power_ups
        .activate_shield(ShieldKind::Standard, hp, BOSS_SHIELD_DURATION);
    boss.shield_timer = interval;
    true
}

fn advance_boss_weapons(
    boss: &mut Boss,
    aim: Option<f32>,
    dt: f32,
) -> Option<(BossAttackKind, u32)> {
    use BossAttackKind::*;

    if let Some(attack) = boss.pending_attack.as_mut() {
        attack.remaining -= dt;
        if attack.remaining > 0.0 {
            return None;
        }
        let kind = attack.kind;
        boss.pending_attack = None;
        boss.rings_left = if kind == Rings {
            BOSS_RING_COUNT - 1
        } else {
            0
        };
        boss.shoot_timer = boss_recovery(boss);
        return Some((kind, 0));
    }

    boss.shoot_timer = (boss.shoot_timer - dt).max(0.0);
    if boss.rings_left > 0 {
        if boss.shoot_timer > 0.0 {
            return None;
        }
        let ring = BOSS_RING_COUNT - boss.rings_left;
        boss.rings_left -= 1;
        boss.shoot_timer = boss_recovery(boss);
        return Some((Rings, ring));
    }

    if let Some(angle) = aim {
        let difference = angle - boss.rotation;
        let turn = difference.sin().atan2(difference.cos());
        boss.rotation += turn.clamp(-1.9 * dt, 1.9 * dt);
    }
    if boss.shoot_timer > 0.0 {
        return None;
    }

    let kind = if aim.is_some() {
        boss.next_attack
    } else if boss.phase == BossPhase::Escalated {
        Rings
    } else {
        return None;
    };
    if let Some(angle) = aim.filter(|_| kind != Rings) {
        boss.rotation = angle;
    }
    boss.pending_attack = Some(BossAttack {
        kind,
        remaining: kind.warning_duration(),
    });
    boss.next_attack = match kind {
        Cannons if boss.phase != BossPhase::Opening => Missiles,
        Missiles if boss.phase == BossPhase::Escalated => Rings,
        _ => Cannons,
    };
    None
}

fn boss_recovery(boss: &Boss) -> f32 {
    if boss.rings_left > 0 {
        BOSS_RING_DELAY
    } else {
        enemy_shot_delay(boss.phase.shot_interval(), &boss.power_ups)
    }
}

impl BossPhase {
    fn shot_interval(self) -> f32 {
        match self {
            Self::Opening => BOSS_SHOOT_INTERVAL,
            Self::MissileSalvos => BOSS_MISSILE_PHASE_SHOOT_INTERVAL,
            Self::Escalated => BOSS_ESCALATED_SHOOT_INTERVAL,
        }
    }

    fn summon_types(self) -> &'static [EnemyType] {
        match self {
            Self::Opening => &[EnemyType::Regular],
            Self::MissileSalvos => &[EnemyType::Interceptor],
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
        BossPhase::MissileSalvos => BOSS_MISSILE_PHASE_SUMMON_INTERVAL,
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
        let height = game.bounds.y;
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
                EnemyShip::new_at_side(from_left, y, enemy_type, game.bounds.x)
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
        boss.pending_attack = None;
        boss.rings_left = 0;
        return None;
    }

    let health_phase = if boss.health <= boss.max_health * BOSS_ESCALATION_HEALTH_RATIO {
        BossPhase::Escalated
    } else if boss.health <= boss.max_health * BOSS_MISSILE_PHASE_HEALTH_RATIO {
        BossPhase::MissileSalvos
    } else {
        BossPhase::Opening
    };
    if health_phase > boss.phase {
        boss.phase = health_phase;
        boss.pending_attack = None;
        boss.rings_left = 0;
        boss.next_attack = match health_phase {
            BossPhase::Opening => BossAttackKind::Cannons,
            BossPhase::MissileSalvos => BossAttackKind::Missiles,
            BossPhase::Escalated => BossAttackKind::Rings,
        };
        boss.shoot_timer = boss.shoot_timer.min(boss_recovery(boss));
        boss.shield_timer = BOSS_FIRST_SHIELD_DELAY;
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

fn keep_boss_in_bounds(boss: &mut Boss, bounds: Vec2) {
    let margin = Vec2::splat(BOSS_SIZE * 0.6).min(bounds / 2.0);
    let max = vec2(bounds.x - margin.x, (bounds.y * 0.52).max(margin.y));
    let clamped = boss.pos.clamp(margin, max);
    if boss.pos.x != clamped.x {
        boss.vel.x = boss.vel.x.copysign(clamped.x - boss.pos.x);
    }
    if boss.pos.y != clamped.y {
        boss.vel.y = boss.vel.y.copysign(clamped.y - boss.pos.y);
    }
    boss.pos = clamped;
}
