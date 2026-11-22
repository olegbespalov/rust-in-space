mod audio;
mod components;
mod draw;
mod game;
mod lan;
mod localization;
mod model_sprite;
mod resources;
mod ship_model;
mod systems;

use macroquad::prelude::*;

use audio::{AudioState, MusicTrack};
use components::{GameState, MenuItem, ShipKind, DEFAULT_PLAYER_NAME};
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
    let save_path = save_file();
    let mut saved = load_save(save_path.as_deref());
    let mut state = if saved
        .player_name
        .as_deref()
        .is_none_or(|name| name.trim().is_empty())
    {
        GameState::PlayerName(String::new())
    } else {
        GameState::Menu
    };
    let mut resources = Resources::new().await;
    resources.lang.current_lang = saved.language.unwrap_or_else(Language::from_system_locale);
    let mut audio = AudioState::new(saved.audio.clone());
    let mut save_timer = None;
    audio.play_music(&resources.audio, MusicTrack::Menu);
    let mut game = Game::new(vec2(screen_width(), screen_height()));
    let mut upgrade_selection = UpgradeSelection::default();

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
        if !matches!(state, GameState::Lan(_)) {
            draw_background(&resources.background);
        }

        let player_name = saved.player_name.as_deref().unwrap_or(DEFAULT_PLAYER_NAME);
        match state {
            GameState::PlayerName(ref mut draft) => {
                if is_key_pressed(KeyCode::Backspace) {
                    draft.pop();
                }
                while let Some(character) = get_char_pressed() {
                    if !character.is_control() {
                        draft.push(character);
                    }
                }
                if render_player_name(draft, &resources) || is_key_pressed(KeyCode::Enter) {
                    let name = draft.trim();
                    saved.player_name = Some(if name.is_empty() {
                        DEFAULT_PLAYER_NAME.to_owned()
                    } else {
                        name.to_owned()
                    });
                    save_timer = Some(0.0);
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    state = GameState::Menu;
                }
            }
            GameState::Lan(ref mut mode) => {
                if lan::frame(mode, frame_dt, player_name, &resources, &mut audio) {
                    state = GameState::Menu;
                }
            }
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

                let click = render_menu(
                    &game,
                    player_name,
                    saved.high_score,
                    audio.settings(),
                    &resources,
                );
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
                    MenuItem::Lan => {
                        if enter || activate {
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                            state = GameState::Lan(Box::new(lan::LanMode::new(game.difficulty)));
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
                    game.reset(selected_ship, vec2(screen_width(), screen_height()));
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    state = GameState::Briefing;
                }
            }

            GameState::Briefing => {
                audio.play_music(&resources.audio, MusicTrack::Menu);
                let launch = render_briefing(&game.current_mission, player_name, true, &resources);

                if launch || is_key_pressed(KeyCode::Space) {
                    game.bounds = vec2(screen_width(), screen_height());
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
                        game.solo_pilot().ship.engine.current_thrust,
                    );

                    let outcome = game.step(
                        [PilotInput::keyboard(), PilotInput::default()],
                        vec2(screen_width(), screen_height()),
                        dt,
                        get_time(),
                    );

                    if matches!(outcome, BattleOutcome::Defeat) {
                        audio.set_engine_active(&resources.audio, false, 0.0);
                        audio.play_sfx(&resources.audio, audio::AudioCue::GameOver);
                        state = GameState::GameOver(game.score);
                    } else if matches!(outcome, BattleOutcome::Success) {
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
                        render_game(&game, None, &resources);
                    }
                }
            }

            GameState::Paused => {
                // Render the game in paused state (frozen frame)
                render_game(&game, None, &resources);
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
                let upgrades = render_mission_success(&game.current_mission, true, &resources);

                if upgrades || is_key_pressed(KeyCode::Enter) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    upgrade_selection = UpgradeSelection::default();
                    state = GameState::UpgradeBay;
                }
            }

            GameState::UpgradeBay => {
                if is_key_pressed(KeyCode::Up) {
                    upgrade_selection.prev();
                }
                if is_key_pressed(KeyCode::Down) {
                    upgrade_selection.next();
                }

                let clicked = render_upgrade_bay(
                    game.solo_pilot(),
                    &upgrade_selection,
                    "upgrade_continue",
                    &resources,
                );
                if let Some(index) = clicked {
                    upgrade_selection.0 = index;
                }

                if clicked.is_some() || is_key_pressed(KeyCode::Enter) {
                    if let Some(id) = upgrade_selection.selected() {
                        let _ = game.solo_pilot_mut().buy_upgrade(id);
                    } else {
                        game.next_mission();
                        audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                        state = GameState::Briefing;
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
