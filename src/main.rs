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
    let dev_mode = std::env::args().any(|arg| arg == "--dev");
    let save_path = save_file();
    let mut saved = load_save(save_path.as_deref());
    let mut state = if dev_mode {
        GameState::DevMenu {
            selection: 0,
            level: 1,
        }
    } else if saved
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
    let mut menu_selection = MenuItem::Start;
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
                    menu_selection = menu_selection.prev();
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }
                if is_key_pressed(KeyCode::Down) {
                    menu_selection = menu_selection.next();
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }

                let click = render_menu(
                    menu_selection,
                    game.difficulty,
                    player_name,
                    saved.high_score,
                    audio.settings(),
                    &resources,
                );
                if let Some(click) = click {
                    let item = click.item();
                    if matches!(click, MenuClick::Select(_)) && item != menu_selection {
                        audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                    }
                    menu_selection = item;
                }
                let activate = matches!(click, Some(MenuClick::Activate(_)));
                let enter = click.is_none() && is_key_pressed(KeyCode::Enter);
                let left = click.is_none() && is_key_pressed(KeyCode::Left);
                let right = click.is_none() && is_key_pressed(KeyCode::Right);

                match menu_selection {
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
                    MenuItem::EnemyCodex => {
                        if enter || activate {
                            audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                            state = GameState::EnemyCodex(0);
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

            GameState::DevMenu {
                mut selection,
                mut level,
            } => {
                audio.play_music(&resources.audio, MusicTrack::Menu);
                if is_key_pressed(KeyCode::Up) {
                    selection = (selection + 2) % 3;
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }
                if is_key_pressed(KeyCode::Down) {
                    selection = (selection + 1) % 3;
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }
                if selection == 0 {
                    if is_key_pressed(KeyCode::Left) {
                        level = level.saturating_sub(1).max(1);
                    } else if is_key_pressed(KeyCode::Right) {
                        level = level.saturating_add(1);
                    }
                }

                draw_text_centered("DEVELOPER MENU", -100.0, 36, ORANGE, &resources);
                let rows = [
                    format!("Level: {level}   [Left/Right]"),
                    "Start selected level".to_owned(),
                    "Open shop (test resources)".to_owned(),
                ];
                for (index, row) in rows.iter().enumerate() {
                    let color = if index == selection { YELLOW } else { WHITE };
                    draw_text_centered(row, -40.0 + index as f32 * 55.0, 28, color, &resources);
                }
                draw_text_centered(
                    "Up/Down: select   Enter: open   Esc: main menu",
                    160.0,
                    18,
                    GRAY,
                    &resources,
                );

                if is_key_pressed(KeyCode::Escape) {
                    state = GameState::Menu;
                } else if is_key_pressed(KeyCode::Enter) {
                    game.reset(ShipKind::Alpha, vec2(screen_width(), screen_height()));
                    if selection == 1 {
                        game.current_mission = systems::get_mission(level);
                        state = GameState::Briefing;
                    } else if selection == 2 {
                        let ship = &mut game.solo_pilot_mut().ship;
                        ship.scrap = 100;
                        ship.rare_metal = 100;
                        upgrade_selection = UpgradeSelection::default();
                        state = GameState::UpgradeBay;
                    } else {
                        state = GameState::DevMenu { selection, level };
                    }
                } else {
                    state = GameState::DevMenu { selection, level };
                }
            }

            GameState::EnemyCodex(selected) => {
                let mut selected = selected;
                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Left) {
                    selected = (selected + 4) % 5;
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::Right) {
                    selected = (selected + 1) % 5;
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiMove);
                }
                render_enemy_codex(selected, &resources);
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
                    state = GameState::Menu;
                } else {
                    state = GameState::EnemyCodex(selected);
                }
            }

            GameState::ShipSelection(previous) => {
                let mut selected_ship = keyboard_ship_selection(previous, &resources);
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
                let ships = game
                    .pilots
                    .each_ref()
                    .map(|pilot| pilot.as_ref().map(|p| p.ship.kind));
                draw_briefing_fleet(&ships, get_time() as f32, 0.0, &resources);
                let launch = render_briefing(&game.current_mission, player_name, true, &resources);

                if launch || is_key_pressed(KeyCode::Space) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    state = GameState::LaunchSequence(0.0);
                }
            }

            GameState::LaunchSequence(mut elapsed) => {
                elapsed += frame_dt;
                let ships = game
                    .pilots
                    .each_ref()
                    .map(|pilot| pilot.as_ref().map(|p| p.ship.kind));
                render_launch_cutscene(&ships, elapsed, &resources);
                if elapsed >= 4.5 {
                    game.bounds = vec2(screen_width(), screen_height());
                    game.start_mission();
                    audio.play_music(&resources.audio, MusicTrack::Gameplay);
                    state = GameState::Playing;
                } else {
                    state = GameState::LaunchSequence(elapsed);
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

                    let mut inputs = [PilotInput::default(); MAX_PILOTS];
                    inputs[0] = PilotInput::keyboard();
                    let outcome = game.step(
                        inputs,
                        vec2(screen_width(), screen_height()),
                        dt,
                        get_time(),
                    );

                    if matches!(outcome, BattleOutcome::Defeat) {
                        audio.set_engine_active(&resources.audio, false, 0.0);
                        audio.play_sfx(&resources.audio, audio::AudioCue::GameOver);
                        state = GameState::GameOver {
                            score: game.score,
                            ship: game.solo_pilot().ship.kind,
                            ship_pos: game.solo_pilot().ship.pos,
                            ship_rotation: game.solo_pilot().ship.rotation,
                            elapsed: 0.0,
                        };
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
                let upgrades = render_mission_success(&game.current_mission, true, &resources);

                if upgrades || is_key_pressed(KeyCode::Enter) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    if dev_mode {
                        state = GameState::DevMenu {
                            selection: 0,
                            level: game.current_mission.level_id,
                        };
                    } else {
                        upgrade_selection = UpgradeSelection::default();
                        state = GameState::UpgradeBay;
                    }
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
                    if upgrade_selection.0 == UpgradeSelection::SELL_SCRAP_INDEX {
                        let _ = game.solo_pilot_mut().sell_scrap();
                    } else if upgrade_selection.0 == UpgradeSelection::CONTINUE_INDEX {
                        game.next_mission();
                        audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                        state = GameState::Briefing;
                    } else if let Some(id) = upgrade_selection.selected() {
                        let _ = game.solo_pilot_mut().buy_upgrade(id);
                    }
                }
            }

            GameState::GameOver {
                score,
                ship,
                ship_pos,
                ship_rotation,
                mut elapsed,
            } => {
                elapsed += frame_dt;
                audio.set_paused(&resources.audio, false);
                audio.play_music(&resources.audio, MusicTrack::Menu);
                render_game(&game, &resources);
                let main_menu = render_game_over(
                    score,
                    saved.high_score,
                    ship,
                    ship_pos,
                    ship_rotation,
                    elapsed,
                    &resources,
                );

                if main_menu || is_key_pressed(KeyCode::Enter) {
                    audio.play_sfx(&resources.audio, audio::AudioCue::UiConfirm);
                    state = if dev_mode {
                        GameState::DevMenu {
                            selection: 0,
                            level: game.current_mission.level_id,
                        }
                    } else {
                        GameState::Menu
                    };
                } else {
                    state = GameState::GameOver {
                        score,
                        ship,
                        ship_pos,
                        ship_rotation,
                        elapsed,
                    };
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
