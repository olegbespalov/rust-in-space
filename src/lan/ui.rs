use super::net;
use super::session::{Action, Host, Phase, Snapshot, Status, FIELD};
use super::Browser;
use crate::audio::{AudioState, MusicTrack};
use crate::components::ShipKind;
use crate::draw::*;
use crate::game::{Game, PilotInput, UpgradeSelection};
use crate::resources::Resources;
use macroquad::prelude::*;
use std::net::{Ipv4Addr, SocketAddrV4};

const HOSTS_VISIBLE: usize = 4;

pub(super) enum BrowserAction {
    Stay,
    Back,
    Host,
    Join(SocketAddrV4),
}

fn button(key: &str, y: f32, res: &Resources) -> bool {
    centered_menu_button(res.lang.t(key), y, res)
}

pub(super) fn browser(browser: &mut Browser, res: &Resources) -> BrowserAction {
    draw_background(&res.background);
    draw_text_centered(res.lang.t("lan_menu"), -224.0, 32, ORANGE, res);
    let host = button("lan_host", -168.0, res);
    let field_width = screen_width().min(460.0) * 0.9;
    let field = Rect::new(
        (screen_width() - field_width) / 2.0,
        screen_height() / 2.0 - 127.0,
        field_width,
        38.0,
    );
    if is_mouse_button_pressed(MouseButton::Left) {
        browser.focused = field.contains(mouse_position().into());
    }
    if is_key_pressed(KeyCode::Tab) {
        browser.focused = !browser.focused;
    }
    while let Some(c) = get_char_pressed() {
        if browser.focused && browser.address.len() < 15 && (c.is_ascii_digit() || c == '.') {
            browser.address.push(c);
        }
    }
    if browser.focused {
        let modifier = [
            KeyCode::LeftControl,
            KeyCode::RightControl,
            KeyCode::LeftSuper,
            KeyCode::RightSuper,
        ]
        .into_iter()
        .any(is_key_down);
        if modifier && is_key_pressed(KeyCode::A) {
            browser.address.clear();
        } else if modifier && is_key_pressed(KeyCode::V) {
            if let Some(text) = macroquad::miniquad::window::clipboard_get() {
                if let Ok(ip) = text.trim().parse::<Ipv4Addr>() {
                    browser.address = ip.to_string();
                }
            }
        } else if is_key_pressed(KeyCode::Backspace) {
            browser.address.pop();
        }
    }
    let label = if browser.address.is_empty() {
        res.lang.t("lan_ip").to_owned()
    } else if browser.focused {
        format!("{}_", browser.address)
    } else {
        browser.address.clone()
    };
    let _ = menu_button(&label, field, 16, browser.focused, res);

    let enter = is_key_pressed(KeyCode::Enter);
    let manual_join = button("lan_join", -57.0, res) || (enter && browser.focused);
    let back = button("ship_back", 236.0, res) || is_key_pressed(KeyCode::Escape);
    if back {
        return BrowserAction::Back;
    }
    if host || (!browser.focused && is_key_pressed(KeyCode::H)) {
        return BrowserAction::Host;
    }
    if manual_join {
        match browser.address.parse::<Ipv4Addr>() {
            Ok(ip) if !ip.is_unspecified() && !ip.is_broadcast() && !ip.is_multicast() => {
                return BrowserAction::Join(net::endpoint(ip));
            }
            _ => browser.status = Some(Status::InvalidAddress),
        }
    }

    let hosts: Vec<_> = browser
        .discovery
        .iter()
        .flat_map(net::Discovery::hosts)
        .collect();
    browser.selected_host = browser.selected_host.min(hosts.len().saturating_sub(1));
    if !browser.focused {
        if is_key_pressed(KeyCode::Up) {
            browser.selected_host = browser.selected_host.saturating_sub(1);
        }
        if is_key_pressed(KeyCode::Down) && browser.selected_host + 1 < hosts.len() {
            browser.selected_host += 1;
        }
    }
    let heading = if browser.discovery.is_none() {
        "lan_search_unavailable"
    } else if hosts.is_empty() {
        "lan_searching"
    } else {
        "lan_games"
    };
    draw_text_centered(res.lang.t(heading), -7.0, 16, SKYBLUE, res);
    let start = browser.selected_host / HOSTS_VISIBLE * HOSTS_VISIBLE;
    for (index, address) in hosts.iter().enumerate().skip(start).take(HOSTS_VISIBLE) {
        let clicked = menu_button(
            &address.ip().to_string(),
            Rect::new(
                field.x,
                screen_height() / 2.0 + 13.0 + 33.0 * (index - start) as f32,
                field.w,
                30.0,
            ),
            14,
            !browser.focused && browser.selected_host == index,
            res,
        );
        if clicked || (enter && !browser.focused && browser.selected_host == index) {
            return BrowserAction::Join(*address);
        }
    }
    if hosts.len() > HOSTS_VISIBLE {
        draw_text_centered(
            &format!("↑ ↓  {}/{}", browser.selected_host + 1, hosts.len()),
            160.0,
            12,
            LIGHTGRAY,
            res,
        );
    }
    if let Some(status) = browser.status {
        draw_text_centered(res.lang.t(status.key()), 190.0, 14, GOLD, res);
    }
    BrowserAction::Stay
}

