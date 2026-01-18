mod components;
mod draw;
mod systems;

use macroquad::prelude::*;
use macroquad::rand::gen_range;

// Re-exporting for convenience
use components::*;
use draw::*;
use systems::*;

// --- Constants ---
const ROTATION_SPEED: f32 = 200.0;
const ACCELERATION: f32 = 150.0;
const BULLET_SPEED: f32 = 400.0;
const BULLET_LIFETIME: f32 = 2.0;
const SHOOT_COOLDOWN: f32 = 0.3;

fn window_conf() -> Conf {
    Conf {
        window_title: "Rust in Space".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    #[allow(unused_assignments)]
    let mut high_score = 0;

    let mut state = GameState::Menu;

    let ship_body_tex = load_texture("assets/ship_body.png").await.unwrap();
    let ship_flame_tex = load_texture("assets/ship_flame.png").await.unwrap();
    ship_body_tex.set_filter(FilterMode::Nearest);
    ship_flame_tex.set_filter(FilterMode::Nearest);

    let logo_texture = load_texture("assets/logo.png").await.unwrap();
    logo_texture.set_filter(FilterMode::Nearest);

    let enemy_texture = load_texture("assets/enemy.png").await.unwrap();
    let background_texture = load_texture("assets/space_bg.png").await.unwrap();

    // Game entities
    let mut ship = create_ship();
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut enemy_bullets: Vec<EnemyBullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut enemy_ships: Vec<EnemyShip> = Vec::new();
    let mut score: u32 = 0;
    let mut powerups = Vec::new();

    // universe state
    let mut current_level_idx: u32 = 1;
    let mut current_mission = get_mission(1);

    // current mission state
    let mut mission_kills = 0;
    let mut mission_scrap_collected = 0;

    let mut enemy_spawn_timer = 0.0;

    loop {
        clear_background(BLACK);
        // draw the background first!
        draw_background(&background_texture);

        match state {
            GameState::Menu => {
                // 1. background
                draw_background(&background_texture);

                // 2. logo animation
                let time = get_time();
                let pulse = 1.0 + (time * 2.0).sin() as f32 * 0.05;

                let target_width = screen_width() * 0.5;

                // calculate the aspect ratio, so the image doesn't get squashed
                let aspect_ratio = logo_texture.height() / logo_texture.width();
                let target_height = target_width * aspect_ratio;

                // apply the pulse to the already fitted sizes
                let logo_w = target_width * pulse;
                let logo_h = target_height * pulse;
                // ---------------------------

                let logo_x = screen_width() / 2.0 - logo_w / 2.0;
                // move the logo up a bit above the center, so the text can fit below
                let logo_y = screen_height() / 2.0 - logo_h / 2.0 - 50.0;

                draw_texture_ex(
                    &logo_texture,
                    logo_x,
                    logo_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(logo_w, logo_h)),
                        ..Default::default()
                    },
                );

                // 3. Text
                if (time * 3.0).sin() > 0.0 {
                    // move the text down a bit below the logo
                    draw_text_centered("Press [ENTER] to Start", logo_h / 2.0 + 20.0, 30, WHITE);
                }

                draw_text_centered(
                    "ARROWS to move | SPACE to shoot",
                    logo_h / 2.0 + 60.0,
                    20,
                    GRAY,
                );

                if is_key_pressed(KeyCode::Enter) {
                    // ... game initialization ...
                    bullets.clear();
                    enemy_bullets.clear();
                    asteroids = (0..5).map(|_| Asteroid::new_large()).collect();
                    powerups.clear();
                    enemy_ships.clear();
                    score = 0;
                    current_level_idx = 1;
                    current_mission = get_mission(current_level_idx);
                    ship = create_ship(); // full reset of the ship
                    state = GameState::Briefing; // go to briefing, not to the game
                }
            }

            GameState::Briefing => {
                // draw the mission interface
                draw_text_centered(
                    &format!("MISSION {}", current_mission.level_id),
                    -100.0,
                    40,
                    ORANGE,
                );
                draw_text_centered(&current_mission.title, -50.0, 60, WHITE);
                draw_text_centered(&current_mission.description, 0.0, 25, GRAY);

                draw_text_centered("OBJECTIVES:", 20.0, 30, GRAY);

                let obj_text = format!(
                    "- Destroy {} Enemies\n- Collect {} Scrap",
                    current_mission.target_kills, current_mission.target_scrap
                );
                draw_text_centered(&obj_text, 70.0, 30, WHITE);

                draw_text_centered("Press [SPACE] to Launch", 200.0, 30, GREEN);

                if is_key_pressed(KeyCode::Space) {
                    // level initialization
                    bullets.clear();
                    enemy_bullets.clear();
                    enemy_ships.clear();
                    powerups.clear();

                    // spawn asteroids according to the mission configuration
                    asteroids = (0..current_mission.asteroid_count)
                        .map(|_| Asteroid::new_large())
                        .collect();

                    // reset the mission counters
                    mission_kills = 0;
                    mission_scrap_collected = 0;
                    enemy_spawn_timer = current_mission.enemy_spawn_interval;

                    // reset the ship position (but keep the upgrades if there are any)
                    ship.pos = vec2(screen_width() / 2.0, screen_height() / 2.0);
                    ship.vel = vec2(0.0, 0.0);

                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
                let dt = get_frame_time();

                // 1. Check the mission objectives
                if mission_kills >= current_mission.target_kills
                    && mission_scrap_collected >= current_mission.target_scrap
                {
                    state = GameState::MissionSuccess;
                }

                // 1. Timers & Spawning
                ship.shoot_timer -= dt;
                ship.rapid_fire_timer -= dt;
                enemy_spawn_timer -= dt;
                if enemy_spawn_timer <= 0.0 {
                    enemy_ships.push(EnemyShip::new());
                    enemy_spawn_timer = current_mission.enemy_spawn_interval;
                }

                // 2. Ship Movement & Input
                if is_key_down(KeyCode::Left) {
                    ship.rotation -= ROTATION_SPEED * dt;
                }
                if is_key_down(KeyCode::Right) {
                    ship.rotation += ROTATION_SPEED * dt;
                }

                let rotation_rad = ship.rotation.to_radians();
                let ship_dir = vec2(rotation_rad.cos(), rotation_rad.sin());

                let is_gas_pedal_down = is_key_down(KeyCode::Up);
                ship.engine.update(dt, is_gas_pedal_down);
                if ship.engine.current_thrust > 0.0 {
                    // thrust force depends on the current engine rotation
                    let thrust_force = ship.engine.current_thrust * ACCELERATION;
                    ship.vel += ship_dir * thrust_force * dt;
                }

                ship.pos += ship.vel * dt;
                wrap_around(&mut ship.pos);

                let current_cooldown = if ship.rapid_fire_timer > 0.0 {
                    SHOOT_COOLDOWN / 3.0
                } else {
                    SHOOT_COOLDOWN
                };
                if is_key_down(KeyCode::Space) && ship.shoot_timer <= 0.0 {
                    bullets.push(Bullet {
                        pos: ship.pos,
                        vel: ship_dir * BULLET_SPEED + ship.vel,
                        life_time: BULLET_LIFETIME,
                    });
                    ship.shoot_timer = current_cooldown;
                }

                // 3. Update Enemies & Powerups
                for e in enemy_ships.iter_mut() {
                    e.pos += e.vel * dt;
                    e.shoot_timer -= dt;

                    let diff = ship.pos - e.pos;
                    e.rotation = diff.y.atan2(diff.x);

                    if e.shoot_timer <= 0.0 {
                        let bullet_vel = vec2(e.rotation.cos(), e.rotation.sin()) * 250.0;

                        enemy_bullets.push(EnemyBullet {
                            pos: e.pos,
                            vel: bullet_vel,
                            life_time: 4.0,
                        });
                        e.shoot_timer = 2.0;
                    }
                }
                enemy_ships.retain(|e| e.pos.x > -100.0 && e.pos.x < screen_width() + 100.0);

                powerups.retain_mut(|p: &mut Powerup| {
                    if (ship.pos - p.pos).length() < p.radius + 15.0 {
                        match p.p_type {
                            PowerupType::Health => {
                                ship.lives += 1;
                                mission_scrap_collected += 1;
                            }
                            PowerupType::RapidFire => {
                                ship.rapid_fire_timer = 6.0;
                                mission_scrap_collected += 1;
                            }
                        }
                        false
                    } else {
                        true
                    }
                });

                // 4. Update Physics
                bullets.iter_mut().for_each(|b| {
                    b.pos += b.vel * dt;
                    b.life_time -= dt;
                });
                bullets.retain(|b| b.life_time > 0.0);
                enemy_bullets.iter_mut().for_each(|b| {
                    b.pos += b.vel * dt;
                    b.life_time -= dt;
                });
                enemy_bullets.retain(|b| b.life_time > 0.0);
                for a in asteroids.iter_mut() {
                    a.pos += a.vel * dt;
                    wrap_around(&mut a.pos);
                }

                // 5. Collision Logic (Condensed)
                let mut new_asteroids = Vec::new();
                bullets.retain(|b| {
                    let mut hit = false;
                    for i in (0..asteroids.len()).rev() {
                        if (b.pos - asteroids[i].pos).length() < asteroids[i].radius {
                            score += 100;
                            let old = asteroids.remove(i);
                            if gen_range(0, 10) == 0 {
                                powerups.push(Powerup {
                                    pos: old.pos,
                                    p_type: if gen_range(0, 2) == 0 {
                                        PowerupType::Health
                                    } else {
                                        PowerupType::RapidFire
                                    },
                                    radius: 12.0,
                                });
                            }
                            if old.radius > 15.0 {
                                new_asteroids.push(Asteroid::new_fragment(old.pos, old.radius));
                                new_asteroids.push(Asteroid::new_fragment(old.pos, old.radius));
                            }
                            hit = true;
                            break;
                        }
                    }

                    // bullet hits an enemy
                    enemy_ships.retain(|e| {
                        if (b.pos - e.pos).length() < 30.0 {
                            score += 500;
                            mission_kills += 1;
                            hit = true;
                            false
                        } else {
                            true
                        }
                    });
                    !hit
                });
                asteroids.extend(new_asteroids);

                // enemy bullet hits the player
                enemy_bullets.retain(|eb| {
                    if (eb.pos - ship.pos).length() < 20.0 {
                        if ship.take_damage(score) {
                            state = GameState::GameOver(score);
                        }
                        false // Remove bullet
                    } else {
                        true
                    }
                });

                for i in (0..asteroids.len()).rev() {
                    if (ship.pos - asteroids[i].pos).length() < asteroids[i].radius + 10.0 {
                        asteroids.remove(i);
                        if ship.take_damage(score) {
                            state = GameState::GameOver(score);
                        }
                        // break;
                    }
                }

                // 6. Rendering
                for p in &powerups {
                    let color = if p.p_type == PowerupType::Health {
                        GREEN
                    } else {
                        PURPLE
                    };
                    draw_circle_lines(p.pos.x, p.pos.y, p.radius, 2.0, color);
                }
                for b in &bullets {
                    draw_circle(b.pos.x, b.pos.y, 6.0, RED);
                }
                for b in &enemy_bullets {
                    draw_circle(b.pos.x, b.pos.y, 9.0, YELLOW);
                }
                for a in &asteroids {
                    draw_poly_lines(a.pos.x, a.pos.y, a.sides, a.radius, 0.0, 2.0, GRAY);
                }
                for e in &enemy_ships {
                    draw_enemy(e, &enemy_texture);
                }

                draw_ship(&ship, &ship_body_tex, &ship_flame_tex);

                draw_text(
                    &format!("SCORE: {score}  LIVES: {}", ship.lives),
                    20.0,
                    30.0,
                    30.0,
                    WHITE,
                );

                let status = format!(
                    "Kills: {}/{}  Scrap: {}/{}",
                    mission_kills,
                    current_mission.target_kills,
                    mission_scrap_collected,
                    current_mission.target_scrap
                );
                draw_text(&status, 20.0, screen_height() - 30.0, 30.0, WHITE);
            }

            GameState::MissionSuccess => {
                draw_text_centered("MISSION COMPLETE!", -50.0, 50, GREEN);
                draw_text_centered(
                    &format!("Level {} Cleared", current_mission.level_id),
                    10.0,
                    30,
                    WHITE,
                );

                // TODO: show bonus points and etc.

                draw_text_centered("Press [ENTER] for Next Mission", 100.0, 30, YELLOW);

                if is_key_pressed(KeyCode::Enter) {
                    current_level_idx += 1;
                    current_mission = get_mission(current_level_idx);
                    state = GameState::Briefing;
                }
            }

            GameState::GameOver(f_score) => {
                // Update high score from file in case it was just saved
                high_score = systems::load_score().high_score;
                draw_text_centered("GAME OVER", -40.0, 60, RED);
                draw_text_centered(&format!("Final Score: {f_score}"), 10.0, 40, WHITE);
                draw_text_centered(&format!("HIGH SCORE: {high_score}"), 60.0, 30, YELLOW);
                if is_key_pressed(KeyCode::Enter) {
                    state = GameState::Menu;
                }
            }
        }
        next_frame().await
    }
}

fn create_ship() -> Ship {
    Ship {
        pos: vec2(screen_width() / 2.0, screen_height() / 2.0),
        vel: vec2(0.0, 0.0),
        rotation: 0.0,
        lives: 10,
        shoot_timer: 0.0,
        rapid_fire_timer: 0.0,
        engine: Engine::basic(),
    }
}
