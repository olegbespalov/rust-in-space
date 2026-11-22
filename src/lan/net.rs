use if_addrs::IfAddr;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::{self, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream, UdpSocket};
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

pub(super) const PORT: u16 = 41616;
pub(super) const TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const COMMAND_LIMIT: usize = 4 * 1024;
pub(super) const SNAPSHOT_LIMIT: usize = 1024 * 1024;
const IO_BUDGET: usize = 128 * 1024;
const FRAME_BUDGET: usize = 64;
const QUEUE_LIMIT: usize = 32;
const COMPACT_FRAME: u32 = 1 << 31;
const DISCOVERY_INTERVAL: Duration = Duration::from_secs(1);
const DISCOVERY_EXPIRY: Duration = Duration::from_secs(3);
const DISCOVER: &[u8] = b"rust-in-space/lan/1/find";
const OFFER: &[u8] = b"rust-in-space/lan/1/offer";

pub(super) fn endpoint(ip: Ipv4Addr) -> SocketAddrV4 {
    SocketAddrV4::new(ip, PORT)
}

fn interfaces() -> Vec<if_addrs::Ifv4Addr> {
    if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|interface| match interface.addr {
            IfAddr::V4(addr) if !addr.ip.is_loopback() && !addr.ip.is_unspecified() => Some(addr),
            _ => None,
        })
        .collect()
}

