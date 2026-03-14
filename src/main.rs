mod audio;
mod components;
mod draw;
mod game;
mod localization;
mod resources;
mod systems;

use macroquad::prelude::*;

use audio::{AudioState, MusicTrack};
use components::{GameState, MenuItem};
use draw::draw_background;
use game::*;
use resources::Resources;
use systems::{load_score, save_audio_settings};

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
    let save_data = load_score();
    let mut audio = AudioState::new(save_data.audio);
    let mut audio_settings_dirty = false;
    let mut audio_settings_save_timer = 0.0f32;
    audio.play_music(&resources.audio, MusicTrack::Menu);
    let mut game = Game::new();

    loop {
        let frame_dt = get_frame_time();
        if audio_settings_save_timer > 0.0 {
            audio_settings_save_timer -= frame_dt;
        }

        clear_background(BLACK);
        draw_background(&resources.background);

        match state {
            GameState::Menu => {
                audio.play_music(&resources.audio, MusicTrack::Menu);
                render_menu(&game, &resources, audio.settings());

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
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::MasterVolume => {
                        if is_key_pressed(KeyCode::Left) {
                            audio.adjust_master_volume(&resources.audio, -0.05);
                            audio_settings_dirty = true;
                            audio_settings_save_timer = 0.25;
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                        if is_key_pressed(KeyCode::Right) {
                            audio.adjust_master_volume(&resources.audio, 0.05);
                            audio_settings_dirty = true;
                            audio_settings_save_timer = 0.25;
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::MusicVolume => {
                        if is_key_pressed(KeyCode::Left) {
                            audio.adjust_music_volume(&resources.audio, -0.05);
                            audio_settings_dirty = true;
                            audio_settings_save_timer = 0.25;
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                        if is_key_pressed(KeyCode::Right) {
                            audio.adjust_music_volume(&resources.audio, 0.05);
                            audio_settings_dirty = true;
                            audio_settings_save_timer = 0.25;
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::SfxVolume => {
                        if is_key_pressed(KeyCode::Left) {
                            audio.adjust_sfx_volume(&resources.audio, -0.05);
                            audio_settings_dirty = true;
                            audio_settings_save_timer = 0.25;
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                        if is_key_pressed(KeyCode::Right) {
                            audio.adjust_sfx_volume(&resources.audio, 0.05);
                            audio_settings_dirty = true;
                            audio_settings_save_timer = 0.25;
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::AudioMute => {
                        if is_key_pressed(KeyCode::Left)
                            || is_key_pressed(KeyCode::Right)
                            || is_key_pressed(KeyCode::Enter)
                        {
                            audio.toggle_mute(&resources.audio);
                            audio_settings_dirty = true;
                            audio_settings_save_timer = 0.25;
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

                // Check for unpause
                if is_key_pressed(KeyCode::Escape) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    audio.set_paused(&resources.audio, false);
                    state = GameState::Playing;
                }
            }

            GameState::MissionSuccess => {
                audio.set_paused(&resources.audio, false);
                audio.play_music(&resources.audio, MusicTrack::Menu);
                render_mission_success(&game.current_mission, &resources);

                if is_key_pressed(KeyCode::Enter) {
                    game.next_mission();
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    state = GameState::Briefing;
                }
            }

            GameState::GameOver(score) => {
                audio.set_paused(&resources.audio, false);
                audio.play_music(&resources.audio, MusicTrack::Menu);
                render_game_over(score, &resources);

                if is_key_pressed(KeyCode::Enter) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    state = GameState::Menu;
                }
            }
        }

        if audio_settings_dirty && audio_settings_save_timer <= 0.0 {
            save_audio_settings(audio.settings().clone());
            audio_settings_dirty = false;
        }

        next_frame().await
    }
}
