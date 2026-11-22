use super::session::{Phase, Snapshot, STEP};
use crate::components::Ship;
use crate::game::{advance_ship_movement, Game, Pilot, PilotInput};
use macroquad::prelude::Vec2;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const MAX_PENDING_STEPS: usize = 8;
const CORRECTION_RATE: f32 = 20.0;
const MAX_SMOOTH_DISTANCE: f32 = 96.0;
const MAX_SMOOTH_ANGLE: f32 = 45.0;

// The host holds each input sample until the next one arrives. Count the
// simulation steps under that sample so a heartbeat cannot acknowledge input.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub(super) struct InputProgress {
    pub sequence: u64,
    pub steps: u32,
}

#[derive(Default)]
pub(super) struct Prediction {
    revision: u64,
    input: PilotInput,
    progress: InputProgress,
    pending: VecDeque<(InputProgress, PilotInput)>,
    remainder: f64,
    position_correction: Vec2,
    rotation_correction: f32,
}

impl Prediction {
    pub fn set_input(&mut self, revision: u64, input: PilotInput) -> u64 {
        let new_revision = self.revision != revision;
        if new_revision {
            self.clear_movement();
            self.revision = revision;
        }
        if new_revision || self.progress.sequence == 0 || self.input != input {
            self.progress.sequence += 1;
            self.progress.steps = 0;
            self.input = input;
        }
        self.progress.sequence
    }

    pub fn reconcile(
        &mut self,
        previous: Option<&Snapshot<Game>>,
        state: &Snapshot<Game>,
        slot: usize,
    ) {
        let Some(pilot) = state.game.pilots[slot]
            .as_ref()
            .filter(|pilot| pilot.is_alive() && state.phase == Phase::Playing)
        else {
            self.clear_movement();
            return;
        };
        let Some(previous) = previous.filter(|previous| {
            previous.revision == state.revision
                && self.revision == state.revision
                && previous.phase == Phase::Playing
        }) else {
            self.clear_movement();
            return;
        };
        let Some(old_pilot) = previous.game.pilots[slot]
            .as_ref()
            .filter(|pilot| pilot.is_alive())
        else {
            self.clear_movement();
            return;
        };

        let before = self.project(old_pilot, previous.game.bounds);
        let acknowledged = state.input_ack[slot];
        while self
            .pending
            .front()
            .is_some_and(|(step, _)| *step <= acknowledged)
        {
            self.pending.pop_front();
        }
        if acknowledged.sequence == self.progress.sequence {
            self.progress.steps = self.progress.steps.max(acknowledged.steps);
        }
        let after = self.project(pilot, state.game.bounds);
        let position = before.pos + self.position_correction - after.pos;
        let rotation = before.rotation + self.rotation_correction - after.rotation;
        let rotation = (rotation + 180.0).rem_euclid(360.0) - 180.0;
        // Large changes, including crossing the arena edge, need to snap.
        self.position_correction = if position.length_squared() <= MAX_SMOOTH_DISTANCE.powi(2) {
            position
        } else {
            Vec2::ZERO
        };
        self.rotation_correction = if rotation.abs() <= MAX_SMOOTH_ANGLE {
            rotation
        } else {
            0.0
        };
    }

    pub fn advance(
        &mut self,
        revision: u64,
        pilot: &Pilot,
        bounds: Vec2,
        input: PilotInput,
        dt: f32,
    ) -> Option<Ship> {
        self.set_input(revision, input);
        if !pilot.is_alive() {
            return None;
        }

        let dt = dt.clamp(0.0, 0.25);
        self.remainder += f64::from(dt);
        while self.remainder >= STEP && self.pending.len() < MAX_PENDING_STEPS {
            self.remainder -= STEP;
            self.progress.steps += 1;
            self.pending.push_back((self.progress, self.input));
        }
        if self.pending.len() == MAX_PENDING_STEPS {
            self.remainder = 0.0;
        }
        let decay = (-CORRECTION_RATE * dt).exp();
        self.position_correction *= decay;
        self.rotation_correction *= decay;

        let mut ship = self.project(pilot, bounds);
        ship.pos += self.position_correction;
        ship.rotation += self.rotation_correction;
        Some(ship)
    }

    fn clear_movement(&mut self) {
        self.pending.clear();
        self.remainder = 0.0;
        self.position_correction = Vec2::ZERO;
        self.rotation_correction = 0.0;
    }

    fn project(&self, pilot: &Pilot, bounds: Vec2) -> Ship {
        let mut ship = pilot.ship.clone();
        let acceleration = pilot.acceleration();
        for &(_, input) in &self.pending {
            advance_ship_movement(&mut ship, acceleration, input, bounds, STEP as f32);
        }
        // Render the remaining fraction of the current step on fast displays.
        if self.remainder > 0.0 {
            advance_ship_movement(
                &mut ship,
                acceleration,
                self.input,
                bounds,
                self.remainder as f32,
            );
        }
        ship
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lan::session::FIELD;

    #[test]
    fn prediction_stops_after_the_pending_limit_and_resets_for_a_new_mission() {
        let game = Game::new(FIELD);
        let pilot = game.solo_pilot();
        let input = PilotInput {
            left: true,
            thrust: true,
            ..PilotInput::default()
        };
        let mut prediction = Prediction::default();
        let at_limit = prediction.advance(1, pilot, FIELD, input, 0.25).unwrap();
        let stalled = prediction.advance(1, pilot, FIELD, input, 0.25).unwrap();
        assert_eq!(prediction.pending.len(), MAX_PENDING_STEPS);
        assert_eq!(stalled.pos, at_limit.pos);
        assert_eq!(stalled.rotation, at_limit.rotation);

        let next_mission = prediction
            .advance(2, pilot, FIELD, PilotInput::default(), 0.0)
            .unwrap();
        assert!(prediction.pending.is_empty());
        assert_eq!(next_mission.pos, pilot.ship.pos);
        assert_eq!(next_mission.rotation, pilot.ship.rotation);
    }
}