pub(super) fn connecting(address: &str, res: &Resources) -> bool {
    draw_background(&res.background);
    draw_text_centered(
        res.lang.t(Status::Connecting.key()),
        -55.0,
        24,
        SKYBLUE,
        res,
    );
    draw_text_centered(address, 5.0, 18, WHITE, res);
    button("ship_back", 70.0, res) || is_key_pressed(KeyCode::Escape)
}

#[derive(Default)]
pub(super) struct SessionUi {
    field: Option<RenderTarget>,
    revision: Option<u64>,
    selection: UpgradeSelection,
    menu: bool,
}

#[derive(Default)]
pub(super) struct Controls {
    pub action: Option<Action>,
    pub input: PilotInput,
    pub leave: bool,
}

impl SessionUi {
    pub fn reset(&mut self) {
        self.revision = None;
        self.selection = UpgradeSelection::default();
        self.menu = false;
    }

    pub fn session(
        &mut self,
        state: &Snapshot<&Game>,
        host: Option<&Host>,
        player_name: &str,
        res: &Resources,
        audio: &mut AudioState,
    ) -> Controls {
        if self.revision != Some(state.revision) {
            self.selection = UpgradeSelection::default();
            self.revision = Some(state.revision);
        }
        let slot = usize::from(host.is_none());
        let playing = state.phase == Phase::Playing;
        if is_key_pressed(KeyCode::Escape) {
            self.menu = !self.menu;
        }

        audio.play_music(
            &res.audio,
            if playing {
                MusicTrack::Gameplay
            } else {
                MusicTrack::Menu
            },
        );
        let mut controls = Controls::default();
        let enter = !self.menu && is_key_pressed(KeyCode::Enter);
        let Some(pilot) = &state.game.pilots[slot] else {
            return controls;
        };

        if !playing {
            draw_background(&res.background);
        }
        match state.phase {
            Phase::Lobby => {
                controls.action = lobby(state, slot, host, enter, res);
                if !self.menu && button("lan_leave", lobby_offset(215.0), res) {
                    controls.leave = true;
                }
            }
            Phase::Briefing => {
                let may_launch = host.is_some() && !self.menu;
                let launch =
                    render_briefing(&state.game.current_mission, player_name, may_launch, res);
                if may_launch && (launch || is_key_pressed(KeyCode::Space)) {
                    controls.action = Some(Action::Advance);
                } else if host.is_none() {
                    draw_text_centered(res.lang.t("lan_wait_launch"), 190.0, 14, SKYBLUE, res);
                }
            }
            Phase::Playing => {
                self.battle(state.game, slot, res);
                if !self.menu && pilot.is_alive() {
                    controls.input = PilotInput::keyboard();
                }
            }
            Phase::Success => {
                let may_advance = host.is_some() && !self.menu;
                let advance = render_mission_success(&state.game.current_mission, may_advance, res);
                if may_advance && (advance || enter) {
                    controls.action = Some(Action::Advance);
                } else if host.is_none() {
                    draw_text_centered(res.lang.t("lan_wait_workshop"), 110.0, 14, SKYBLUE, res);
                }
            }
            Phase::UpgradeBay => {
                let can_advance =
                    host.is_some() && (state.ready[1] || state.game.pilots[1].is_none());
                let continue_key = if can_advance {
                    "lan_next_briefing"
                } else if state.ready[slot] {
                    "lan_not_ready"
                } else {
                    "lan_ready"
                };
                if !self.menu {
                    if is_key_pressed(KeyCode::Up) {
                        self.selection.prev();
                    }
                    if is_key_pressed(KeyCode::Down) {
                        self.selection.next();
                    }
                }
                let clicked = render_upgrade_bay(pilot, &self.selection, continue_key, res);
                if !self.menu {
                    if let Some(index) = clicked {
                        self.selection.0 = index;
                    }
                    if clicked.is_some() || enter {
                        controls.action = match self.selection.selected() {
                            Some(id) if host.is_some() || !state.ready[slot] => {
                                Some(Action::Buy(id))
                            }
                            Some(_) => None,
                            None if can_advance => Some(Action::Advance),
                            None => Some(Action::Ready),
                        };
                    }
                }
                let readiness = if can_advance {
                    "lan_ready_advance"
                } else if state.ready[slot] {
                    if host.is_some() {
                        "lan_wait_guest_ready"
                    } else {
                        "lan_wait_briefing"
                    }
                } else {
                    "lan_upgrade_ready"
                };
                draw_text_centered(res.lang.t(readiness), 237.0, 14, SKYBLUE, res);
            }
            Phase::Defeat => {
                draw_text_centered(res.lang.t("game_over"), -50.0, 40, RED, res);
                draw_text_centered(
                    &format!("{} {}", res.lang.t("final_score_prefix"), state.game.score),
                    10.0,
                    24,
                    WHITE,
                    res,
                );
                if host.is_some() && (button("lan_back_lobby", 100.0, res) || enter) {
                    controls.action = Some(Action::Advance);
                } else if host.is_none() {
                    draw_text_centered(res.lang.t("lan_wait_lobby"), 85.0, 14, SKYBLUE, res);
                }
            }
        }

        if let Some(status) = host.and_then(|host| host.status) {
            let top = if playing { 94.0 } else { 29.0 };
            draw_text_centered(
                res.lang.t(status.key()),
                top - screen_height() / 2.0,
                14,
                GOLD,
                res,
            );
        }

        if self.menu {
            controls = Controls::default();
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, 0.85),
            );
            draw_text_centered(res.lang.t("lan_session_menu"), -50.0, 30, ORANGE, res);
            if button("lan_resume", 20.0, res) || is_key_pressed(KeyCode::Enter) {
                self.menu = false;
            } else if button("lan_leave", 84.0, res) {
                controls.leave = true;
            }
        }
        audio.set_engine_active(
            &res.audio,
            controls.input.thrust,
            pilot.ship.engine.current_thrust,
        );
        controls
    }

    fn battle(&mut self, game: &Game, slot: usize, res: &Resources) {
        let target = self
            .field
            .get_or_insert_with(|| render_target(FIELD.x as u32, FIELD.y as u32));
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, FIELD.x, FIELD.y));
        camera.render_target = Some(target.clone());
        set_camera(&camera);
        clear_background(BLACK);
        draw_background_in(&res.background, FIELD);
        render_game(game, Some(slot), res);
        set_default_camera();

        let window = vec2(screen_width(), screen_height());
        let size = FIELD * (window / FIELD).min_element();
        let offset = (window - size) / 2.0;
        draw_texture_ex(
            &target.texture,
            offset.x,
            offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                flip_y: true,
                ..Default::default()
            },
        );
    }
}

