mod audio;
mod components;
mod draw;
mod game;
mod localization;
mod model_sprite;
mod resources;
mod ship_model;
mod systems;

use macroquad::prelude::*;

use audio::{AudioState, MusicTrack};
use components::{GameState, MenuItem};
use draw::*;
use game::*;
use resources::Resources;
use systems::{load_save, save_data, save_file};

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

fn window_conf() -> macroquad::conf::Conf {
    let window = Conf {
        window_title: "Rust in Space".to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        high_dpi: true,
        #[cfg(target_os = "linux")]
        icon: Some(macroquad::miniquad::conf::Icon {
            small: *include_bytes!(concat!(env!("OUT_DIR"), "/icon-16.rgba")),
            medium: *include_bytes!(concat!(env!("OUT_DIR"), "/icon-32.rgba")),
            big: *include_bytes!(concat!(env!("OUT_DIR"), "/icon-64.rgba")),
        }),
        #[cfg(target_os = "macos")]
        icon: None,
        platform: macroquad::miniquad::conf::Platform {
            linux_wm_class: "io.github.olegbespalov.RustInSpace",
            ..Default::default()
        },
        ..Default::default()
    };
    let mut config = macroquad::conf::Conf::from(window);
    config.draw_call_index_capacity = ship_model::MODEL_MESH_INDEX_CAPACITY;
    config
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = GameState::Menu;
    let save_path = save_file();
    let mut saved = load_save(save_path.as_deref());
    let mut resources = Resources::new().await;
    resources.lang.current_lang = saved.language;
    let mut audio = AudioState::new(saved.audio.clone());
    let mut save_timer = None;
    audio.play_music(&resources.audio, MusicTrack::Menu);
    let mut game = Game::new();

    // XWayland can leave the window at 4×4 after the initial configure.
    #[cfg(target_os = "linux")]
    let mut startup_resize_pending = true;
    #[cfg(target_os = "linux")]
    let startup_resize_deadline = get_time() + 2.0;

    loop {
        #[cfg(target_os = "linux")]
        if startup_resize_pending
            && get_time() < startup_resize_deadline
            && (screen_width() <= 16.0 || screen_height() <= 16.0)
        {
            request_new_screen_size(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32);
            startup_resize_pending = false;
        }

        let frame_dt = get_frame_time();
        if let Some(remaining) = &mut save_timer {
            *remaining -= frame_dt;
        }

        clear_background(BLACK);
        draw_background(&resources.background);

        match state {
            GameState::Menu => {
                audio.play_music(&resources.audio, MusicTrack::Menu);

                render_menu(&game, &resources);

                // Menu navigation
                if is_key_pressed(KeyCode::Up) {
                    game.menu_selection = game.menu_selection.prev();
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }
                if is_key_pressed(KeyCode::Down) {
                    game.menu_selection = game.menu_selection.next();
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }

                // Handle actions based on selected menu item
                match game.menu_selection {
                    MenuItem::Start => {
                        if is_key_pressed(KeyCode::Enter) {
                            game.reset();
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                            state = GameState::Briefing;
                        }
                    }
                    MenuItem::Difficulty => {
                        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
                            game.cycle_difficulty();
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::Language => {
                        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::Right) {
                            resources.lang.cycle_lang();
                            saved.language = resources.lang.current_lang;
                            save_timer = Some(0.0);
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::MasterVolume => {
                        if is_key_pressed(KeyCode::Left) {
                            audio.adjust_master_volume(&resources.audio, -0.05);
                            save_timer = Some(0.25);
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                        if is_key_pressed(KeyCode::Right) {
                            audio.adjust_master_volume(&resources.audio, 0.05);
                            save_timer = Some(0.25);
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::MusicVolume => {
                        if is_key_pressed(KeyCode::Left) {
                            audio.adjust_music_volume(&resources.audio, -0.05);
                            save_timer = Some(0.25);
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                        if is_key_pressed(KeyCode::Right) {
                            audio.adjust_music_volume(&resources.audio, 0.05);
                            save_timer = Some(0.25);
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::SfxVolume => {
                        if is_key_pressed(KeyCode::Left) {
                            audio.adjust_sfx_volume(&resources.audio, -0.05);
                            save_timer = Some(0.25);
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                        if is_key_pressed(KeyCode::Right) {
                            audio.adjust_sfx_volume(&resources.audio, 0.05);
                            save_timer = Some(0.25);
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::AudioMute => {
                        if is_key_pressed(KeyCode::Left)
                            || is_key_pressed(KeyCode::Right)
                            || is_key_pressed(KeyCode::Enter)
                        {
                            audio.toggle_mute(&resources.audio);
                            save_timer = Some(0.25);
                        }
                    }
                }
            }

            GameState::Briefing => {
                audio.play_music(&resources.audio, MusicTrack::Menu);
                render_briefing(&game.current_mission, &resources);

                if is_key_pressed(KeyCode::Space) {
                    game.start_mission();
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    audio.play_music(&resources.audio, MusicTrack::Gameplay);
                    state = GameState::Playing;
                }
            }

            GameState::Playing => {
                // Check for pause
                if is_key_pressed(KeyCode::Escape) {
                    audio.set_paused(&resources.audio, true);
                    state = GameState::Paused;
                } else {
                    let dt = frame_dt;
                    audio.play_music(&resources.audio, MusicTrack::Gameplay);
                    audio.set_engine_active(
                        &resources.audio,
                        is_key_down(KeyCode::Up),
                        game.ship.engine.current_thrust,
                    );

                    update_timers(&mut game, dt);
                    update_ship_movement(&mut game, dt);
                    update_ship_shooting(&mut game);
                    update_enemies(&mut game, dt);
                    update_boss(&mut game, dt);
                    update_loot(&mut game, dt);
                    update_physics(&mut game, dt);

                    if update_collisions(&mut game) {
                        audio.set_engine_active(&resources.audio, false, 0.0);
                        audio.play_sfx(&resources.audio, audio::AudioCue::GameOver);
                        state = GameState::GameOver(game.score);
                    } else if game.is_mission_complete() {
                        audio.set_engine_active(&resources.audio, false, 0.0);
                        audio.play_sfx(&resources.audio, audio::AudioCue::MissionSuccess);
                        state = GameState::MissionSuccess;
                    }

                    if game.score > saved.high_score {
                        saved.high_score = game.score;
                        save_timer = Some(0.0);
                    }

                    for cue in game.drain_audio_cues() {
                        audio.play_sfx(&resources.audio, cue);
                    }

                    if matches!(state, GameState::Playing) {
                        render_game(&game, &resources);
                    }
                }
            }

            GameState::Paused => {
                // Render the game in paused state (frozen frame)
                render_game(&game, &resources);
                render_pause(&resources);

                // ENTER resumes the mission, ESC exits to main menu
                if is_key_pressed(KeyCode::Enter) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    audio.set_paused(&resources.audio, false);
                    state = GameState::Playing;
                } else if is_key_pressed(KeyCode::Escape) {
                    audio.set_paused(&resources.audio, false);
                    state = GameState::Menu;
                }
            }

            GameState::MissionSuccess => {
                audio.set_paused(&resources.audio, false);
                audio.play_music(&resources.audio, MusicTrack::Menu);
                render_mission_success(&game.current_mission, &resources);

                if is_key_pressed(KeyCode::Enter) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    game.upgrade_selection = 0;
                    state = GameState::UpgradeBay;
                }
            }

            GameState::UpgradeBay => {
                render_upgrade_bay(&game, &resources);

                if is_key_pressed(KeyCode::Up) {
                    game.prev_upgrade_selection();
                }
                if is_key_pressed(KeyCode::Down) {
                    game.next_upgrade_selection();
                }

                if is_key_pressed(KeyCode::Enter) {
                    if game.is_continue_selected() {
                        game.next_mission();
                        audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                        state = GameState::Briefing;
                    } else if let Some(id) = game.selected_upgrade() {
                        let _ = game.buy_upgrade(id);
                    }
                }
            }

            GameState::GameOver(score) => {
                audio.set_paused(&resources.audio, false);
                audio.play_music(&resources.audio, MusicTrack::Menu);
                render_game_over(score, saved.high_score, &resources);

                if is_key_pressed(KeyCode::Enter) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    state = GameState::Menu;
                }
            }
        }

        if save_timer.is_some_and(|remaining| remaining <= 0.0) {
            if let Some(path) = save_path.as_deref() {
                saved.audio = audio.settings().clone();
                save_data(path, &saved);
            }
            save_timer = None;
        }

        next_frame().await
    }
}
