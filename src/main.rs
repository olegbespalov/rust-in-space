mod components;
mod draw;
mod game;
mod resources;
mod systems;

use macroquad::prelude::*;

use components::GameState;
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
    let resources = Resources::new().await;
    let mut game = Game::new();

    loop {
        clear_background(BLACK);
        draw_background(&resources.background);

        match state {
            GameState::Menu => {
                render_menu(&resources);

                if is_key_pressed(KeyCode::Enter) {
                    game.reset();
                    state = GameState::Briefing;
                }
            }

            GameState::Briefing => {
                render_briefing(&game.current_mission);

                if is_key_pressed(KeyCode::Space) {
                    game.start_mission();
                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
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

            GameState::MissionSuccess => {
                render_mission_success(&game.current_mission);

                if is_key_pressed(KeyCode::Enter) {
                    game.next_mission();
                    state = GameState::Briefing;
                }
            }

            GameState::GameOver(score) => {
                render_game_over(score);

                if is_key_pressed(KeyCode::Enter) {
                    state = GameState::Menu;
                }
            }
        }

        next_frame().await
    }
}
