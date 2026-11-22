mod net;
mod session;
mod ui;

use crate::audio::{AudioState, MusicTrack};
use crate::components::Difficulty;
use crate::resources::Resources;
use macroquad::prelude::*;
use net::Discovery;
use session::{Guest, Host, Status, HOST};
use std::net::SocketAddrV4;
use std::time::Instant;
use ui::{BrowserAction, SessionUi};

struct Browser {
    discovery: Option<Discovery>,
    status: Option<Status>,
    address: String,
    focused: bool,
    selected_host: usize,
}

impl Browser {
    fn new(now: Instant, status: Option<Status>, address: String) -> Self {
        Self {
            discovery: Discovery::new(now).ok(),
            status,
            address,
            focused: false,
            selected_host: 0,
        }
    }
}

enum Screen {
    Browser(Browser),
    Host(Box<Host>),
    Guest(Box<Guest>),
}

pub struct LanMode {
    screen: Screen,
    ui: SessionUi,
    difficulty: Difficulty,
    last_address: String,
}

impl LanMode {
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            screen: Screen::Browser(Browser::new(Instant::now(), None, String::new())),
            ui: SessionUi::default(),
            difficulty,
            last_address: String::new(),
        }
    }

    fn open_browser(&mut self, now: Instant, status: Status) {
        self.screen = Screen::Browser(Browser::new(now, Some(status), self.last_address.clone()));
        self.ui.reset();
    }

    fn join(&mut self, address: SocketAddrV4, now: Instant) -> Result<(), Status> {
        self.last_address = address.ip().to_string();
        self.screen = Screen::Guest(Box::new(Guest::new(address, now)?));
        Ok(())
    }
}

pub fn frame(
    mode: &mut LanMode,
    dt: f32,
    player_name: &str,
    res: &Resources,
    audio: &mut AudioState,
) -> bool {
    let now = Instant::now();
    let mut leave = None;
    match &mut mode.screen {
        Screen::Browser(browser) => {
            audio.set_engine_active(&res.audio, false, 0.0);
            audio.play_music(&res.audio, MusicTrack::Menu);
            if let Some(discovery) = &mut browser.discovery {
                discovery.poll(now);
            }
            match ui::browser(browser, res) {
                BrowserAction::Stay => {}
                BrowserAction::Back => return true,
                BrowserAction::Host => match Host::new(mode.difficulty, now) {
                    Ok(host) => mode.screen = Screen::Host(Box::new(host)),
                    Err(status) => browser.status = Some(status),
                },
                BrowserAction::Join(address) => {
                    if let Err(status) = mode.join(address, now) {
                        leave = Some(status);
                    }
                }
            }
        }
        Screen::Host(host) => {
            host.poll(now);
            let state = host.snapshot();
            let revision = state.revision;
            let controls = mode
                .ui
                .session(&state, HOST, Some(host), player_name, res, audio);
            if controls.leave {
                leave = Some(Status::Left);
            } else {
                if let Some(action) = controls.action {
                    host.command(revision, action);
                    audio.play_sfx(&res.audio, crate::audio::AudioCue::UiConfirm);
                }
                for cue in host.update(dt, controls.input, now) {
                    audio.play_sfx(&res.audio, cue);
                }
                mode.difficulty = host.snapshot().game.difficulty;
            }
        }
        Screen::Guest(guest) => match guest.poll(now) {
            Err(status) => leave = Some(status),
            Ok(cues) => {
                for cue in cues {
                    audio.play_sfx(&res.audio, cue);
                }
                if let (Some(state), Some(slot)) = (&guest.state, guest.slot) {
                    let view = session::Snapshot {
                        game: &state.game,
                        phase: state.phase,
                        revision: state.revision,
                        ready: state.ready,
                    };
                    let controls = mode.ui.session(&view, slot, None, player_name, res, audio);
                    if controls.leave {
                        leave = Some(Status::Left);
                    } else {
                        if controls.action.is_some() {
                            audio.play_sfx(&res.audio, crate::audio::AudioCue::UiConfirm);
                        }
                        if let Err(status) = guest.send(controls.action, controls.input, now) {
                            leave = Some(status);
                        }
                    }
                } else if ui::connecting(&mode.last_address, res) {
                    leave = Some(Status::Left);
                }
            }
        },
    }
    if let Some(status) = leave {
        audio.set_engine_active(&res.audio, false, 0.0);
        audio.play_music(&res.audio, MusicTrack::Menu);
        mode.open_browser(now, status);
    }
    false
}