fn lobby_offset(y: f32) -> f32 {
    y * (screen_height() / 540.0).min(1.0)
}

fn lobby(
    state: &Snapshot<&Game>,
    slot: usize,
    host: Option<&Host>,
    enter: bool,
    res: &Resources,
) -> Option<Action> {
    let is_host = host.is_some();
    draw_text_centered(
        res.lang.t("lan_lobby"),
        lobby_offset(-212.0),
        30,
        ORANGE,
        res,
    );
    let role = if is_host {
        "lan_you_host"
    } else {
        "lan_you_guest"
    };
    let paired = state.game.pilots[1].is_some();
    draw_text_centered(
        &format!("{}  {} / 2", res.lang.t(role), if paired { 2 } else { 1 }),
        lobby_offset(-163.0),
        18,
        WHITE,
        res,
    );
    if let Some(host) = host {
        let addresses = host
            .addresses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let addresses = if addresses.is_empty() {
            res.lang.t("lan_no_ip").to_owned()
        } else {
            addresses.join(" / ")
        };
        draw_text_centered(&addresses, lobby_offset(-130.0), 14, SKYBLUE, res);
        if !host.discovery_available() {
            draw_text_centered(
                res.lang.t("lan_search_unavailable"),
                lobby_offset(-103.0),
                12,
                GOLD,
                res,
            );
        }
    }

    let pilot = state.game.pilots[slot].as_ref()?;
    let active_kind = pilot.ship.kind;
    let can_choose_delta = res.ship_model(ShipKind::Delta).is_some();
    let ship_clicked = render_ship_cards(
        active_kind,
        vec2(
            screen_width() / 2.0,
            screen_height() / 2.0 + lobby_offset(-32.0),
        ),
        vec2((screen_width() * 0.43).min(180.0), lobby_offset(116.0)),
        res,
    );

    let difficulty = format!(
        "{}  {}",
        res.lang.t("menu_difficulty"),
        res.lang.t(difficulty_key(state.game.difficulty))
    );
    let difficulty_clicked = if is_host {
        centered_menu_button(&difficulty, lobby_offset(57.0), res)
    } else {
        draw_text_centered(&difficulty, lobby_offset(59.0), 16, WHITE, res);
        false
    };

    let status = if !paired {
        "lan_wait_guest"
    } else if state.ready[1] {
        "lan_guest_ready"
    } else {
        "lan_guest_joined"
    };
    draw_text_centered(res.lang.t(status), lobby_offset(116.0), 16, SKYBLUE, res);
    let start = if is_host {
        paired && state.ready[1] && (button("lan_start", lobby_offset(154.0), res) || enter)
    } else {
        let key = if state.ready[1] {
            "lan_not_ready"
        } else {
            "lan_ready"
        };
        button(key, lobby_offset(154.0), res) || enter
    };

    if let Some(kind) = ship_clicked.filter(|kind| *kind != active_kind) {
        return Some(Action::SelectShip(kind));
    }
    if is_key_pressed(KeyCode::Left) && active_kind != ShipKind::Alpha {
        return Some(Action::SelectShip(ShipKind::Alpha));
    }
    if can_choose_delta && is_key_pressed(KeyCode::Right) && active_kind != ShipKind::Delta {
        return Some(Action::SelectShip(ShipKind::Delta));
    }
    if is_host
        && (difficulty_clicked || is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::Down))
    {
        return Some(Action::CycleDifficulty);
    }
    start.then_some(if is_host {
        Action::Advance
    } else {
        Action::Ready
    })
}