pub(super) fn local_addresses() -> Vec<Ipv4Addr> {
    interfaces()
        .into_iter()
        .map(|addr| addr.ip)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn listen(address: SocketAddrV4) -> io::Result<TcpListener> {
    let listener = TcpListener::bind(address)?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

pub(super) fn connect(address: SocketAddrV4) -> io::Result<Receiver<io::Result<TcpStream>>> {
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("lan-connect".into())
        .spawn(move || {
            let result = TcpStream::connect_timeout(&address.into(), TIMEOUT);
            let _ = sender.send(result);
        })?;
    Ok(receiver)
}

pub(super) struct Connection {
    stream: TcpStream,
    reader: FrameReader,
    output: VecDeque<Outbound>,
    closed: bool,
    write_progress: Instant,
    pub last_received: Instant,
}

#[derive(Clone)]
pub(super) struct Frame(Rc<Vec<u8>>);

impl Frame {
    pub fn compact(message: &impl Serialize, limit: usize) -> io::Result<Self> {
        Ok(Self(Rc::new(encode(message, limit, true)?)))
    }
}

struct Outbound {
    encoded: Frame,
    written: usize,
    replaceable: bool,
}

impl Connection {
    pub fn new(stream: TcpStream, receive_limit: usize, now: Instant) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        Ok(Self {
            stream,
            reader: FrameReader::new(receive_limit),
            output: VecDeque::new(),
            closed: false,
            write_progress: now,
            last_received: now,
        })
    }

    pub fn queue(
        &mut self,
        message: &impl Serialize,
        limit: usize,
        replaceable: bool,
        now: Instant,
    ) -> io::Result<()> {
        self.queue_frame(Frame::compact(message, limit)?, replaceable, now)
    }

    pub fn queue_handshake(
        &mut self,
        message: &impl Serialize,
        limit: usize,
        now: Instant,
    ) -> io::Result<()> {
        self.queue_frame(Frame(Rc::new(encode(message, limit, false)?)), false, now)
    }

    pub fn queue_frame(&mut self, frame: Frame, replaceable: bool, now: Instant) -> io::Result<()> {
        if self.output.is_empty() {
            self.write_progress = now;
        }
        if replaceable {
            self.output
                .retain(|frame| !frame.replaceable || frame.written != 0);
        }
        if self.output.len() >= QUEUE_LIMIT {
            return Err(io::Error::new(ErrorKind::WouldBlock, "LAN send queue full"));
        }
        self.output.push_back(Outbound {
            encoded: frame,
            written: 0,
            replaceable,
        });
        Ok(())
    }

    pub fn flush(&mut self, now: Instant) -> io::Result<()> {
        let mut budget = IO_BUDGET;
        while let Some(frame) = self.output.front_mut().filter(|_| budget > 0) {
            let bytes = &frame.encoded.0;
            let end = (frame.written + budget).min(bytes.len());
            match self.stream.write(&bytes[frame.written..end]) {
                Ok(0) => return Err(ErrorKind::WriteZero.into()),
                Ok(n) => {
                    self.write_progress = now;
                    budget -= n;
                    frame.written += n;
                    if frame.written == bytes.len() {
                        self.output.pop_front();
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        if !self.output.is_empty() && now.duration_since(self.write_progress) >= TIMEOUT {
            return Err(ErrorKind::TimedOut.into());
        }
        Ok(())
    }

    pub fn is_flushed(&self) -> bool {
        self.output.is_empty()
    }

    pub fn receive<T: DeserializeOwned>(&mut self, now: Instant) -> io::Result<Vec<T>> {
        let mut messages = Vec::new();
        let mut budget = IO_BUDGET;
        let mut chunk = [0; 8192];
        while messages.len() < FRAME_BUDGET {
            if let Some(message) = self.reader.next()? {
                self.last_received = now;
                messages.push(message);
                continue;
            }
            if self.closed || budget == 0 {
                break;
            }
            let limit = chunk.len().min(budget);
            match self.stream.read(&mut chunk[..limit]) {
                Ok(0) => self.closed = true,
                Ok(n) => {
                    self.reader.bytes.extend_from_slice(&chunk[..n]);
                    budget -= n;
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
        if messages.is_empty() {
            if self.closed {
                return Err(ErrorKind::UnexpectedEof.into());
            }
            if now.duration_since(self.last_received) >= TIMEOUT {
                return Err(ErrorKind::TimedOut.into());
            }
        }
        Ok(messages)
    }
}

fn encode(message: &impl Serialize, limit: usize, compact: bool) -> io::Result<Vec<u8>> {
    let mut frame = vec![0; 4];
    if compact {
        rmp_serde::encode::write(&mut frame, message).map_err(io::Error::other)?;
    } else {
        serde_json::to_writer(&mut frame, message)?;
    }
    let len = frame.len() - 4;
    if len > limit {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "LAN frame too large",
        ));
    }
    // Keep JSON handshakes compatible so older peers receive a version rejection.
    let header = len as u32 | if compact { COMPACT_FRAME } else { 0 };
    frame[..4].copy_from_slice(&header.to_be_bytes());
    Ok(frame)
}

struct FrameReader {
    bytes: Vec<u8>,
    limit: usize,
}

impl FrameReader {
    fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }

    fn next<T: DeserializeOwned>(&mut self) -> io::Result<Option<T>> {
        let Some(header) = self.bytes.first_chunk::<4>() else {
            return Ok(None);
        };
        let header = u32::from_be_bytes(*header);
        let len = (header & !COMPACT_FRAME) as usize;
        if len == 0 || len > self.limit {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "invalid LAN frame length",
            ));
        }
        let Some(body) = self.bytes.get(4..4 + len) else {
            return Ok(None);
        };
        let message = if header & COMPACT_FRAME != 0 {
            rmp_serde::from_slice(body)
                .map_err(|error| io::Error::new(ErrorKind::InvalidData, error))?
        } else {
            serde_json::from_slice(body)?
        };
        self.bytes.drain(..4 + len);
        Ok(Some(message))
    }
}

pub(super) struct Advertiser(UdpSocket);

impl Advertiser {
    pub fn new(address: SocketAddrV4) -> io::Result<Self> {
        let socket = UdpSocket::bind(address)?;
        socket.set_nonblocking(true)?;
        Ok(Self(socket))
    }

    pub fn poll(&self, available: bool) -> io::Result<()> {
        let mut bytes = [0; 64];
        for _ in 0..FRAME_BUDGET {
            match self.0.recv_from(&mut bytes) {
                Ok((size, from)) if available && &bytes[..size] == DISCOVER => {
                    if let Err(error) = self.0.send_to(OFFER, from) {
                        if !matches!(
                            error.kind(),
                            ErrorKind::WouldBlock
                                | ErrorKind::Interrupted
                                | ErrorKind::NetworkUnreachable
                        ) {
                            return Err(error);
                        }
                    }
                }
                Ok(_) => {}
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}

pub(super) struct Discovery {
    socket: UdpSocket,
    targets: BTreeSet<SocketAddrV4>,
    hosts: BTreeMap<SocketAddrV4, Instant>,
    next_probe: Instant,
}

impl Discovery {
    pub fn new(now: Instant) -> io::Result<Self> {
        let targets = interfaces()
            .into_iter()
            .filter_map(|addr| addr.broadcast)
            .chain([Ipv4Addr::LOCALHOST, Ipv4Addr::BROADCAST])
            .map(endpoint)
            .collect();
        Self::with_targets(targets, now)
    }

    fn with_targets(targets: BTreeSet<SocketAddrV4>, now: Instant) -> io::Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
        socket.set_nonblocking(true)?;
        socket.set_broadcast(true)?;
        Ok(Self {
            socket,
            targets,
            hosts: BTreeMap::new(),
            next_probe: now,
        })
    }

    pub fn poll(&mut self, now: Instant) {
        if now >= self.next_probe {
            for &target in &self.targets {
                let _ = self.socket.send_to(DISCOVER, target);
            }
            self.next_probe = now + DISCOVERY_INTERVAL;
        }

        self.hosts
            .retain(|_, seen| now.duration_since(*seen) < DISCOVERY_EXPIRY);
        let mut bytes = [0; 64];
        for _ in 0..FRAME_BUDGET {
            let Ok((size, SocketAddr::V4(from))) = self.socket.recv_from(&mut bytes) else {
                break;
            };
            if &bytes[..size] == OFFER
                && self
                    .targets
                    .iter()
                    .any(|target| target.port() == from.port())
                && (self.hosts.len() < 64 || self.hosts.contains_key(&from))
            {
                self.hosts.insert(from, now);
            }
        }
    }

    pub fn hosts(&self) -> impl Iterator<Item = SocketAddrV4> + '_ {
        self.hosts.keys().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_handle_partial_reads_and_coalesced_messages() {
        let a = encode(&42, COMMAND_LIMIT, false).unwrap();
        let b = encode(&43, COMMAND_LIMIT, true).unwrap();
        let mut reader = FrameReader::new(COMMAND_LIMIT);
        reader.bytes.extend_from_slice(&a[..2]);
        assert_eq!(reader.next::<u32>().unwrap(), None);
        reader.bytes.extend_from_slice(&a[2..5]);
        assert_eq!(reader.next::<u32>().unwrap(), None);
        reader.bytes.extend_from_slice(&a[5..]);
        reader.bytes.extend_from_slice(&b);
        assert_eq!(reader.next::<u32>().unwrap(), Some(42));
        assert_eq!(reader.next::<u32>().unwrap(), Some(43));
        assert_eq!(reader.next::<u32>().unwrap(), None);
    }

    #[test]
    fn frames_reject_invalid_sizes_and_json() {
        for bytes in [
            &[0, 0, 0, 0][..],
            &[0, 0, 0, 9][..],
            &[0, 0, 0, 1, b'!'][..],
            &[0x80, 0, 0, 9][..],
            &[0x80, 0, 0, 1, 0xc1][..],
        ] {
            let mut reader = FrameReader::new(8);
            reader.bytes.extend_from_slice(bytes);
            assert!(reader.next::<u32>().is_err());
        }
        assert!(encode(&1234, 3, false).is_err());
    }

    #[test]
    fn combat_snapshot_is_smaller_with_messagepack() {
        use super::super::session::{Phase, Snapshot, FIELD};
        use crate::components::{Bullet, BulletStyle, ShipKind};
        use crate::game::{Game, MAX_PILOTS};
        use macroquad::prelude::vec2;

        let mut game = Game::new(FIELD);
        game.set_pilot(1, ShipKind::Delta);
        game.prepare_lan_mission();
        game.start_mission();
        for index in 0..24 {
            game.bullets.push(Bullet {
                pos: vec2(180.25 + 30.0 * index as f32, 250.5),
                vel: vec2(480.0, 90.0),
                life_time: 1.5,
                style: BulletStyle::Player,
                homing: false,
                damage: 10.0,
                radius: 4.0,
            });
        }
        let state = Snapshot {
            game: &game,
            phase: Phase::Playing,
            revision: 2,
            ready: [false; MAX_PILOTS],
        };
        let json = encode(&state, SNAPSHOT_LIMIT, false).unwrap();
        let compact = encode(&state, SNAPSHOT_LIMIT, true).unwrap();
        println!(
            "Combat snapshot: {} B JSON, {} B MessagePack",
            json.len(),
            compact.len()
        );
        assert!(compact.len() * 2 < json.len());
        const V3_FRAME_BYTES: usize = 1890;
        assert!(compact.len() * 5 <= V3_FRAME_BYTES * 4);

        let mut reader = FrameReader::new(SNAPSHOT_LIMIT);
        reader.bytes = compact;
        let received = reader.next::<Snapshot<Game>>().unwrap().unwrap();
        assert_eq!(received.phase, Phase::Playing);
        assert_eq!(received.game.bullets.len(), game.bullets.len());
        assert_eq!(received.game.asteroids.len(), game.asteroids.len());
        let bullet = &received.game.bullets[23];
        assert_eq!(bullet.pos, game.bullets[23].pos);
        assert_eq!(bullet.vel, game.bullets[23].vel);
        assert_eq!(bullet.radius, game.bullets[23].radius);
        assert_eq!(bullet.life_time, 0.0);
        assert_eq!(bullet.damage, 0.0);
    }

    #[test]
    fn compact_game_state_preserves_visual_effects() {
        use super::super::session::FIELD;
        use crate::components::{
            Boss, BossPhase, BossSummon, Explosion, LootItem, LootType, ShieldKind,
        };
        use crate::game::Game;
        use macroquad::prelude::{vec2, Vec2};

        let mut game = Game::new(FIELD);
        let ship = &mut game.solo_pilot_mut().ship;
        ship.activate_shield(ShieldKind::Golden, 30.0, 2.0);
        ship.invisibility_timer = 1.25;
        ship.engine.current_thrust = 0.75;
        game.loot_items.push(LootItem {
            pos: vec2(200.0, 100.0),
            vel: Vec2::ZERO,
            drift_vel: Vec2::ZERO,
            item_type: LootType::Scrap(7),
            time_left: 2.5,
            radius: 8.0,
            magnet_active: true,
            rotation: 0.5,
            rotation_speed: 1.0,
        });
        let mut explosion = Explosion::new(vec2(250.0, 100.0), 1.5);
        explosion.frame = 3;
        game.explosions.push(explosion);
        let mut boss = Boss::new(vec2(400.0, 100.0), 2.0);
        boss.phase = BossPhase::Escalated;
        boss.pending_summon = Some(BossSummon {
            from_left: true,
            y: 150.0,
            remaining: 0.75,
        });
        boss.power_ups.pick_up(
            &LootType::Shield(ShieldKind::Golden, 30),
            &mut boss.health,
            boss.max_health,
        );
        game.boss = Some(boss);

        let mut reader = FrameReader::new(SNAPSHOT_LIMIT);
        reader.bytes = encode(&game, SNAPSHOT_LIMIT, true).unwrap();
        let received = reader.next::<Game>().unwrap().unwrap();
        let ship = &received.solo_pilot().ship;
        assert!(ship.has_shield());
        assert_eq!(ship.invisibility_timer, 1.25);
        assert_eq!(ship.engine.current_thrust, 0.75);
        assert_eq!(ship.engine.offset, game.solo_pilot().ship.engine.offset);
        assert_eq!(received.loot_items[0].time_left, 2.5);
        assert_eq!(received.loot_items[0].rotation, 0.5);
        assert_eq!(received.explosions[0].frame, 3);
        assert_eq!(
            received.explosions[0].max_frames,
            game.explosions[0].max_frames
        );
        let boss = received.boss.unwrap();
        assert_eq!(boss.phase, BossPhase::Escalated);
        assert_eq!(boss.pending_summon.unwrap().remaining, 0.75);
        assert_eq!(boss.power_ups.shield.unwrap().hp, 30.0);
    }

    #[test]
    fn discovery_finds_a_loopback_lobby_and_expires_its_offer() {
        let now = Instant::now();
        let advertiser = Advertiser::new(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let SocketAddr::V4(address) = advertiser.0.local_addr().unwrap() else {
            unreachable!()
        };
        let mut browser = Discovery::with_targets(BTreeSet::from([address]), now).unwrap();
        for _ in 0..30 {
            browser.poll(now);
            advertiser.poll(true).unwrap();
            if browser.hosts().any(|host| host == address) {
                break;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(browser.hosts().collect::<Vec<_>>(), vec![address]);
        browser.poll(now + DISCOVERY_EXPIRY);
        advertiser.poll(false).unwrap();
        assert_eq!(browser.hosts().count(), 0);
    }
}
