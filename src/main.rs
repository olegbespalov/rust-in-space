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
// const ENEMY_SPEED: f32 = 120.0;
const ENEMY_SHOOT_INTERVAL: f32 = 1.5;

#[macroquad::main("Rust in Space")]
async fn main() {
    #[allow(unused_assignments)]
    let mut high_score = 0;

    let mut state = GameState::Menu;

    let ship_texture = load_texture("assets/ship_v1.png").await.unwrap();
    let enemy_texture = load_texture("assets/enemy_1.png").await.unwrap();

    // Game entities
    let mut ship = create_ship();
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut enemy_bullets: Vec<EnemyBullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut powerups: Vec<Powerup> = Vec::new();
    let mut enemy_ships: Vec<EnemyShip> = Vec::new();
    let mut score: u32 = 0;
    let mut enemy_spawn_timer = 10.0;

    loop {
        clear_background(BLACK);
        let dt = get_frame_time();

        match state {
            GameState::Menu => {
                draw_text_centered("RUST IN SPACE", -40.0, 60, WHITE);
                draw_text_centered("Press [ENTER] to Start", 20.0, 30, YELLOW);
                if is_key_pressed(KeyCode::Enter) {
                    ship = create_ship();
                    bullets.clear();
                    enemy_bullets.clear();
                    asteroids = (0..5).map(|_| Asteroid::new_large()).collect();
                    powerups.clear();
                    enemy_ships.clear();
                    score = 0;
                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
                // 1. Timers & Spawning
                ship.shoot_timer -= dt;
                ship.rapid_fire_timer -= dt;
                enemy_spawn_timer -= dt;
                if enemy_spawn_timer <= 0.0 {
                    enemy_ships.push(EnemyShip::new());
                    enemy_spawn_timer = 12.0;
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

                if is_key_down(KeyCode::Up) {
                    ship.vel += ship_dir * ACCELERATION * dt;
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
                    if e.shoot_timer <= 0.0 {
                        let dir = (ship.pos - e.pos).normalize();
                        enemy_bullets.push(EnemyBullet {
                            pos: e.pos,
                            vel: dir * 250.0,
                            life_time: 4.0,
                        });
                        e.shoot_timer = ENEMY_SHOOT_INTERVAL;
                    }
                }
                enemy_ships.retain(|e| e.pos.x > -100.0 && e.pos.x < screen_width() + 100.0);

                powerups.retain_mut(|p| {
                    if (ship.pos - p.pos).length() < p.radius + 15.0 {
                        match p.p_type {
                            PowerupType::Health => ship.lives += 1,
                            PowerupType::RapidFire => ship.rapid_fire_timer = 6.0,
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
                    enemy_ships.retain(|e| {
                        if (b.pos - e.pos).length() < 25.0 {
                            score += 500;
                            hit = true;
                            false
                        } else {
                            true
                        }
                    });
                    !hit
                });
                asteroids.extend(new_asteroids);

                // Check Ship damage
                enemy_bullets.retain(|eb| {
                    if (eb.pos - ship.pos).length() < 15.0 {
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
                        // break; // Optional: break if you want only one hit per frame
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
                    draw_circle(b.pos.x, b.pos.y, 2.0, RED);
                }
                for b in &enemy_bullets {
                    draw_circle(b.pos.x, b.pos.y, 3.0, YELLOW);
                }
                for a in &asteroids {
                    draw_poly_lines(a.pos.x, a.pos.y, a.sides, a.radius, 0.0, 2.0, GRAY);
                }
                for e in &enemy_ships {
                    draw_enemy(e, &enemy_texture);
                }

                draw_ship(&ship, &ship_texture);

                draw_text(
                    &format!("SCORE: {score}  LIVES: {}", ship.lives),
                    20.0,
                    30.0,
                    30.0,
                    WHITE,
                );
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
        lives: 3,
        shoot_timer: 0.0,
        rapid_fire_timer: 0.0,
    }
}
