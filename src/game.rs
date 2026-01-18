use crate::components::*;
use crate::draw::*;
use crate::resources::Resources;
use crate::systems::{generate_loot, get_mission, load_score, wrap_around, LootSource};
use macroquad::prelude::*;

// Game constants
pub const ROTATION_SPEED: f32 = 200.0;
pub const ACCELERATION: f32 = 150.0;
pub const BULLET_SPEED: f32 = 400.0;
pub const BULLET_LIFETIME: f32 = 2.0;
pub const SHOOT_COOLDOWN: f32 = 0.3;
pub const PLAYER_BULLET_DAMAGE: f32 = 10.0;
pub const PLAYER_BULLET_RADIUS: f32 = 6.0;
pub const BIG_BULLET_DAMAGE: f32 = 20.0;
pub const BIG_BULLET_RADIUS: f32 = 12.0;
pub const ENEMY_BULLET_DAMAGE: f32 = 15.0;
pub const BASE_ASTEROID_DAMAGE: f32 = 5.0;
pub const SCORE_PER_ENEMY_HP: u32 = 10;

pub struct Game {
    pub ship: Ship,
    pub bullets: Vec<Bullet>,
    pub asteroids: Vec<Asteroid>,
    pub enemy_ships: Vec<EnemyShip>,
    pub loot_items: Vec<LootItem>,
    pub explosions: Vec<Explosion>,
    pub score: u32,
    pub current_level_idx: u32,
    pub current_mission: Mission,
    pub mission_kills: u32,
    pub mission_scrap_collected: u32,
    pub mission_rare_metal_collected: u32,
    pub enemy_spawn_timer: f32,
}

impl Game {
    pub fn new() -> Self {
        Self {
            ship: create_ship(),
            bullets: Vec::new(),
            asteroids: Vec::new(),
            enemy_ships: Vec::new(),
            loot_items: Vec::new(),
            explosions: Vec::new(),
            score: 0,
            current_level_idx: 1,
            current_mission: get_mission(1),
            mission_kills: 0,
            mission_scrap_collected: 0,
            mission_rare_metal_collected: 0,
            enemy_spawn_timer: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.bullets.clear();
        self.asteroids = (0..5).map(|_| Asteroid::new_large()).collect();
        self.loot_items.clear();
        self.enemy_ships.clear();
        self.score = 0;
        self.current_level_idx = 1;
        self.current_mission = get_mission(self.current_level_idx);
        self.ship = create_ship();
    }

    pub fn start_mission(&mut self) {
        self.bullets.clear();
        self.enemy_ships.clear();
        self.loot_items.clear();

        self.asteroids = (0..self.current_mission.asteroid_count)
            .map(|_| Asteroid::new_large())
            .collect();

        self.mission_kills = 0;
        self.mission_scrap_collected = 0;
        self.mission_rare_metal_collected = 0;
        self.enemy_spawn_timer = self.current_mission.enemy_spawn_interval;

        self.ship.pos = vec2(screen_width() / 2.0, screen_height() / 2.0);
        self.ship.vel = vec2(0.0, 0.0);
    }

    pub fn next_mission(&mut self) {
        self.current_level_idx += 1;
        self.current_mission = get_mission(self.current_level_idx);
    }

    pub fn is_mission_complete(&self) -> bool {
        self.mission_kills >= self.current_mission.target_kills
            && self.mission_scrap_collected >= self.current_mission.target_scrap
            && self.mission_rare_metal_collected >= self.current_mission.target_rare_metal
    }
}

pub fn create_ship() -> Ship {
    Ship {
        pos: vec2(screen_width() / 2.0, screen_height() / 2.0),
        vel: vec2(0.0, 0.0),
        rotation: 0.0,
        health: 100.0,
        max_health: 100.0,
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

    game.enemy_spawn_timer -= dt;
    if game.enemy_spawn_timer <= 0.0 {
        game.enemy_ships.push(EnemyShip::new());
        game.enemy_spawn_timer = game.current_mission.enemy_spawn_interval;
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
        let thrust_force = game.ship.engine.current_thrust * ACCELERATION;
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
            damage,
            radius,
        });
        game.ship.shoot_timer = current_cooldown;
    }
}

pub fn update_enemies(game: &mut Game, dt: f32) {
    for e in game.enemy_ships.iter_mut() {
        e.pos += e.vel * dt;
        e.shoot_timer -= dt;

        let diff = game.ship.pos - e.pos;
        e.rotation = diff.y.atan2(diff.x);

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
        }
    }
    game.enemy_ships
        .retain(|e| e.pos.x > -100.0 && e.pos.x < screen_width() + 100.0);
}

