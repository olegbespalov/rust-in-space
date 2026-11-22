use super::net;
use super::prediction::Prediction;
use super::session::{Action, Host, Phase, SnapshotView, Status, HOST};
use super::Browser;
use crate::audio::{AudioState, MusicTrack};
use crate::components::Ship;
use crate::draw::*;
use crate::game::{Game, PilotInput, UpgradeSelection, MAX_PILOTS};
use crate::resources::Resources;
use macroquad::prelude::*;
use std::net::{Ipv4Addr, SocketAddrV4};

const HOSTS_VISIBLE: usize = 4;
const VICTORY_DURATION: f64 = 2.4;

fn view_size(bounds: Vec2) -> Vec2 {
    vec2(screen_width().max(1.0), screen_height().max(1.0)).min(bounds)
}

pub(super) enum BrowserAction {
    Stay,
    Back,
    Host,
    Join(SocketAddrV4),
}

fn button(key: &str, y: f32, res: &Resources) -> bool {
    centered_menu_button(&res.lang.t(key), y, res)
}

pub(super) fn browser(browser: &mut Browser, res: &Resources) -> BrowserAction {
    draw_background(&res.background);
    draw_text_centered(&res.lang.t("lan_menu"), -224.0, 32, ORANGE, res);
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
        res.lang.t("lan_ip").into_owned()
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
    draw_text_centered(&res.lang.t(heading), -7.0, 16, SKYBLUE, res);
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
        draw_text_centered(&res.lang.t(status.key()), 190.0, 14, GOLD, res);
    }
    BrowserAction::Stay
}

