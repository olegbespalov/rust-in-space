use super::net::{self, Advertiser, Connection, Frame, COMMAND_LIMIT, SNAPSHOT_LIMIT};
use crate::audio::AudioCue;
use crate::components::{Difficulty, ShipKind, UpgradeId};
use crate::game::{BattleOutcome, Game, PilotInput, MAX_PILOTS};
use macroquad::prelude::{vec2, Vec2};
use serde::{Deserialize, Serialize};
use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

pub(super) const FIELD: Vec2 = vec2(1920.0, 1080.0);
pub(super) const HOST: usize = 0;
const VERSION: u16 = 6;
const STEP: f64 = 1.0 / 60.0;
const INPUT_PERIOD: Duration = Duration::from_nanos(33_333_334);
const HEARTBEAT_PERIOD: Duration = Duration::from_secs(1);
const INPUT_MAX_AGE: Duration = Duration::from_millis(300);
const MAX_HANDSHAKES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) enum Phase {
    Lobby,
    Briefing,
    Playing,
    Success,
    UpgradeBay,
    Defeat,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) enum Action {
    SelectShip(ShipKind),
    CycleDifficulty,
    Ready,
    Advance,
    Buy(UpgradeId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) enum Status {
    InvalidAddress,
    Connecting,
    PortBusy,
    HostFailed,
    ConnectFailed,
    Full,
    Incompatible,
    HostLost,
    GuestLost,
    Left,
}

impl Status {
    pub fn key(self) -> &'static str {
        match self {
            Self::InvalidAddress => "lan_invalid_ip",
            Self::Connecting => "lan_connecting",
            Self::PortBusy => "lan_port_busy",
            Self::HostFailed => "lan_host_failed",
            Self::ConnectFailed => "lan_connect_failed",
            Self::Full => "lan_full",
            Self::Incompatible => "lan_version",
            Self::HostLost => "lan_host_lost",
            Self::GuestLost => "lan_guest_lost",
            Self::Left => "lan_left",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Snapshot<G> {
    pub game: G,
    pub phase: Phase,
    pub revision: u64,
    pub ready: [bool; MAX_PILOTS],
}

impl Snapshot<&Game> {
    pub fn ready_guests(&self) -> usize {
        self.game.pilots[1..]
            .iter()
            .zip(&self.ready[1..])
            .filter(|(pilot, ready)| pilot.is_some() && **ready)
            .count()
    }

    pub fn guests_ready(&self) -> bool {
        self.ready_guests() == self.game.pilot_count() - 1
    }
}

#[derive(Serialize, Deserialize)]
enum FromClient {
    Hello(u16),
    Input { revision: u64, keys: PilotInput },
    Command { revision: u64, action: Action },
}

#[derive(Serialize, Deserialize)]
enum FromHost<G> {
    Welcome { version: u16, slot: usize },
    Reject(Status),
    State(Box<Snapshot<G>>),
    Audio(Vec<AudioCue>),
}

pub(super) struct Campaign {
    game: Game,
    phase: Phase,
    revision: u64,
    ready: [bool; MAX_PILOTS],
    needs_snapshot: bool,
    pending_time: f64,
    simulation_time: f64,
}

impl Campaign {
    fn new(difficulty: Difficulty) -> Self {
        let mut game = Game::new(FIELD);
        game.difficulty = difficulty;
        Self {
            game,
            phase: Phase::Lobby,
            revision: 0,
            ready: [false; MAX_PILOTS],
            needs_snapshot: false,
            pending_time: 0.0,
            simulation_time: 0.0,
        }
    }

    pub fn snapshot(&self) -> Snapshot<&Game> {
        Snapshot {
            game: &self.game,
            phase: self.phase,
            revision: self.revision,
            ready: self.ready,
        }
    }

    fn transition(&mut self, phase: Phase) {
        self.phase = phase;
        self.revision += 1;
        self.ready = [false; MAX_PILOTS];
        self.needs_snapshot = true;
        self.pending_time = 0.0;
    }

    fn add_guest(&mut self, slot: usize) {
        self.game.set_pilot(slot, ShipKind::Alpha);
        self.ready[slot] = false;
        self.needs_snapshot = true;
    }

    fn remove_guest(&mut self, slot: usize) {
        self.game.pilots[slot] = None;
        self.ready[slot] = false;
        self.needs_snapshot = true;
        if self.phase == Phase::Briefing {
            self.game.prepare_lan_mission();
        } else if self.phase == Phase::Playing && !self.game.has_survivors() {
            self.defeat();
        }
    }

    fn defeat(&mut self) {
        self.transition(Phase::Defeat);
        self.game.audio_cues.push(AudioCue::GameOver);
    }

    fn command(&mut self, slot: usize, revision: u64, action: Action) {
        if self.revision != revision || self.game.pilots[slot].is_none() {
            return;
        }

        match (self.phase, action) {
            (Phase::Lobby, Action::SelectShip(kind)) => {
                self.game.set_pilot(slot, kind);
                self.ready[slot] = false;
            }
            (Phase::Lobby, Action::CycleDifficulty) if slot == HOST => self.game.cycle_difficulty(),
            (Phase::Lobby, Action::Ready) if slot != HOST => self.ready[slot] ^= true,
            (Phase::Lobby, Action::Advance)
                if slot == HOST
                    && self.game.pilot_count() > 1
                    && self.snapshot().guests_ready() =>
            {
                let host_kind = self.game.solo_pilot().ship.kind;
                let kinds = self
                    .game
                    .pilots
                    .each_ref()
                    .map(|pilot| pilot.as_ref().map(|pilot| pilot.ship.kind));
                self.game.reset(host_kind, FIELD);
                for (slot, kind) in kinds.into_iter().enumerate().skip(1) {
                    if let Some(kind) = kind {
                        self.game.set_pilot(slot, kind);
                    }
                }
                self.game.prepare_lan_mission();
                self.simulation_time = 0.0;
                self.transition(Phase::Briefing);
            }
            (Phase::Briefing, Action::Advance) if slot == HOST => {
                self.game.prepare_lan_mission();
                self.game.start_mission();
                self.transition(Phase::Playing);
            }
            (Phase::Success, Action::Advance) if slot == HOST => self.transition(Phase::UpgradeBay),
            (Phase::UpgradeBay, Action::Buy(id)) if slot == HOST || !self.ready[slot] => {
                if let Some(pilot) = &mut self.game.pilots[slot] {
                    let _ = pilot.buy_upgrade(id);
                }
            }
            (Phase::UpgradeBay, Action::Ready) => self.ready[slot] ^= true,
            (Phase::UpgradeBay, Action::Advance)
                if slot == HOST && self.snapshot().guests_ready() =>
            {
                self.game.next_mission();
                self.game.prepare_lan_mission();
                self.transition(Phase::Briefing);
            }
            (Phase::Defeat, Action::Advance) if slot == HOST => self.transition(Phase::Lobby),
            _ => return,
        }
        self.needs_snapshot = true;
    }

    fn simulate(&mut self, dt: f32, mut inputs: impl FnMut() -> [PilotInput; MAX_PILOTS]) {
        if self.phase != Phase::Playing {
            return;
        }
        self.pending_time += f64::from(dt.clamp(0.0, 0.25));
        while self.pending_time >= STEP {
            self.pending_time -= STEP;
            self.simulation_time += STEP;
            self.needs_snapshot = true;
            match self
                .game
                .step(inputs(), FIELD, STEP as f32, self.simulation_time)
            {
                BattleOutcome::Playing => {}
                BattleOutcome::Defeat => self.defeat(),
                BattleOutcome::Success => {
                    self.transition(Phase::Success);
                    self.game.audio_cues.push(AudioCue::MissionSuccess);
                }
            }
        }
    }
}

struct Handshake {
    peer: Connection,
    rejecting: bool,
}

struct GuestPeer {
    peer: Connection,
    input: PilotInput,
    pressed: PilotInput,
    input_revision: u64,
    input_at: Instant,
}

impl GuestPeer {
    fn record_input(&mut self, revision: u64, keys: PilotInput, now: Instant) {
        if revision != self.input_revision || now.duration_since(self.input_at) >= INPUT_MAX_AGE {
            self.input = PilotInput::default();
            self.pressed = PilotInput::default();
        }
        self.pressed.left |= keys.left && !self.input.left;
        self.pressed.right |= keys.right && !self.input.right;
        self.pressed.thrust |= keys.thrust && !self.input.thrust;
        self.pressed.fire |= keys.fire && !self.input.fire;
        self.input = keys;
        self.input_revision = revision;
        self.input_at = now;
    }

    fn input_for_step(&mut self) -> PilotInput {
        let pressed = std::mem::take(&mut self.pressed);
        PilotInput {
            left: self.input.left || pressed.left,
            right: self.input.right || pressed.right,
            thrust: self.input.thrust || pressed.thrust,
            fire: self.input.fire || pressed.fire,
        }
    }

    fn send(
        &mut self,
        audio: Option<&Frame>,
        snapshot: Option<&Frame>,
        now: Instant,
    ) -> io::Result<()> {
        if let Some(audio) = audio {
            self.peer.queue_frame(audio.clone(), false, now)?;
        }
        if let Some(snapshot) = snapshot {
            self.peer.queue_frame(snapshot.clone(), true, now)?;
        }
        self.peer.flush(now)
    }
}

pub(super) struct Host {
    listener: TcpListener,
    advertiser: Option<Advertiser>,
    pending: Vec<Handshake>,
    guests: [Option<GuestPeer>; MAX_PILOTS],
    campaign: Campaign,
    next_heartbeat: Instant,
    pub status: Option<Status>,
    pub addresses: Vec<Ipv4Addr>,
}

impl Host {
    pub fn new(difficulty: Difficulty, now: Instant) -> Result<Self, Status> {
        Self::listen(net::endpoint(Ipv4Addr::UNSPECIFIED), difficulty, now)
    }

    fn listen(address: SocketAddrV4, difficulty: Difficulty, now: Instant) -> Result<Self, Status> {
        let listener = net::listen(address).map_err(|error| match error.kind() {
            ErrorKind::AddrInUse => Status::PortBusy,
            _ => Status::HostFailed,
        })?;
        Ok(Self {
            listener,
            advertiser: Advertiser::new(address).ok(),
            pending: Vec::new(),
            guests: std::array::from_fn(|_| None),
            campaign: Campaign::new(difficulty),
            next_heartbeat: now,
            status: None,
            addresses: net::local_addresses(),
        })
    }

    pub fn snapshot(&self) -> Snapshot<&Game> {
        self.campaign.snapshot()
    }

    pub fn discovery_available(&self) -> bool {
        self.advertiser.is_some()
    }

    pub fn command(&mut self, revision: u64, action: Action) {
        self.campaign.command(HOST, revision, action);
    }

    pub fn poll(&mut self, now: Instant) {
        for slot in 1..MAX_PILOTS {
            if self.receive_guest(slot, now).is_err() {
                self.lose_guest(slot);
            }
        }
        self.accept_connections(now);
        self.advance_handshakes(now);

        let available =
            self.campaign.game.pilot_count() < MAX_PILOTS && self.campaign.phase == Phase::Lobby;
        if let Some(advertiser) = &self.advertiser {
            if advertiser.poll(available).is_err() {
                self.advertiser = None;
            }
        }
    }

    pub fn update(&mut self, dt: f32, host_input: PilotInput, now: Instant) -> Vec<AudioCue> {
        let revision = self.campaign.revision;
        let guests = &mut self.guests;
        self.campaign.simulate(dt, || {
            let mut inputs = [PilotInput::default(); MAX_PILOTS];
            inputs[HOST] = host_input;
            for (input, guest) in inputs.iter_mut().zip(guests.iter_mut()).skip(1) {
                if let Some(guest) = guest.as_mut().filter(|guest| {
                    guest.input_revision == revision
                        && now.duration_since(guest.input_at) < INPUT_MAX_AGE
                }) {
                    *input = guest.input_for_step();
                }
            }
            inputs
        });
        let cues = self.campaign.game.drain_audio_cues();
        if self.send_guests(&cues, now).is_err() {
            for slot in 1..MAX_PILOTS {
                if self.guests[slot].is_some() {
                    self.lose_guest(slot);
                }
            }
        }
        cues
    }

    fn lose_guest(&mut self, slot: usize) {
        self.guests[slot] = None;
        self.campaign.remove_guest(slot);
        self.status = Some(Status::GuestLost);
    }

    fn receive_guest(&mut self, slot: usize, now: Instant) -> io::Result<()> {
        let Some(guest) = &mut self.guests[slot] else {
            return Ok(());
        };
        for message in guest.peer.receive::<FromClient>(now)? {
            match message {
                FromClient::Input { revision, keys }
                    if self.campaign.phase == Phase::Playing
                        && revision == self.campaign.revision =>
                {
                    guest.record_input(revision, keys, now);
                }
                FromClient::Input { .. } => {}
                FromClient::Command { revision, action } => {
                    self.campaign.command(slot, revision, action)
                }
                FromClient::Hello(_) => return Err(ErrorKind::InvalidData.into()),
            }
        }
        Ok(())
    }

    fn send_guests(&mut self, cues: &[AudioCue], now: Instant) -> io::Result<()> {
        if self.guests.iter().all(Option::is_none) {
            return Ok(());
        }
        let audio = if cues.is_empty() {
            None
        } else {
            Some(Frame::compact(
                &FromHost::<()>::Audio(cues.to_vec()),
                SNAPSHOT_LIMIT,
            )?)
        };
        let snapshot = if self.campaign.needs_snapshot || now >= self.next_heartbeat {
            let state = FromHost::State(Box::new(self.campaign.snapshot()));
            let frame = Frame::compact(&state, SNAPSHOT_LIMIT)?;
            self.campaign.needs_snapshot = false;
            self.next_heartbeat = now + HEARTBEAT_PERIOD;
            Some(frame)
        } else {
            None
        };
        for slot in 1..MAX_PILOTS {
            if let Some(guest) = &mut self.guests[slot] {
                if guest.send(audio.as_ref(), snapshot.as_ref(), now).is_err() {
                    self.lose_guest(slot);
                }
            }
        }
        Ok(())
    }

    fn accept_connections(&mut self, now: Instant) {
        for _ in 0..MAX_HANDSHAKES {
            if self.pending.len() == MAX_HANDSHAKES {
                break;
            }
            let Ok((stream, _)) = self.listener.accept() else {
                break;
            };
            if let Ok(peer) = Connection::new(stream, COMMAND_LIMIT, now) {
                self.pending.push(Handshake {
                    peer,
                    rejecting: false,
                });
            }
        }
    }

    fn advance_handshakes(&mut self, now: Instant) {
        let mut index = 0;
        while index < self.pending.len() {
            let handshake = &mut self.pending[index];
            if handshake.rejecting {
                if handshake.peer.flush(now).is_err() || handshake.peer.is_flushed() {
                    self.pending.swap_remove(index);
                } else {
                    index += 1;
                }
                continue;
            }

            let messages = handshake.peer.receive::<FromClient>(now);
            let version = match messages.as_deref() {
                Ok([]) => {
                    index += 1;
                    continue;
                }
                Ok([FromClient::Hello(version)]) => *version,
                _ => {
                    self.pending.swap_remove(index);
                    continue;
                }
            };

            let admission = if version != VERSION {
                Err(Status::Incompatible)
            } else if self.campaign.phase != Phase::Lobby {
                Err(Status::Full)
            } else {
                (1..MAX_PILOTS)
                    .find(|&slot| self.guests[slot].is_none())
                    .ok_or(Status::Full)
            };
            let slot = match admission {
                Ok(slot) => slot,
                Err(status) => {
                    let result = handshake.peer.queue_handshake(
                        &FromHost::<()>::Reject(status),
                        SNAPSHOT_LIMIT,
                        now,
                    );
                    if result.is_ok() {
                        handshake.rejecting = true;
                    } else {
                        self.pending.swap_remove(index);
                    }
                    continue;
                }
            };

            let mut peer = self.pending.swap_remove(index).peer;
            if peer
                .queue_handshake(
                    &FromHost::<()>::Welcome {
                        version: VERSION,
                        slot,
                    },
                    SNAPSHOT_LIMIT,
                    now,
                )
                .is_ok()
            {
                self.campaign.add_guest(slot);
                self.status = None;
                self.guests[slot] = Some(GuestPeer {
                    peer,
                    input: PilotInput::default(),
                    pressed: PilotInput::default(),
                    input_revision: self.campaign.revision,
                    input_at: now,
                });
            }
        }
    }
}

enum GuestConnection {
    Connecting(Receiver<io::Result<TcpStream>>),
    Peer(Connection),
}

pub(super) struct Guest {
    connection: GuestConnection,
    pub slot: Option<usize>,
    pub state: Option<Snapshot<Game>>,
    next_input: Instant,
    last_input: PilotInput,
    last_revision: u64,
}

impl Guest {
    pub fn new(address: SocketAddrV4, now: Instant) -> Result<Self, Status> {
        let receiver = net::connect(address).map_err(|_| Status::ConnectFailed)?;
        Ok(Self {
            connection: GuestConnection::Connecting(receiver),
            slot: None,
            state: None,
            next_input: now,
            last_input: PilotInput::default(),
            last_revision: 0,
        })
    }

    pub fn poll(&mut self, now: Instant) -> Result<Vec<AudioCue>, Status> {
        if let GuestConnection::Connecting(receiver) = &self.connection {
            let stream = match receiver.try_recv() {
                Err(TryRecvError::Empty) => return Ok(Vec::new()),
                Ok(Ok(stream)) => stream,
                _ => return Err(Status::ConnectFailed),
            };
            let mut peer =
                Connection::new(stream, SNAPSHOT_LIMIT, now).map_err(|_| Status::ConnectFailed)?;
            peer.queue_handshake(&FromClient::Hello(VERSION), COMMAND_LIMIT, now)
                .map_err(|_| Status::ConnectFailed)?;
            self.connection = GuestConnection::Peer(peer);
        }

        let mut cues = Vec::new();
        if let GuestConnection::Peer(peer) = &mut self.connection {
            let error = if self.slot.is_some() {
                Status::HostLost
            } else {
                Status::ConnectFailed
            };
            for message in peer.receive::<FromHost<Game>>(now).map_err(|_| error)? {
                match message {
                    FromHost::Welcome {
                        version: VERSION,
                        slot,
                    } if self.slot.is_none() && (1..MAX_PILOTS).contains(&slot) => {
                        self.slot = Some(slot);
                    }
                    FromHost::Welcome { .. } => return Err(Status::Incompatible),
                    FromHost::Reject(status) if self.slot.is_none() => return Err(status),
                    FromHost::State(state)
                        if self
                            .slot
                            .is_some_and(|slot| state.game.pilots[slot].is_some())
                            && state.game.bounds == FIELD
                            && state.game.pilots[HOST].is_some() =>
                    {
                        self.state = Some(*state);
                    }
                    FromHost::Audio(mut audio) if self.slot.is_some() => cues.append(&mut audio),
                    _ => return Err(error),
                }
            }
            peer.flush(now).map_err(|_| error)?;
        }
        Ok(cues)
    }

    pub fn send(
        &mut self,
        command: Option<Action>,
        input: PilotInput,
        now: Instant,
    ) -> Result<(), Status> {
        let (GuestConnection::Peer(peer), Some(state)) = (&mut self.connection, &self.state) else {
            return Ok(());
        };
        let revision = state.revision;
        if let Some(action) = command {
            peer.queue(
                &FromClient::Command { revision, action },
                COMMAND_LIMIT,
                false,
                now,
            )
            .map_err(|_| Status::HostLost)?;
        }
        let input_changed = input != self.last_input || revision != self.last_revision;
        if now >= self.next_input || input_changed {
            peer.queue(
                &FromClient::Input {
                    revision,
                    keys: input,
                },
                COMMAND_LIMIT,
                !input_changed,
                now,
            )
            .map_err(|_| Status::HostLost)?;
            self.last_input = input;
            self.last_revision = revision;
            self.next_input = now
                + if state.phase == Phase::Playing {
                    INPUT_PERIOD
                } else {
                    HEARTBEAT_PERIOD
                };
        }
        peer.flush(now).map_err(|_| Status::HostLost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::MissionObjective;
    use crate::systems::get_mission;
    use std::net::SocketAddr;

    const GUEST: usize = 1;

    fn create_host() -> Host {
        Host::listen(
            SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0),
            Difficulty::Nebula,
            Instant::now(),
        )
        .unwrap()
    }

    fn address(host: &Host) -> SocketAddrV4 {
        let SocketAddr::V4(address) = host.listener.local_addr().unwrap() else {
            unreachable!()
        };
        address
    }

    fn wait_for(host: &mut Host, guest: &mut Guest, condition: impl Fn(&Host, &Guest) -> bool) {
        for _ in 0..500 {
            let now = Instant::now();
            host.poll(now);
            host.update(STEP as f32, PilotInput::default(), now);
            guest.poll(now).unwrap();
            if condition(host, guest) {
                return;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        panic!("condition failed in phase {:?}", host.campaign.phase);
    }

    fn connect(host: &mut Host) -> Guest {
        let mut guest = Guest::new(address(host), Instant::now()).unwrap();
        wait_for(host, &mut guest, |_, guest| {
            guest
                .state
                .as_ref()
                .is_some_and(|state| state.phase == Phase::Lobby)
        });
        guest
    }

    fn guest_command(guest: &mut Guest, action: Action) {
        guest
            .send(Some(action), PilotInput::default(), Instant::now())
            .unwrap();
    }

    fn host_command(host: &mut Host, action: Action) {
        host.command(host.campaign.revision, action);
    }

    fn wait_for_phase(host: &mut Host, guest: &mut Guest, phase: Phase) {
        wait_for(host, guest, |_, guest| {
            guest
                .state
                .as_ref()
                .is_some_and(|state| state.phase == phase)
        });
    }

    #[test]
    fn guest_receives_each_battle_update_at_sixty_fps() {
        let mut host = create_host();
        let mut guest = connect(&mut host);
        guest_command(&mut guest, Action::Ready);
        wait_for(&mut host, &mut guest, |host, _| host.campaign.ready[GUEST]);
        host_command(&mut host, Action::Advance);
        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::Playing);

        let start = Instant::now();
        let input = PilotInput {
            right: true,
            ..PilotInput::default()
        };
        for frame in 1..=4 {
            let now = start + Duration::from_secs_f64(STEP * f64::from(frame));
            host.update(STEP as f32, input, now);
            let expected = host.campaign.game.solo_pilot().ship.rotation;
            for _ in 0..100 {
                guest.poll(now).unwrap();
                if guest
                    .state
                    .as_ref()
                    .unwrap()
                    .game
                    .solo_pilot()
                    .ship
                    .rotation
                    == expected
                {
                    break;
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(
                guest
                    .state
                    .as_ref()
                    .unwrap()
                    .game
                    .solo_pilot()
                    .ship
                    .rotation,
                expected,
                "guest missed simulation update on frame {frame}",
            );
        }
    }

    #[test]
    fn guest_tap_survives_a_batched_release_and_waits_for_a_simulation_step() {
        let mut host = create_host();
        let mut guest = connect(&mut host);
        guest_command(&mut guest, Action::Ready);
        wait_for(&mut host, &mut guest, |host, _| host.campaign.ready[GUEST]);
        host_command(&mut host, Action::Advance);
        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::Playing);
        host.campaign.pending_time = 0.0;
        host.campaign.game.asteroids.clear();

        let tap = PilotInput {
            fire: true,
            ..PilotInput::default()
        };
        guest.send(None, tap, Instant::now()).unwrap();
        guest
            .send(None, PilotInput::default(), Instant::now())
            .unwrap();
        for _ in 0..100 {
            host.poll(Instant::now());
            let remote = host.guests[GUEST].as_ref().unwrap();
            if remote.pressed.fire && !remote.input.fire {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let remote = host.guests[GUEST].as_ref().unwrap();
        assert!(remote.pressed.fire);
        assert!(!remote.input.fire);

        let half_step = (STEP / 2.0) as f32;
        assert!(!host
            .update(half_step, PilotInput::default(), Instant::now())
            .contains(&AudioCue::PlayerShot));
        assert!(host
            .update(half_step, PilotInput::default(), Instant::now())
            .contains(&AudioCue::PlayerShot));

        host.campaign.game.pilots[GUEST]
            .as_mut()
            .unwrap()
            .ship
            .shoot_timer = 0.0;
        assert!(!host
            .update(STEP as f32, PilotInput::default(), Instant::now())
            .contains(&AudioCue::PlayerShot));
    }

    #[test]
    fn loopback_campaign_synchronizes_combat_upgrades_and_returns_to_lobby() {
        let mut host = create_host();
        let mut guest = connect(&mut host);

        host_command(&mut host, Action::Advance);
        assert_eq!(host.campaign.phase, Phase::Lobby);
        host_command(&mut host, Action::CycleDifficulty);
        guest_command(&mut guest, Action::SelectShip(ShipKind::Delta));
        guest_command(&mut guest, Action::Ready);
        wait_for(&mut host, &mut guest, |_, guest| {
            guest.state.as_ref().unwrap().ready[GUEST]
        });
        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::Briefing);
        let state = guest.state.as_ref().unwrap();
        assert_eq!(state.game.current_mission.objectives[0].target(), 12);
        assert_eq!(state.game.current_mission.asteroid_count, 14);
        assert!(state.game.difficulty == Difficulty::Supernova);
        assert!(state.game.pilots[GUEST].as_ref().unwrap().ship.kind == ShipKind::Delta);

        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::Playing);
        let initial = guest.state.as_ref().unwrap().game.pilots[GUEST]
            .as_ref()
            .unwrap()
            .ship
            .pos;
        guest
            .send(
                None,
                PilotInput {
                    right: true,
                    thrust: true,
                    fire: true,
                    ..PilotInput::default()
                },
                Instant::now(),
            )
            .unwrap();
        wait_for(&mut host, &mut guest, |_, guest| {
            let game = &guest.state.as_ref().unwrap().game;
            game.pilots[GUEST].as_ref().unwrap().ship.pos != initial && !game.bullets.is_empty()
        });
        {
            let game = &mut host.campaign.game;
            game.mission_progress.enemies_defeated = 12;
            game.mission_progress.asteroids_destroyed = 15;
            game.pilots[HOST].as_mut().unwrap().ship.scrap = 30;
            game.pilots[GUEST].as_mut().unwrap().ship.scrap = 40;
            game.score = 950;
        }
        wait_for_phase(&mut host, &mut guest, Phase::Success);
        let game = &guest.state.as_ref().unwrap().game;
        assert_eq!(game.mission_progress.enemies_defeated, 12);
        assert_eq!(game.score, 950);
        let stale_revision = host.campaign.revision;
        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::UpgradeBay);

        host_command(&mut host, Action::Advance);
        assert_eq!(host.campaign.phase, Phase::UpgradeBay);
        host.command(stale_revision, Action::Buy(UpgradeId::ReinforcedHull));
        assert_eq!(host.campaign.game.solo_pilot().ship.scrap, 30);
        host_command(&mut host, Action::Buy(UpgradeId::ReinforcedHull));
        guest_command(&mut guest, Action::Buy(UpgradeId::WeaponTuning));
        guest_command(&mut guest, Action::Ready);
        wait_for(&mut host, &mut guest, |_, guest| {
            guest.state.as_ref().unwrap().ready[GUEST]
        });
        let game = &guest.state.as_ref().unwrap().game;
        let host_pilot = game.solo_pilot();
        let guest_pilot = game.pilots[GUEST].as_ref().unwrap();
        assert_eq!(host_pilot.ship.scrap, 10);
        assert_eq!(guest_pilot.ship.scrap, 10);
        assert_eq!(host_pilot.upgrade_level(UpgradeId::ReinforcedHull), 1);
        assert_eq!(guest_pilot.upgrade_level(UpgradeId::WeaponTuning), 1);
        assert_eq!(host_pilot.upgrade_level(UpgradeId::WeaponTuning), 0);

        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::Briefing);
        assert_eq!(
            guest.state.as_ref().unwrap().game.current_mission.level_id,
            2
        );
        host_command(&mut host, Action::Advance);
        host.campaign.game.pilots[HOST]
            .as_mut()
            .unwrap()
            .ship
            .health = 0.0;
        wait_for_phase(&mut host, &mut guest, Phase::Playing);
        let game = &guest.state.as_ref().unwrap().game;
        assert!(!game.solo_pilot().is_alive());
        assert!(game.pilots[GUEST].as_ref().unwrap().is_alive());
        host.campaign.game.pilots[GUEST]
            .as_mut()
            .unwrap()
            .ship
            .health = 0.0;
        wait_for_phase(&mut host, &mut guest, Phase::Defeat);
        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::Lobby);
        guest_command(&mut guest, Action::Ready);
        wait_for(&mut host, &mut guest, |_, guest| {
            guest.state.as_ref().unwrap().ready[GUEST]
        });
        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::Briefing);
        assert_eq!(guest.state.as_ref().unwrap().game.score, 0);
        assert_eq!(
            guest.state.as_ref().unwrap().game.current_mission.level_id,
            1
        );
    }

    #[test]
    fn four_pilots_share_a_battle_with_independent_input_and_disconnects() {
        let mut host = create_host();
        let [mut first, mut second, mut third] = std::array::from_fn(|_| connect(&mut host));
        assert_eq!(
            [first.slot, second.slot, third.slot],
            [Some(1), Some(2), Some(3)]
        );

        guest_command(&mut first, Action::Ready);
        guest_command(&mut third, Action::Ready);
        wait_for(&mut host, &mut first, |host, _| {
            host.snapshot().ready_guests() == 2
        });
        host_command(&mut host, Action::Advance);
        assert_eq!(host.campaign.phase, Phase::Lobby);
        guest_command(&mut second, Action::Ready);
        wait_for(&mut host, &mut second, |host, _| {
            host.snapshot().guests_ready()
        });
        host_command(&mut host, Action::Advance);
        host_command(&mut host, Action::Advance);
        for guest in [&mut first, &mut second, &mut third] {
            wait_for_phase(&mut host, guest, Phase::Playing);
            let game = &guest.state.as_ref().unwrap().game;
            assert_eq!(game.bounds, FIELD);
            assert_eq!(game.pilot_count(), MAX_PILOTS);
        }
        let mut input = PilotInput {
            left: true,
            ..PilotInput::default()
        };
        first.send(None, input, Instant::now()).unwrap();
        input.left = false;
        input.right = true;
        third.send(None, input, Instant::now()).unwrap();
        let both_turning = |_: &Host, guest: &Guest| {
            let pilots = &guest.state.as_ref().unwrap().game.pilots;
            pilots[1].as_ref().unwrap().ship.rotation < 0.0
                && pilots[2].as_ref().unwrap().ship.rotation == 0.0
                && pilots[3].as_ref().unwrap().ship.rotation > 0.0
        };
        wait_for(&mut host, &mut third, both_turning);
        wait_for(&mut host, &mut first, both_turning);

        drop(second);
        wait_for(&mut host, &mut third, |_, guest| {
            guest.state.as_ref().unwrap().game.pilots[2].is_none()
        });
        assert_eq!(host.campaign.phase, Phase::Playing);
        assert!(host.guests[1].is_some() && host.guests[3].is_some());
        assert_eq!(third.slot, Some(3));
    }

    #[test]
    fn loopback_declines_extra_guest_and_incompatible_protocol() {
        let mut host = create_host();
        let guests: [_; MAX_PILOTS - 1] = std::array::from_fn(|_| connect(&mut host));
        let mut extra = Guest::new(address(&host), Instant::now()).unwrap();
        let wrong = TcpStream::connect(address(&host)).unwrap();
        let mut incompatible = Connection::new(wrong, SNAPSHOT_LIMIT, Instant::now()).unwrap();
        let now = Instant::now();
        incompatible
            .queue_handshake(&FromClient::Hello(VERSION - 1), COMMAND_LIMIT, now)
            .unwrap();
        incompatible.flush(now).unwrap();
        let mut full = false;
        let mut version = false;
        for _ in 0..500 {
            let now = Instant::now();
            host.poll(now);
            if !full {
                full = matches!(extra.poll(now), Err(Status::Full));
            }
            if !version {
                version = matches!(
                    incompatible
                        .receive::<FromHost<Game>>(now)
                        .unwrap()
                        .as_slice(),
                    [FromHost::Reject(Status::Incompatible)]
                );
            }
            if full && version {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(full && version);
        assert_eq!(host.campaign.game.pilot_count(), MAX_PILOTS);
        assert!(guests
            .iter()
            .enumerate()
            .all(|(index, guest)| guest.slot == Some(index + 1)));
    }

    #[test]
    fn guest_disconnect_in_battle_preserves_current_scaling() {
        let mut host = create_host();
        let mut guest = connect(&mut host);
        guest_command(&mut guest, Action::Ready);
        wait_for(&mut host, &mut guest, |host, _| host.campaign.ready[GUEST]);
        host_command(&mut host, Action::Advance);
        host_command(&mut host, Action::Advance);
        wait_for_phase(&mut host, &mut guest, Phase::Playing);
        drop(guest);
        wait_until_guest_leaves(&mut host);

        assert_eq!(host.campaign.phase, Phase::Playing);
        assert_eq!(host.campaign.game.current_mission.asteroid_count, 14);
        let now = Instant::now();
        host.update(0.0, PilotInput::default(), now);
        host.campaign.transition(Phase::Success);
        host_command(&mut host, Action::Advance);
        host_command(&mut host, Action::Advance);
        assert_eq!(host.campaign.phase, Phase::Briefing);
        let mission = &host.campaign.game.current_mission;
        assert_eq!(mission.asteroid_count, 8);
        assert_eq!(mission.objectives[1].target(), 5);
    }

    #[test]
    fn guest_disconnect_unblocks_upgrade_bay_and_rescales_pending_briefing() {
        let mut host = create_host();
        let mut guest = connect(&mut host);
        host.campaign.transition(Phase::UpgradeBay);
        host.campaign.game.solo_pilot_mut().ship.scrap = 55;
        host_command(&mut host, Action::Ready);
        wait_for_phase(&mut host, &mut guest, Phase::UpgradeBay);
        guest_command(&mut guest, Action::Ready);
        wait_for(&mut host, &mut guest, |host, _| host.campaign.ready[GUEST]);
        assert!(host.campaign.ready[HOST]);
        host_command(&mut host, Action::Buy(UpgradeId::ReinforcedHull));
        assert_eq!(host.campaign.game.solo_pilot().ship.scrap, 35);

        drop(guest);
        wait_until_guest_leaves(&mut host);
        host_command(&mut host, Action::Buy(UpgradeId::WeaponTuning));
        assert_eq!(host.campaign.game.solo_pilot().ship.scrap, 5);
        host_command(&mut host, Action::Advance);
        assert_eq!(host.campaign.phase, Phase::Briefing);

        let mut host = create_host();
        let guest = connect(&mut host);
        host.campaign.ready[GUEST] = true;
        host_command(&mut host, Action::Advance);
        assert_eq!(host.campaign.game.current_mission.asteroid_count, 14);
        drop(guest);
        wait_until_guest_leaves(&mut host);
        assert_eq!(host.campaign.phase, Phase::Briefing);
        assert_eq!(host.campaign.game.current_mission.asteroid_count, 9);
    }

    fn wait_until_guest_leaves(host: &mut Host) {
        for _ in 0..500 {
            host.poll(Instant::now());
            if host.guests[GUEST].is_none() {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(host.status, Some(Status::GuestLost));
    }

    #[test]
    fn guest_reports_loss_of_host() {
        let mut host = create_host();
        let mut guest = connect(&mut host);
        drop(host);
        for _ in 0..500 {
            match guest.poll(Instant::now()) {
                Err(Status::HostLost) => return,
                Err(status) => panic!("unexpected status {status:?}"),
                Ok(_) => std::thread::sleep(Duration::from_millis(2)),
            }
        }
        panic!("guest did not detect loss of host");
    }

    #[test]
    fn surviving_pilot_can_finish_and_destroyed_pilot_respawns() {
        let mut campaign = Campaign::new(Difficulty::Nebula);
        campaign.add_guest(GUEST);
        campaign.game.solo_pilot_mut().ship.scrap = 21;
        assert!(matches!(
            campaign
                .game
                .solo_pilot_mut()
                .buy_upgrade(UpgradeId::ReinforcedHull),
            crate::game::BuyUpgradeResult::Bought
        ));

        campaign.game.start_mission();
        campaign.transition(Phase::Playing);
        campaign.game.solo_pilot_mut().ship.health = 0.0;
        campaign.game.mission_progress.enemies_defeated = 100;
        campaign.game.mission_progress.asteroids_destroyed = 100;
        campaign.simulate(0.05, || [PilotInput::default(); MAX_PILOTS]);
        assert_eq!(campaign.phase, Phase::Success);

        campaign.command(HOST, campaign.revision, Action::Advance);
        campaign.command(GUEST, campaign.revision, Action::Ready);
        campaign.command(HOST, campaign.revision, Action::Advance);
        campaign.command(HOST, campaign.revision, Action::Advance);
        let pilot = campaign.game.solo_pilot();
        assert!(pilot.is_alive());
        assert_eq!(pilot.ship.health, pilot.ship.max_health);
        assert_eq!(pilot.ship.scrap, 1);
        assert_eq!(pilot.upgrade_level(UpgradeId::ReinforcedHull), 1);
    }

    #[test]
    fn dreadnought_health_scales_but_its_wave_schedule_does_not() {
        let mut campaign = Campaign::new(Difficulty::Supernova);
        campaign.game.current_mission = get_mission(3);
        campaign.game.start_mission();
        let (hp, delay) = {
            let boss = campaign.game.boss.as_ref().unwrap();
            (boss.max_health, boss.summon_timer)
        };
        campaign.add_guest(GUEST);
        campaign.game.prepare_lan_mission();
        campaign.game.start_mission();
        let boss = campaign.game.boss.as_ref().unwrap();
        assert_eq!(boss.health, hp * 1.5);
        assert_eq!(boss.health, boss.max_health);
        assert_eq!(boss.summon_timer, delay);
        assert!(campaign
            .game
            .current_mission
            .objectives
            .contains(&MissionObjective::DefeatBoss));
    }
}
