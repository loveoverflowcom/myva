pub(crate) const ARENA_WIDTH: f32 = 640.0;
pub(crate) const ARENA_HEIGHT: f32 = 360.0;
const PLAYER_HALF_SIZE: f32 = 14.0;
const PLAYER_SPEED: f32 = 180.0;
const TARGETS: [(f32, f32); 6] = [
    (0.0, 0.0),
    (230.0, 90.0),
    (150.0, -105.0),
    (-175.0, -95.0),
    (-230.0, 110.0),
    (70.0, 115.0),
];

#[derive(Clone, Copy)]
pub(crate) struct GameState {
    pub x: f32,
    pub y: f32,
    pub score: u32,
    pub frames: u64,
    pub updates: u64,
    pub paused: bool,
    pub ready: bool,
    input: (f32, f32),
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            x: -220.0,
            y: 0.0,
            score: 0,
            frames: 0,
            updates: 0,
            paused: false,
            ready: false,
            input: (0.0, 0.0),
        }
    }
}

impl GameState {
    pub fn set_input(&mut self, x: f32, y: f32) {
        if !x.is_finite() || !y.is_finite() {
            self.input = (0.0, 0.0);
            return;
        }
        // Clamp first so even very large external inputs cannot overflow length.
        let x = x.clamp(-1.0, 1.0);
        let y = y.clamp(-1.0, 1.0);
        let length = x.hypot(y).max(1.0);
        self.input = (x / length, y / length);
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if paused {
            self.input = (0.0, 0.0);
        }
    }

    pub fn reset(&mut self) {
        *self = Self {
            paused: self.paused,
            ready: self.ready,
            updates: self.updates,
            ..Self::default()
        };
    }

    pub fn target(&self) -> (f32, f32) {
        TARGETS[self.score as usize % TARGETS.len()]
    }

    pub fn step(&mut self, delta_seconds: f32) {
        self.ready = true;
        self.updates = self.updates.saturating_add(1);
        if self.paused || !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }
        // A background-tab stall must not teleport through a target on resume.
        let travel = PLAYER_SPEED * delta_seconds.min(0.1);
        let limit_x = ARENA_WIDTH / 2.0 - PLAYER_HALF_SIZE - 8.0;
        let limit_y = ARENA_HEIGHT / 2.0 - PLAYER_HALF_SIZE - 8.0;
        self.x = (self.x + self.input.0 * travel).clamp(-limit_x, limit_x);
        self.y = (self.y + self.input.1 * travel).clamp(-limit_y, limit_y);
        self.frames = self.frames.saturating_add(1);

        let target = self.target();
        if (self.x - target.0).abs() < 25.0 && (self.y - target.1).abs() < 25.0 {
            self.score = self.score.saturating_add(1);
        }
    }

    pub fn telemetry(&self) -> String {
        format!(
            "{{\"x\":{:.3},\"y\":{:.3},\"score\":{},\"frames\":{},\"updates\":{},\"paused\":{},\"ready\":{}}}",
            self.x, self.y, self.score, self.frames, self.updates, self.paused, self.ready,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_stays_in_arena_and_diagonals_keep_the_same_speed() {
        let mut diagonal = GameState::default();
        diagonal.set_input(1.0, 1.0);
        diagonal.step(0.1);
        assert!(((diagonal.x + 220.0).hypot(diagonal.y) - 18.0).abs() < 0.001);
        for _ in 0..100 {
            diagonal.step(0.1);
        }
        assert_eq!((diagonal.x, diagonal.y), (298.0, 158.0));
    }

    #[test]
    fn touching_a_target_scores_once_and_moves_the_target() {
        let mut game = GameState::default();
        game.set_input(1.0, 0.0);
        for _ in 0..12 {
            game.step(0.1);
        }
        assert_eq!(game.score, 1);
        assert_eq!(game.target(), (230.0, 90.0));
        game.set_input(0.0, 0.0);
        game.step(0.1);
        assert_eq!(game.score, 1);
    }

    #[test]
    fn pause_freezes_simulation_and_clears_held_input() {
        let mut game = GameState::default();
        game.set_input(1.0, 0.0);
        game.set_paused(true);
        game.step(0.1);
        assert_eq!((game.x, game.frames), (-220.0, 0));
        assert_eq!(game.updates, 1);
        assert!(game.ready);
        game.set_paused(false);
        game.step(0.1);
        assert_eq!(game.x, -220.0);
    }

    #[test]
    fn reset_preserves_host_lifecycle_and_clears_game_progress() {
        let mut game = GameState {
            x: 100.0,
            score: 3,
            frames: 500,
            updates: 600,
            paused: true,
            ready: true,
            input: (1.0, 0.0),
            ..GameState::default()
        };
        game.reset();
        assert_eq!((game.x, game.score, game.frames), (-220.0, 0, 0));
        assert_eq!(game.updates, 600);
        assert!(game.paused && game.ready);
        assert_eq!(game.input, (0.0, 0.0));
    }

    #[test]
    fn invalid_external_input_and_large_frame_gaps_stay_bounded() {
        let mut game = GameState::default();
        game.set_input(f32::NAN, f32::INFINITY);
        game.step(0.1);
        assert_eq!(game.x, -220.0);
        game.set_input(f32::MAX, 0.0);
        game.step(300.0);
        assert_eq!(game.x, -202.0);
        game.step(f32::NAN);
        assert_eq!(game.x, -202.0);
        assert!(!game.telemetry().contains("NaN"));
    }
}
