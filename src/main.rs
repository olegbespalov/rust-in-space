mod components;
mod draw;
mod resources;
mod systems;

use macroquad::prelude::*;

// Re-exporting for convenience
use components::*;
use draw::*;
use resources::Resources;
use systems::*;

// --- Constants ---
const ROTATION_SPEED: f32 = 200.0;
const ACCELERATION: f32 = 150.0;
const BULLET_SPEED: f32 = 400.0;
const BULLET_LIFETIME: f32 = 2.0;
const SHOOT_COOLDOWN: f32 = 0.3;
const PLAYER_BULLET_DAMAGE: f32 = 10.0;
const ENEMY_BULLET_DAMAGE: f32 = 15.0;
const BASE_ASTEROID_DAMAGE: f32 = 5.0; // Base damage per 10 units of radius

fn window_conf() -> Conf {
    Conf {
        window_title: "Rust in Space".to_owned(),
        window_width: 1280 * 2,
        window_height: 720 * 2,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    #[allow(unused_assignments)]
    let mut high_score = 0;

    let mut state = GameState::Menu;

    let resources = Resources::new().await;

    // Game entities
    let mut ship = create_ship();
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut enemy_ships: Vec<EnemyShip> = Vec::new();
    let mut score: u32 = 0;
    let mut loot_items: Vec<LootItem> = Vec::new();

    // universe state
    let mut current_level_idx: u32 = 1;
    let mut current_mission = get_mission(1);

    // current mission state
    let mut mission_kills = 0;
    let mut mission_scrap_collected = 0;
    let mut mission_rare_metal_collected = 0;

    let mut enemy_spawn_timer = 0.0;

    loop {
        clear_background(BLACK);
        // draw the background first!
        draw_background(&resources.background);

        match state {
            GameState::Menu => {
                // 1. background
                draw_background(&resources.background);

                // 2. logo animation
                let time = get_time();
                let pulse = 1.0 + (time * 2.0).sin() as f32 * 0.05;

                let target_width = screen_width() * 0.5;

                // calculate the aspect ratio, so the image doesn't get squashed
                let aspect_ratio = resources.logo.height() / resources.logo.width();
                let target_height = target_width * aspect_ratio;

                // apply the pulse to the already fitted sizes
                let logo_w = target_width * pulse;
                let logo_h = target_height * pulse;
                // ---------------------------

                let logo_x = screen_width() / 2.0 - logo_w / 2.0;
                // move the logo up a bit above the center, so the text can fit below
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
                    asteroids = (0..5).map(|_| Asteroid::new_large()).collect();
                    loot_items.clear();
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

                let mut objectives = vec![format!(
                    "- Destroy {} Enemies",
                    current_mission.target_kills
                )];
                if current_mission.target_scrap > 0 {
                    objectives.push(format!(
                        "- Collect {} Rust Piles",
                        current_mission.target_scrap
                    ));
                }
                if current_mission.target_rare_metal > 0 {
                    objectives.push(format!(
                        "- Collect {} Gold",
                        current_mission.target_rare_metal
                    ));
                }
                let obj_text = objectives.join("\n");
                draw_text_centered(&obj_text, 70.0, 30, WHITE);

                draw_text_centered("Press [SPACE] to Launch", 200.0, 30, GREEN);

                if is_key_pressed(KeyCode::Space) {
                    // level initialization
                    bullets.clear();
                    enemy_ships.clear();
                    loot_items.clear();

                    // spawn asteroids according to the mission configuration
                    asteroids = (0..current_mission.asteroid_count)
                        .map(|_| Asteroid::new_large())
                        .collect();

                    // reset the mission counters
                    mission_kills = 0;
                    mission_scrap_collected = 0;
                    mission_rare_metal_collected = 0;
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
                    && mission_rare_metal_collected >= current_mission.target_rare_metal
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
                        style: BulletStyle::Player,
                        damage: PLAYER_BULLET_DAMAGE,
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

                        bullets.push(Bullet {
                            pos: e.pos,
                            vel: bullet_vel,
                            life_time: 4.0,
                            style: BulletStyle::Enemy,
                            damage: ENEMY_BULLET_DAMAGE,
                        });
                        e.shoot_timer = 2.0;
                    }
                }
                enemy_ships.retain(|e| e.pos.x > -100.0 && e.pos.x < screen_width() + 100.0);

                // 2. UPDATE LOOT (Magnet and Collection)
                let mut items_to_remove = Vec::new();
                for (i, item) in loot_items.iter_mut().enumerate() {
                    // Animation of slowing down the spread (initial explosion velocity)
                    item.vel *= 0.95;
                    item.pos += item.vel * dt;

                    // Slow constant drift in space (super slow floating)
                    item.pos += item.drift_vel * dt;
                    wrap_around(&mut item.pos);

                    // Rotate loot items in space with random speed and direction
                    item.rotation += item.rotation_speed * dt;
                    // Wrap rotation to keep it in 0-2π range
                    if item.rotation > std::f32::consts::PI * 2.0 {
                        item.rotation -= std::f32::consts::PI * 2.0;
                    } else if item.rotation < 0.0 {
                        item.rotation += std::f32::consts::PI * 2.0;
                    }

                    let dist_to_ship = (ship.pos - item.pos).length();

                    // Magnet: if close, fly to the player
                    if dist_to_ship < 150.0 {
                        item.magnet_active = true;
                    }

                    if item.magnet_active {
                        let dir = (ship.pos - item.pos).normalize();
                        let magnet_speed = 300.0;
                        item.pos += dir * magnet_speed * dt;
                    }

                    // Collection
                    if dist_to_ship < (72.0 / 2.0 + item.radius) {
                        match item.item_type {
                            LootType::Scrap(amount) => {
                                ship.scrap += amount;
                                mission_scrap_collected += amount; // Count for mission objective
                                                                   // play_sound_pickup();
                            }
                            LootType::RareMetal(amount) => {
                                ship.rare_metal += amount;
                                mission_rare_metal_collected += amount; // Count for mission objective
                                                                        // play_sound_rare();
                            }
                            LootType::HealthPack(hp) => {
                                ship.heal(hp as f32);
                                // Health packs don't count as resources
                            }
                            LootType::WeaponBoost => {
                                ship.rapid_fire_timer = 10.0;
                                // Weapon boosts don't count as resources
                            }
                        }
                        items_to_remove.push(i);
                    }
                }
                // Remove collected items (in reverse order to maintain indices)
                for &i in items_to_remove.iter().rev() {
                    loot_items.remove(i);
                }

                // 4. Update Physics
                bullets.iter_mut().for_each(|b| {
                    b.pos += b.vel * dt;
                    b.life_time -= dt;
                });
                bullets.retain(|b| b.life_time > 0.0);
                for a in asteroids.iter_mut() {
                    a.pos += a.vel * dt;
                    wrap_around(&mut a.pos);
                }

                // 5. Collision Logic (Condensed)
                let mut new_asteroids = Vec::new();
                bullets.retain(|b| {
                    // Only player bullets can hit asteroids and enemies
                    if b.style != BulletStyle::Player {
                        return true;
                    }

                    let mut hit = false;
                    for i in (0..asteroids.len()).rev() {
                        if (b.pos - asteroids[i].pos).length() < asteroids[i].radius {
                            score += 100;
                            let is_rare = asteroids[i].is_rare;
                            let asteroid_pos = asteroids[i].pos;

                            // Rare asteroids always drop loot
                            if is_rare {
                                if let Some(loot) =
                                    generate_loot(asteroid_pos, LootSource::RareAsteroid)
                                {
                                    loot_items.push(loot);
                                }
                            } else if let Some(loot) =
                                generate_loot(asteroid_pos, LootSource::Asteroid)
                            {
                                loot_items.push(loot);
                            }
                            let old = asteroids.remove(i);

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
                            if let Some(loot) = generate_loot(e.pos, LootSource::EnemySmall) {
                                loot_items.push(loot);
                            }
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
                bullets.retain(|b| {
                    if b.style == BulletStyle::Enemy && (b.pos - ship.pos).length() < 20.0 {
                        if ship.take_damage(b.damage, score) {
                            state = GameState::GameOver(score);
                        }
                        false // Remove bullet
                    } else {
                        true
                    }
                });

                for i in (0..asteroids.len()).rev() {
                    if (ship.pos - asteroids[i].pos).length() < asteroids[i].radius + 10.0 {
                        // Calculate damage based on asteroid size
                        // Bigger asteroids deal more damage
                        let asteroid_damage = (asteroids[i].radius / 10.0) * BASE_ASTEROID_DAMAGE;
                        asteroids.remove(i);
                        if ship.take_damage(asteroid_damage, score) {
                            state = GameState::GameOver(score);
                        }
                        // break;
                    }
                }

                // 6. Rendering
                for item in &loot_items {
                    draw_loot(item, &resources);
                }
                for b in &bullets {
                    let texture = match b.style {
                        BulletStyle::Player => &resources.bullet,
                        BulletStyle::Enemy => &resources.enemy_bullet,
                    };

                    // Calculate rotation from velocity direction
                    let rotation = b.vel.y.atan2(b.vel.x) + std::f32::consts::FRAC_PI_2;

                    // Determine bullet size based on style
                    let size = match b.style {
                        BulletStyle::Player => 12.0,
                        BulletStyle::Enemy => 18.0,
                    };

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
                for a in &asteroids {
                    draw_asteroid(a, &resources);
                }
                for e in &enemy_ships {
                    draw_enemy(e, &resources);
                }

                draw_ship(&ship, &resources.ship_body, &resources.ship_flame);

                draw_text(
                    &format!(
                        "SCORE: {score}  HP: {:.0}/{:.0}",
                        ship.health, ship.max_health
                    ),
                    20.0,
                    30.0,
                    30.0,
                    WHITE,
                );

                let status = format!(
                    "Kills: {}/{}  Rust: {}/{}  Gold: {}/{}",
                    mission_kills,
                    current_mission.target_kills,
                    mission_scrap_collected,
                    current_mission.target_scrap,
                    mission_rare_metal_collected,
                    current_mission.target_rare_metal
                );
                draw_text(&status, 20.0, screen_height() - 30.0, 30.0, WHITE);

                // Display total resources in inventory
                let inventory =
                    format!("Resources: Rust {} | Gold {}", ship.scrap, ship.rare_metal);
                draw_text(&inventory, 20.0, screen_height() - 60.0, 25.0, GRAY);
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
        health: 100.0,
        max_health: 100.0,
        shoot_timer: 0.0,
        rapid_fire_timer: 0.0,
        engine: Engine::basic(),
        scrap: 0,
        rare_metal: 0,
    }
}
