use oorandom::Rand32;
use ::{JOY_LOWER_BOUND, JOY_UPPER_BOUND};

pub const PLAYER_SIZE: i16 = 4;
const MAX_SCORE: u8 = 11;
const PLAYER_MOVE_DELTA: i16 = 2;

pub enum PongDirection {
    UpperRight,
    LowerRight,
    UpperLeft,
    LowerLeft
}

pub enum PlayerTurn {
    Player1 = 0,
    Player2 = 1
}

pub struct Point {
    pub x: i16,
    pub y: i16
}

pub struct Pong {
    pub width: i16,
    pub height: i16,
    pub ball: Point,
    pub ball_direction: PongDirection,
    pub player1: i16,
    pub player2: i16,
    pub player1_score: u8,
    pub player2_score: u8,
    pub is_running: bool,
    pub rng: Rand32
}

impl Pong {
    pub fn new(width: i16, height: i16) -> Self {
        let mut pong = Pong{
            width,
            height,
            ball: Point {x: width / 2, y: height / 2},
            ball_direction: PongDirection::UpperRight,
            player1: height / 2,
            player2: height / 2,
            player1_score: 0,
            player2_score: 0,
            is_running: true,
            rng: Rand32::new(2137)
        };
        let direction = pong.random_direction();
        pong.set_ball_direction(direction);
        pong
    }

    pub fn random_direction(&mut self) -> u8 {
        self.rng.rand_range(0..4) as u8
        //1
    }

    pub fn set_ball_direction(&mut self, direction: u8) {
        self.ball_direction = match direction {
            1 => PongDirection::UpperRight,
            2 => PongDirection::LowerRight,
            3 => PongDirection::LowerLeft,
            4 => PongDirection::UpperLeft,
            _ => PongDirection::UpperRight
        };
    }

    pub fn change_at_wall(&mut self) {
        self.ball_direction = match self.ball_direction {
            PongDirection::UpperRight => PongDirection::LowerRight,
            PongDirection::LowerRight => PongDirection::UpperRight,
            PongDirection::LowerLeft => PongDirection::UpperLeft,
            PongDirection::UpperLeft => PongDirection::LowerLeft,
        };
    }

    pub fn change_at_player(&mut self) {
        self.ball_direction = match self.ball_direction {
            PongDirection::UpperRight => PongDirection::UpperLeft,
            PongDirection::LowerRight => PongDirection::LowerLeft,
            PongDirection::LowerLeft => PongDirection::LowerRight,
            PongDirection::UpperLeft => PongDirection::UpperRight,
        };
    }

    pub fn move_player(&mut self, which_player: PlayerTurn, value: i16) {
        let dy = if value > JOY_UPPER_BOUND as i16 { PLAYER_MOVE_DELTA } else if value < JOY_LOWER_BOUND as i16 { -PLAYER_MOVE_DELTA } else { 0 };

        match which_player {
            PlayerTurn::Player1 => {
                self.player1 = (self.player1 + dy).clamp(PLAYER_SIZE, self.height - PLAYER_SIZE);
            }
            PlayerTurn::Player2 => {
                self.player2 = (self.player2 + dy).clamp(PLAYER_SIZE, self.height - PLAYER_SIZE);
            }
        }
    }

    pub fn score(&mut self, which_player: PlayerTurn) {
        match which_player {
            PlayerTurn::Player1 => self.player1_score += 1,
            PlayerTurn::Player2 => self.player2_score += 1
        }
    }

    pub fn check_for_win(&mut self) {
        if self.player1_score == MAX_SCORE || self.player2_score == MAX_SCORE {
            self.is_running = false
        }
    }

    pub fn spawn_ball(&mut self) {
        let direction = self.random_direction();
        self.set_ball_direction(direction);
        self.ball = Point { x: self.width / 2, y: self.height / 2 };
    }

    pub fn update_ball(&mut self) {
        let (next_x, next_y) = match self.ball_direction {
            PongDirection::UpperRight => (self.ball.x + 1, self.ball.y + 1),
            PongDirection::LowerRight => (self.ball.x + 1, self.ball.y - 1),
            PongDirection::LowerLeft => (self.ball.x - 1, self.ball.y - 1),
            PongDirection::UpperLeft => (self.ball.x - 1, self.ball.y + 1),
        };

        if next_y <= 0 || next_y >= self.height - 1 {
            self.change_at_wall();
        }

        if (next_x == 0 && i16::abs(next_y - self.player1) <= PLAYER_SIZE) ||
            (next_x == self.width - 1 && i16::abs(next_y - self.player2) <= PLAYER_SIZE) {
            self.change_at_player();
        }

        if next_x < 0 {
            self.score(PlayerTurn::Player2);
            self.spawn_ball();
            return;
        } else if next_x > self.width {
            self.score(PlayerTurn::Player1);
            self.spawn_ball();
            return;
        }

        self.ball = Point { x: next_x, y: next_y };
    }

}