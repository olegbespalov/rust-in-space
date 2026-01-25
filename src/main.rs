mod components;
mod draw;
mod game;
mod localization;
mod resources;
mod systems;

use macroquad::prelude::*;

use components::{GameState, MenuItem};
use draw::draw_background;
use game::*;
use resources::Resources;

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
    let mut state = GameState::Menu;
    let mut resources = Resources::new().await;
    let mut game = Game::new();

    loop {
        clear_background(BLACK);
        draw_background(&resources.background);

        match state {
            GameState::Menu => {
                render_menu(&game, &resources);

                // Menu navigation
                if is_key_pressed(KeyCode::Up) {
                    game.menu_selection = game.menu_selection.prev();
                }
                if is_key_pressed(KeyCode::Down) {
                    game.menu_selection = game.menu_selection.next();
                }

                // Handle actions based on selected menu item
                match game.menu_selection {
                    MenuItem::Start => {
                        if is_key_pressed(KeyCode::Enter) {
                            game.reset();
                            state = GameState::Briefing;
                        }
                    }
                    MenuItem::Difficulty => {
                        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
                            game.cycle_difficulty();
                        }
                    }
                    MenuItem::Language => {
                        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
                            resources.lang.cycle_lang();
                        }
                    }
                }
            }

            GameState::Briefing => {
                render_briefing(&game.current_mission, &resources);

                if is_key_pressed(KeyCode::Space) {
                    game.start_mission();
                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
                // Check for pause
                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Paused;
                } else {
                    let dt = get_frame_time();

                    if game.is_mission_complete() {
                        state = GameState::MissionSuccess;
                    }

                    update_timers(&mut game, dt);
                    update_ship_movement(&mut game, dt);
                    update_ship_shooting(&mut game);
                    update_enemies(&mut game, dt);
                    update_loot(&mut game, dt);
                    update_physics(&mut game, dt);

                    if update_collisions(&mut game) {
                        state = GameState::GameOver(game.score);
                    }

                    render_game(&game, &resources);
                }
            }

            GameState::Paused => {
                // Render the game in paused state (frozen frame)
                render_game(&game, &resources);
                render_pause(&resources);

                // Check for unpause
                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Playing;
                }
            }

            GameState::MissionSuccess => {
                render_mission_success(&game.current_mission, &resources);

                if is_key_pressed(KeyCode::Enter) {
                    game.next_mission();
                    state = GameState::Briefing;
                }
            }

            GameState::GameOver(score) => {
                render_game_over(score, &resources);

                if is_key_pressed(KeyCode::Enter) {
                    state = GameState::Menu;
                }
            }
        }

        next_frame().await
    }
}
