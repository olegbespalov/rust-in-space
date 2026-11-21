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
use components::{GameState, MenuItem, ShipKind};
use draw::*;
use game::*;
use localization::Language;
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
    resources.lang.current_lang = saved.language.unwrap_or_else(Language::from_system_locale);
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

                // Menu navigation
                if is_key_pressed(KeyCode::Up) {
                    game.menu_selection = game.menu_selection.prev();
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }
                if is_key_pressed(KeyCode::Down) {
                    game.menu_selection = game.menu_selection.next();
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }

                let click = render_menu(&game, saved.high_score, audio.settings(), &resources);
                if let Some(click) = click {
                    let item = click.item();
                    if matches!(click, MenuClick::Select(_)) && item != game.menu_selection {
                        audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                    }
                    game.menu_selection = item;
                }
                let activate = matches!(click, Some(MenuClick::Activate(_)));
                let enter = click.is_none() && is_key_pressed(KeyCode::Enter);
                let left = click.is_none() && is_key_pressed(KeyCode::Left);
                let right = click.is_none() && is_key_pressed(KeyCode::Right);

                match game.menu_selection {
                    MenuItem::Start => {
                        if enter || activate {
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                            state = GameState::ShipSelection(ShipKind::Alpha);
                        }
                    }
                    MenuItem::Difficulty => {
                        if left || right || activate {
                            game.cycle_difficulty();
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::Language => {
                        if left || right || activate {
                            resources.lang.cycle_lang();
                            saved.language = Some(resources.lang.current_lang);
                            save_timer = Some(0.0);
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                        }
                    }
                    MenuItem::AudioMute => {
                        if left || right || enter || activate {
                            audio.toggle_mute(&resources.audio);
                            save_timer = Some(0.25);
                        }
                    }
                    item @ (MenuItem::MasterVolume
                    | MenuItem::MusicVolume
                    | MenuItem::SfxVolume) => {
                        let decrease = left || matches!(click, Some(MenuClick::DecreaseVolume(_)));
                        let increase = right || matches!(click, Some(MenuClick::IncreaseVolume(_)));
                        let adjust_volume = match item {
                            MenuItem::MasterVolume => AudioState::adjust_master_volume,
                            MenuItem::MusicVolume => AudioState::adjust_music_volume,
                            MenuItem::SfxVolume => AudioState::adjust_sfx_volume,
                            _ => unreachable!(),
                        };
                        for (pressed, delta) in [(decrease, -0.05), (increase, 0.05)] {
                            if pressed {
                                adjust_volume(&mut audio, &resources.audio, delta);
                                save_timer = Some(0.25);
                                audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                            }
                        }
                    }
                }
            }

            GameState::ShipSelection(previous) => {
                let mut selected_ship = previous;
                if is_key_pressed(KeyCode::Left) {
                    selected_ship = ShipKind::Alpha;
                } else if is_key_pressed(KeyCode::Right)
                    && resources.ship_model(ShipKind::Delta).is_some()
                {
                    selected_ship = ShipKind::Delta;
                }
                let click = render_ship_selection(selected_ship, &resources);
                if let Some(ShipSelectionClick::Select(kind)) = click {
                    selected_ship = kind;
                }
                if selected_ship != previous {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                    state = GameState::ShipSelection(selected_ship);
                }

                if matches!(click, Some(ShipSelectionClick::Back))
                    || is_key_pressed(KeyCode::Escape)
                {
                    state = GameState::Menu;
                } else if matches!(click, Some(ShipSelectionClick::Confirm))
                    || is_key_pressed(KeyCode::Enter)
                {
                    game.reset(selected_ship);
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    state = GameState::Briefing;
                }
            }

            GameState::Briefing => {
                audio.play_music(&resources.audio, MusicTrack::Menu);
                let launch = render_briefing(&game.current_mission, &resources);

                if launch || is_key_pressed(KeyCode::Space) {
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
                    let mut ship_destroyed = update_loot(&mut game, dt);
                    if !ship_destroyed {
                        update_physics(&mut game, dt);
                        ship_destroyed = update_collisions(&mut game);
                    }

                    if ship_destroyed {
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
                let (resume, main_menu) = render_pause(&resources);

                if resume || (!main_menu && is_key_pressed(KeyCode::Enter)) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    audio.set_paused(&resources.audio, false);
                    state = GameState::Playing;
                } else if main_menu || is_key_pressed(KeyCode::Escape) {
                    audio.set_paused(&resources.audio, false);
                    state = GameState::Menu;
                }
            }

            GameState::MissionSuccess => {
                audio.set_paused(&resources.audio, false);
                audio.play_music(&resources.audio, MusicTrack::Menu);
                let upgrades = render_mission_success(&game.current_mission, &resources);

                if upgrades || is_key_pressed(KeyCode::Enter) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    game.upgrade_selection = 0;
                    state = GameState::UpgradeBay;
                }
            }

            GameState::UpgradeBay => {
                if is_key_pressed(KeyCode::Up) {
                    game.prev_upgrade_selection();
                }
                if is_key_pressed(KeyCode::Down) {
                    game.next_upgrade_selection();
                }

                let clicked = render_upgrade_bay(&game, &resources);
                if let Some(index) = clicked {
                    game.upgrade_selection = index;
                }

                if clicked.is_some() || is_key_pressed(KeyCode::Enter) {
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
                let main_menu = render_game_over(score, saved.high_score, &resources);

                if main_menu || is_key_pressed(KeyCode::Enter) {
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