pub(super) fn connecting(address: &str, res: &Resources) -> bool {
    draw_background(&res.background);
    draw_text_centered(
        &res.lang.t(Status::Connecting.key()),
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
    phase_started_at: f64,
    followed_slot: Option<usize>,
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
        self.followed_slot = None;
        self.selection = UpgradeSelection::default();
        self.menu = false;
    }

    pub fn session(
        &mut self,
        state: &SnapshotView<'_>,
        slot: usize,
        host: Option<&Host>,
        prediction: Option<(&mut Prediction, f32)>,
        res: &Resources,
        audio: &mut AudioState,
    ) -> Controls {
        if self.revision != Some(state.revision) {
            self.selection = UpgradeSelection::default();
            self.revision = Some(state.revision);
            self.phase_started_at = get_time();
            self.followed_slot = None;
        }
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
                if !self.menu && lobby_button(&res.lang.t("lan_leave"), 321.0, res) {
                    controls.leave = true;
                }
            }
            Phase::Briefing => {
                let may_launch = host.is_some() && !self.menu;
                let ships = state
                    .game
                    .pilots
                    .each_ref()
                    .map(|pilot| pilot.as_ref().map(|p| p.ship.kind));
                draw_briefing_fleet(&ships, get_time() as f32, 0.0, res);
                let launch = render_briefing(
                    &state.game.current_mission,
                    state.player_name(slot),
                    may_launch,
                    res,
                );
                if may_launch && (launch || is_key_pressed(KeyCode::Space)) {
                    controls.action = Some(Action::Advance);
                } else if host.is_none() {
                    draw_text_centered(&res.lang.t("lan_wait_launch"), 190.0, 14, SKYBLUE, res);
                }
            }
            Phase::Launching => {
                let ships = state
                    .game
                    .pilots
                    .each_ref()
                    .map(|pilot| pilot.as_ref().map(|p| p.ship.kind));
                render_launch_cutscene(&ships, (get_time() - self.phase_started_at) as f32, res);
            }
            Phase::Playing => {
                if !self.menu && pilot.is_alive() {
                    controls.input = PilotInput::keyboard();
                }
                let local_ship = prediction.and_then(|(prediction, dt)| {
                    prediction.advance(state.revision, pilot, state.game.bounds, controls.input, dt)
                });
                self.battle(state, slot, local_ship.as_ref(), res);
            }
            Phase::Success => {
                let progress = ((get_time() - self.phase_started_at) / VICTORY_DURATION)
                    .clamp(0.0, 1.0) as f32;
                if let Some(mvp) = &state.mvp {
                    render_lan_victory(
                        &state.game.current_mission,
                        mvp.ship,
                        &mvp.name,
                        mvp.points,
                        progress,
                        res,
                    );
                }
                if progress >= 1.0 {
                    let height = screen_height();
                    let button_height = MENU_BUTTON_HEIGHT.min(height * 0.075);
                    if host.is_none() {
                        draw_menu_label(
                            &res.lang.t("lan_wait_workshop"),
                            Rect::new(0.0, height * 0.85, screen_width(), height * 0.08),
                            14,
                            SKYBLUE,
                            res,
                        );
                    } else if !self.menu
                        && (centered_menu_button_with_height(
                            &res.lang.t("next_mission"),
                            height * 0.39,
                            button_height,
                            res,
                        ) || enter)
                    {
                        controls.action = Some(Action::Advance);
                    }
                }
            }
            Phase::UpgradeBay => {
                let can_advance = host.is_some() && state.guests_ready();
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
                        controls.action = if self.selection.0 == UpgradeSelection::SELL_SCRAP_INDEX
                        {
                            (host.is_some() || !state.ready[slot]).then_some(Action::SellScrap)
                        } else if self.selection.0 == UpgradeSelection::CONTINUE_INDEX {
                            if can_advance {
                                Some(Action::Advance)
                            } else {
                                Some(Action::Ready)
                            }
                        } else {
                            self.selection
                                .selected()
                                .filter(|_| host.is_some() || !state.ready[slot])
                                .map(Action::Buy)
                        };
                    }
                }
                let readiness = if can_advance {
                    "lan_ready_advance"
                } else if state.ready[slot] {
                    "lan_marked_ready"
                } else {
                    "lan_upgrade_ready"
                };
                let (readiness_color, readiness_size) = if state.ready[slot] {
                    (GREEN, 24)
                } else if can_advance {
                    (GOLD, 18)
                } else {
                    (SKYBLUE, 14)
                };
                draw_text_centered(
                    &res.lang.t(readiness),
                    237.0,
                    readiness_size,
                    readiness_color,
                    res,
                );
            }
            Phase::Defeat => {
                draw_text_centered(&res.lang.t("game_over"), -50.0, 40, RED, res);
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
                    draw_text_centered(&res.lang.t("lan_wait_lobby"), 85.0, 14, SKYBLUE, res);
                }
            }
        }

        if let Some(status) = host
            .filter(|_| state.phase != Phase::Lobby)
            .and_then(|host| host.status)
        {
            let top = if playing { 94.0 } else { 29.0 };
            draw_text_centered(
                &res.lang.t(status.key()),
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
            draw_text_centered(&res.lang.t("lan_session_menu"), -50.0, 30, ORANGE, res);
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

    fn battle(
        &mut self,
        state: &SnapshotView<'_>,
        slot: usize,
        ship: Option<&Ship>,
        res: &Resources,
    ) {
        let game = state.game;
        let view_size = view_size(game.bounds);
        let switch = !self.menu && is_key_pressed(KeyCode::Tab);
        let focus = battle_focus(game, slot, self.followed_slot, switch);
        self.followed_slot = Some(focus);
        let view = LanView {
            slot,
            focus,
            area: battle_area(game, focus, view_size, ship.filter(|_| focus == slot)),
            names: std::array::from_fn(|index| state.player_name(index)),
            local_ship: ship,
        };
        let target = self
            .field
            .take()
            .filter(|target| {
                target.texture.width() == view_size.x && target.texture.height() == view_size.y
            })
            .unwrap_or_else(|| render_target(view_size.x as u32, view_size.y as u32));
        self.field = Some(target);
        let target = self
            .field
            .as_ref()
            .expect("render target was just assigned");
        let mut camera = Camera2D::from_display_rect(view.area);
        camera.render_target = Some(target.clone());
        set_camera(&camera);
        clear_background(BLACK);
        draw_background_in(&res.background, game.bounds);
        render_battle_world(game, Some(view), res);

        camera.target = view_size / 2.0;
        set_camera(&camera);
        render_battle_hud(game, Some(view), res);
        set_default_camera();

        let window = vec2(screen_width(), screen_height());
        let size = view_size * (window / view_size).min_element();
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

fn battle_focus(game: &Game, local: usize, followed: Option<usize>, switch: bool) -> usize {
    let alive = |slot: usize| {
        game.pilots[slot]
            .as_ref()
            .is_some_and(|pilot| pilot.is_alive())
    };
    if alive(local) {
        return local;
    }
    let current = followed
        .filter(|&slot| alive(slot))
        .or_else(|| (0..MAX_PILOTS).find(|&slot| alive(slot)))
        .unwrap_or(local);
    if switch && followed == Some(current) {
        return (1..MAX_PILOTS)
            .map(|offset| (current + offset) % MAX_PILOTS)
            .find(|&slot| alive(slot))
            .unwrap_or(current);
    }
    current
}

fn battle_area(game: &Game, slot: usize, view_size: Vec2, ship: Option<&Ship>) -> Rect {
    let focus = ship
        .or_else(|| game.pilots[slot].as_ref().map(|pilot| &pilot.ship))
        .map_or(game.bounds / 2.0, |ship| ship.pos);
    let origin = (focus - view_size / 2.0).clamp(Vec2::ZERO, game.bounds - view_size);
    Rect::new(origin.x, origin.y, view_size.x, view_size.y)
}

fn lobby_offset(y: f32) -> f32 {
    y * (screen_height() / 720.0).min(1.0)
}

fn lobby_button(label: &str, y: f32, res: &Resources) -> bool {
    centered_menu_button_with_height(
        label,
        lobby_offset(y),
        lobby_offset(MENU_BUTTON_HEIGHT),
        res,
    )
}

fn lobby_label(text: &str, y: f32, height: f32, size: u16, color: Color, res: &Resources) {
    let bounds = Rect::new(
        0.0,
        screen_height() / 2.0 + lobby_offset(y - height / 2.0),
        screen_width(),
        lobby_offset(height),
    );
    draw_menu_label(text, bounds, size, color, res);
}

fn lobby_slots(state: &SnapshotView<'_>, own_slot: usize, res: &Resources) {
    let y = screen_height() / 2.0 + lobby_offset(-237.0);
    let slots = ship_card_row(MAX_PILOTS, y, lobby_offset(108.0));
    for (slot, (pilot, bounds)) in state.game.pilots.iter().zip(slots).enumerate() {
        let Rect {
            x,
            y,
            w: width,
            h: height,
        } = bounds;
        let border = if slot == own_slot { SKYBLUE } else { GRAY };
        draw_rectangle(x, y, width, height, Color::from_rgba(12, 17, 33, 230));
        draw_rectangle_lines(x, y, width, height, 1.0, border);

        let mut label = format!("{}", slot + 1);
        if slot == HOST {
            label.push_str(" · ");
            label.push_str(&res.lang.t("lan_slot_host"));
        }
        if slot == own_slot {
            label.push_str(" · ");
            label.push_str(&res.lang.t("lan_slot_you"));
        }
        draw_menu_label(
            &label,
            Rect::new(x, y, width, height * 0.22),
            13,
            border,
            res,
        );

        if let Some(pilot) = pilot {
            let name = state.player_name(slot);
            let kind = pilot.ship.kind;
            draw_ship_hull(
                kind,
                vec2(x + width / 2.0, y + height * 0.43),
                (height * 0.48).min(width * 0.9),
                0.0,
                0.0,
                WHITE,
                res,
            );
            let name_bounds = Rect::new(x, y + height * 0.65, width, height * 0.19);
            let ship_bounds = Rect::new(x, y + height * 0.83, width, height * 0.16);
            draw_menu_label(name, name_bounds, 16, WHITE, res);
            draw_menu_label(&res.lang.t(kind.label()), ship_bounds, 12, SKYBLUE, res);
            if state.ready[slot] {
                let check = vec2(x + width - 17.0, y + 17.0);
                draw_circle(check.x, check.y, 12.0, GREEN);
                draw_line(
                    check.x - 6.0,
                    check.y,
                    check.x - 1.5,
                    check.y + 4.5,
                    2.5,
                    WHITE,
                );
                draw_line(
                    check.x - 1.5,
                    check.y + 4.5,
                    check.x + 7.0,
                    check.y - 5.0,
                    2.5,
                    WHITE,
                );
            }
        } else {
            let bounds = Rect::new(x, y + height * 0.22, width, height * 0.72);
            draw_menu_label(&res.lang.t("lan_slot_empty"), bounds, 14, LIGHTGRAY, res);
        }
    }
}

fn lobby(
    state: &SnapshotView<'_>,
    slot: usize,
    host: Option<&Host>,
    enter: bool,
    res: &Resources,
) -> Option<Action> {
    let is_host = host.is_some();
    lobby_label(&res.lang.t("lan_lobby"), -306.0, 52.0, 30, ORANGE, res);
    let pilots = state.game.pilot_count();
    if let Some(host) = host {
        let addresses = host
            .addresses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let addresses = if addresses.is_empty() {
            res.lang.t("lan_no_ip").into_owned()
        } else {
            addresses.join(" / ")
        };
        let (label, color) = if host.discovery_available() {
            (addresses, SKYBLUE)
        } else {
            (
                format!("{}  {addresses}", res.lang.t("lan_search_unavailable")),
                GOLD,
            )
        };
        lobby_label(&label, -265.0, 32.0, 14, color, res);
    }
    lobby_slots(state, slot, res);

    let pilot = state.game.pilots[slot].as_ref()?;
    let active_kind = pilot.ship.kind;
    let card_height = lobby_offset(SHIP_CARD_HEIGHT);
    let ship_clicked = render_ship_cards(
        active_kind,
        (screen_height() - card_height) / 2.0,
        card_height,
        res,
    );

    let difficulty = format!(
        "{}  {}",
        res.lang.t("menu_difficulty"),
        res.lang.t(difficulty_key(state.game.difficulty))
    );
    let difficulty_clicked = if is_host {
        lobby_button(&difficulty, 153.0, res)
    } else {
        lobby_label(&difficulty, 153.0, MENU_BUTTON_HEIGHT, 16, WHITE, res);
        false
    };

    let status = if pilots == 1 {
        res.lang.t("lan_wait_guest").into_owned()
    } else {
        format!(
            "{} {}/{}",
            res.lang.t("lan_ready_count"),
            state.ready_guests(),
            pilots - 1
        )
    };
    let (status, color) = match host.and_then(|host| host.status) {
        Some(notice) => (format!("{}  ·  {status}", res.lang.t(notice.key())), GOLD),
        None => (status, SKYBLUE),
    };
    lobby_label(&status, 202.0, 28.0, 16, color, res);
    let start = if is_host {
        pilots > 1
            && state.guests_ready()
            && (lobby_button(&res.lang.t("lan_start"), 251.0, res) || enter)
    } else {
        let key = if state.ready[slot] {
            "lan_not_ready"
        } else {
            "lan_ready"
        };
        lobby_button(&res.lang.t(key), 251.0, res) || enter
    };

    let kind = ship_clicked.unwrap_or_else(|| keyboard_ship_selection(active_kind, res));
    if kind != active_kind {
        return Some(Action::SelectShip(kind));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ShipKind;
    use crate::lan::session::FIELD;

    #[test]
    fn battle_camera_follows_the_local_pilot_within_the_arena() {
        let mut game = Game::new(FIELD);
        let slot = MAX_PILOTS - 1;
        game.set_pilot(slot, ShipKind::Alpha);
        let local = game.pilots[slot].as_mut().unwrap();
        local.ship.pos = FIELD;
        assert_eq!(battle_focus(&game, slot, Some(HOST), true), slot);
        let viewport = vec2(1280.0, 720.0);
        assert_eq!(
            battle_area(&game, slot, viewport, None).point(),
            FIELD - viewport
        );
        let mut predicted = game.pilots[slot].as_ref().unwrap().ship.clone();
        predicted.pos = FIELD / 2.0;
        assert_eq!(
            battle_area(&game, slot, viewport, Some(&predicted)).point(),
            (FIELD - viewport) / 2.0
        );

        game.pilots[slot].as_mut().unwrap().ship.health = 0.0;
        game.solo_pilot_mut().ship.pos = Vec2::ZERO;
        let focus = battle_focus(&game, slot, Some(slot), false);
        assert_eq!(focus, HOST);
        assert_eq!(
            battle_area(&game, focus, viewport, None).point(),
            Vec2::ZERO
        );
    }

    #[test]
    fn spectator_cycles_living_pilots_and_recovers_if_the_target_leaves() {
        let mut game = Game::new(FIELD);
        let guest = MAX_PILOTS - 1;
        game.set_pilot(guest, ShipKind::Alpha);
        game.solo_pilot_mut().ship.health = 0.0;
        game.set_pilot(1, ShipKind::Alpha);

        assert_eq!(battle_focus(&game, HOST, None, false), 1);
        assert_eq!(battle_focus(&game, HOST, Some(1), true), guest);
        assert_eq!(battle_focus(&game, HOST, Some(guest), false), guest);
        assert_eq!(battle_focus(&game, HOST, Some(guest), true), 1);

        game.pilots[guest] = None;
        assert_eq!(battle_focus(&game, HOST, Some(guest), false), 1);
        game.pilots[1].as_mut().unwrap().ship.health = 0.0;
        assert_eq!(battle_focus(&game, HOST, Some(1), true), HOST);
    }
}
