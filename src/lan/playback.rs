use super::session::{Phase, Snapshot, STEP};
use crate::game::Game;
use std::collections::VecDeque;

const BUFFER_TICKS: f64 = 3.0;
const MAX_BUFFERED_FRAMES: usize = 8;
const REFILL_RATE: f64 = 0.9;

#[derive(Default)]
pub(super) struct Playback {
    history: VecDeque<Snapshot<Game>>,
    render_tick: Option<f64>,
    refilling: bool,
}

impl Playback {
    pub fn record(&mut self, previous: Snapshot<Game>, next: &Snapshot<Game>, slot: usize) {
        let local_alive = |state: &Snapshot<Game>| {
            state.game.pilots[slot]
                .as_ref()
                .is_some_and(|pilot| pilot.is_alive())
        };
        if previous.phase != Phase::Playing
            || next.phase != Phase::Playing
            || previous.revision != next.revision
            || previous.tick > next.tick
            || local_alive(&previous) != local_alive(next)
            || previous.names != next.names
        {
            self.history.clear();
            self.render_tick = None;
            self.refilling = false;
        } else if previous.tick < next.tick {
            if self.history.len() == MAX_BUFFERED_FRAMES {
                self.history.pop_front();
            }
            self.history.push_back(previous);
        }
    }

    pub fn world<'a>(&'a mut self, latest: &'a Snapshot<Game>, dt: f32) -> &'a Game {
        if latest.phase != Phase::Playing {
            return &latest.game;
        }

        let latest_tick = latest.tick as f64;
        let desired_tick = latest_tick - BUFFER_TICKS;
        let tick = self.render_tick.map_or(desired_tick, |previous| {
            // Rebuild the buffer gradually after a stall, without rewinding the scene.
            let elapsed = f64::from(dt.clamp(0.0, 0.25)) / STEP;
            let rate = if self.refilling { REFILL_RATE } else { 1.0 };
            (previous + elapsed * rate).clamp(desired_tick, latest_tick)
        });
        self.render_tick = Some(tick);
        self.refilling = tick > desired_tick && (self.refilling || tick == latest_tick);

        while self
            .history
            .get(1)
            .is_some_and(|frame| frame.tick as f64 <= tick)
        {
            self.history.pop_front();
        }
        if tick >= latest_tick {
            return &latest.game;
        }
        self.history
            .front()
            .map_or(&latest.game, |frame| &frame.game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lan::session::FIELD;

    fn snapshot(tick: u64, revision: u64) -> Snapshot<Game> {
        let mut game = Game::new(FIELD);
        game.score = tick as u32;
        Snapshot {
            game,
            phase: Phase::Playing,
            revision,
            tick,
            input_ack: Default::default(),
            ready: Default::default(),
            names: Default::default(),
            mvp: None,
        }
    }

    #[test]
    fn plays_intermediate_frames_when_snapshots_arrive_in_a_burst() {
        let mut playback = Playback::default();
        let mut latest = snapshot(8, 1);
        for tick in 9..=12 {
            let next = snapshot(tick, 1);
            playback.record(latest, &next, 0);
            latest = next;
        }

        assert_eq!(playback.world(&latest, 0.0).score, 9);
        assert_eq!(playback.world(&latest, STEP as f32).score, 10);
        assert_eq!(playback.world(&latest, STEP as f32).score, 11);
        assert_eq!(playback.world(&latest, STEP as f32).score, 12);
    }

    #[test]
    fn resumes_after_a_stall_without_rewinding_and_resets_on_new_revision() {
        let mut playback = Playback::default();
        let before_stall = snapshot(10, 1);
        assert_eq!(playback.world(&before_stall, 0.25).score, 10);
        assert_eq!(playback.world(&before_stall, 0.25).score, 10);

        let resumed = snapshot(11, 1);
        playback.record(before_stall, &resumed, 0);
        assert_eq!(playback.world(&resumed, STEP as f32).score, 10);

        let new_revision = snapshot(0, 2);
        playback.record(resumed, &new_revision, 0);
        assert!(playback.history.is_empty());
        assert_eq!(playback.world(&new_revision, 0.0).score, 0);
    }
}