pub fn update_loot(game: &mut Game, dt: f32) {
    let mut items_to_remove = Vec::new();

    for (i, item) in game.loot_items.iter_mut().enumerate() {
        item.vel *= 0.95;
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

        if dist_to_ship < 150.0 {
            item.magnet_active = true;
        }

        if item.magnet_active {
            let dir = (game.ship.pos - item.pos).normalize();
            let magnet_speed = 300.0;
            item.pos += dir * magnet_speed * dt;
        }

        if dist_to_ship < (72.0 / 2.0 + item.radius) {
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
                    game.ship.rapid_fire_timer = 10.0;
                }
                LootType::BigBulletBoost => {
                    game.ship.big_bullet_timer = 15.0;
                }
                LootType::Shield(hp) => {
                    game.ship.activate_shield(hp as f32, 30.0);
                }
            }
            items_to_remove.push(i);
        }
    }

    for &i in items_to_remove.iter().rev() {
        game.loot_items.remove(i);
    }
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

pub fn update_collisions(game: &mut Game) -> bool {
    let mut new_asteroids = Vec::new();
    let mut game_over = false;

    // Player bullets vs asteroids and enemies
    game.bullets.retain(|b| {
        if b.style != BulletStyle::Player {
            return true;
        }

        let mut hit = false;

        // Check asteroid collisions
        for i in (0..game.asteroids.len()).rev() {
            if (b.pos - game.asteroids[i].pos).length() < game.asteroids[i].radius + b.radius {
                game.score += 100;
                let is_rare = game.asteroids[i].is_rare;
                let asteroid_pos = game.asteroids[i].pos;

                if is_rare {
                    if let Some(loot) = generate_loot(asteroid_pos, LootSource::RareAsteroid) {
                        game.loot_items.push(loot);
                    }
                } else if let Some(loot) = generate_loot(asteroid_pos, LootSource::Asteroid) {
                    game.loot_items.push(loot);
                }

                let old = game.asteroids.remove(i);
                if old.radius > 15.0 {
                    new_asteroids.push(Asteroid::new_fragment(old.pos, old.radius));
                    new_asteroids.push(Asteroid::new_fragment(old.pos, old.radius));
                }
                hit = true;
                break;
            }
        }

        // Check enemy collisions
        game.enemy_ships.retain_mut(|e| {
            if (b.pos - e.pos).length() < 30.0 + b.radius {
                hit = true;
                if e.take_damage(b.damage) {
                    let score_gain = (e.max_health as u32) * SCORE_PER_ENEMY_HP;
                    game.score += score_gain;
                    if let Some(loot) = generate_loot(e.pos, LootSource::EnemySmall) {
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

        !hit
    });
    game.asteroids.extend(new_asteroids);

    // Enemy bullets vs player
    game.bullets.retain(|b| {
        if b.style == BulletStyle::Enemy && (b.pos - game.ship.pos).length() < 20.0 + b.radius {
            game.explosions.push(Explosion::new(game.ship.pos, 0.5));
            if game.ship.take_damage(b.damage, game.score) {
                game_over = true;
            }
            false
        } else {
            true
        }
    });

    // Ship vs asteroids
    for i in (0..game.asteroids.len()).rev() {
        if (game.ship.pos - game.asteroids[i].pos).length() < game.asteroids[i].radius + 10.0 {
            let asteroid_damage = (game.asteroids[i].radius / 10.0) * BASE_ASTEROID_DAMAGE;
            let asteroid_radius = game.asteroids[i].radius;
            game.asteroids.remove(i);
            let explosion_scale = (asteroid_radius / 40.0).clamp(0.3, 0.8);
            game.explosions
                .push(Explosion::new(game.ship.pos, explosion_scale));
            if game.ship.take_damage(asteroid_damage, game.score) {
                game_over = true;
            }
        }
    }

    game_over
}

pub fn render_game(game: &Game, resources: &Resources) {
    for item in &game.loot_items {
        draw_loot(item, resources);
    }

    for b in &game.bullets {
        let texture = match b.style {
            BulletStyle::Player => &resources.bullet,
            BulletStyle::Enemy => &resources.enemy_bullet,
        };

        let rotation = b.vel.y.atan2(b.vel.x) + std::f32::consts::FRAC_PI_2;
        let size = b.radius * 2.0;

        draw_texture_ex(
            texture,
            b.pos.x - size / 2.0,
            b.pos.y - size / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                rotation,
                ..Default::default()
            },
        );
    }

    for a in &game.asteroids {
        draw_asteroid(a, resources);
    }

    for e in &game.enemy_ships {
        draw_enemy(e, resources);
    }

    for ex in &game.explosions {
        draw_explosion(ex, resources);
    }

    draw_ship(
        &game.ship,
        &resources.ship_body,
        &resources.ship_flame,
        Some(&resources.shield_active),
    );

    let mut status_text = format!(
        "SCORE: {}  HP: {:.0}/{:.0}",
        game.score, game.ship.health, game.ship.max_health
    );
    if game.ship.has_shield() {
        status_text.push_str(&format!(
            "  SHIELD: {:.0}/{:.0}",
            game.ship.shield_hp, game.ship.shield_max_hp
        ));
    }
    draw_text(&status_text, 20.0, 30.0, 30.0, WHITE);

    let status = format!(
        "Kills: {}/{}  Rust: {}/{}  Gold: {}/{}",
        game.mission_kills,
        game.current_mission.target_kills,
        game.mission_scrap_collected,
        game.current_mission.target_scrap,
        game.mission_rare_metal_collected,
        game.current_mission.target_rare_metal
    );
    draw_text(&status, 20.0, screen_height() - 30.0, 30.0, WHITE);

    let inventory = format!(
        "Resources: Rust {} | Gold {}",
        game.ship.scrap, game.ship.rare_metal
    );
    draw_text(&inventory, 20.0, screen_height() - 60.0, 25.0, GRAY);
}

pub fn render_menu(resources: &Resources) {
    draw_background(&resources.background);

    let time = get_time();
    let pulse = 1.0 + (time * 2.0).sin() as f32 * 0.05;

    let target_width = screen_width() * 0.5;
    let aspect_ratio = resources.logo.height() / resources.logo.width();
    let target_height = target_width * aspect_ratio;

    let logo_w = target_width * pulse;
    let logo_h = target_height * pulse;

    let logo_x = screen_width() / 2.0 - logo_w / 2.0;
    let logo_y = screen_height() / 2.0 - logo_h / 2.0 - 50.0;

    draw_texture_ex(
        &resources.logo,
        logo_x,
        logo_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(logo_w, logo_h)),
            ..Default::default()
        },
    );

    if (time * 3.0).sin() > 0.0 {
        draw_text_centered("Press [ENTER] to Start", logo_h / 2.0 + 20.0, 30, WHITE);
    }

    draw_text_centered(
        "ARROWS to move | SPACE to shoot",
        logo_h / 2.0 + 60.0,
        20,
        GRAY,
    );
}

pub fn render_briefing(mission: &Mission) {
    draw_text_centered(&format!("MISSION {}", mission.level_id), -100.0, 40, ORANGE);
    draw_text_centered(&mission.title, -50.0, 60, WHITE);
    draw_text_centered(&mission.description, 0.0, 25, GRAY);

    draw_text_centered("OBJECTIVES:", 20.0, 30, GRAY);

    let mut objectives = vec![format!("- Destroy {} Enemies", mission.target_kills)];
    if mission.target_scrap > 0 {
        objectives.push(format!("- Collect {} Rust Piles", mission.target_scrap));
    }
    if mission.target_rare_metal > 0 {
        objectives.push(format!("- Collect {} Gold", mission.target_rare_metal));
    }
    let obj_text = objectives.join("\n");
    draw_text_centered(&obj_text, 70.0, 30, WHITE);

    draw_text_centered("Press [SPACE] to Launch", 200.0, 30, GREEN);
}

pub fn render_mission_success(mission: &Mission) {
    draw_text_centered("MISSION COMPLETE!", -50.0, 50, GREEN);
    draw_text_centered(
        &format!("Level {} Cleared", mission.level_id),
        10.0,
        30,
        WHITE,
    );
    draw_text_centered("Press [ENTER] for Next Mission", 100.0, 30, YELLOW);
}

pub fn render_game_over(score: u32) {
    let high_score = load_score().high_score;
    draw_text_centered("GAME OVER", -40.0, 60, RED);
    draw_text_centered(&format!("Final Score: {score}"), 10.0, 40, WHITE);
    draw_text_centered(&format!("HIGH SCORE: {high_score}"), 60.0, 30, YELLOW);
}
