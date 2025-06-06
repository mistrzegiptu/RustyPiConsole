use ::{JOY_LOWER_BOUND, JOY_UPPER_BOUND};

pub const PLAYER_SIZE: i16 = 2;
const MAX_SCORE: u8 = 11;

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
    pub is_running: bool
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
            is_running: true
        };
        pong.set_ball_direction(Self::random_direction());
        pong
    }

    pub fn random_direction() -> u8 {
        //1 + (rand::random::<u8>() % 4) TODO: Make rand work
        1
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

    pub fn move_ball(&mut self) {
        self.ball = match self.ball_direction {
            PongDirection::UpperRight => Point{x: self.ball.x+1, y: self.ball.y+1},
            PongDirection::LowerRight => Point{x: self.ball.x+1, y: self.ball.y-1},
            PongDirection::LowerLeft => Point{x: self.ball.x-1, y: self.ball.y-1},
            PongDirection::UpperLeft => Point{x: self.ball.x-1, y: self.ball.y+1},
        };
    }

    pub fn move_player(&mut self, which_player: PlayerTurn, value: i16) {
        let dy = if value > JOY_UPPER_BOUND as i16 { 1 } else if value < JOY_LOWER_BOUND as i16 { -1 } else { 0 };

        match which_player {
            PlayerTurn::Player1 => {
                self.player1 += dy;

                if self.player1 == (0 + PLAYER_SIZE - 1){
                    self.player1 = PLAYER_SIZE
                }
                else if self.player1 == (self.height - PLAYER_SIZE + 1) {
                    self.player1 = self.height - PLAYER_SIZE
                }
            }
            PlayerTurn::Player2 => {
                self.player2 += dy;

                if self.player1 == (0 + PLAYER_SIZE - 1){
                    self.player1 = PLAYER_SIZE
                }
                else if self.player1 == (self.height - PLAYER_SIZE + 1) {
                    self.player1 = self.height - PLAYER_SIZE
                }
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

    pub fn check_for_collision(&mut self) {
        if self.ball.y == 0 || self.ball.y == self.height {
            self.change_at_wall();
        }
        else if (self.ball.x == 0 && i16::abs(self.ball.y - self.player1) <= PLAYER_SIZE) ||
                (self.ball.x == self.width && i16::abs(self.ball.y - self.player2) <= PLAYER_SIZE){

            self.change_at_player();
        }
        else if self.ball.x == 0 && i16::abs(self.ball.y - self.player1) > PLAYER_SIZE {
            self.score(PlayerTurn::Player2)
        }
        else if self.ball.x == self.width && i16::abs(self.ball.y - self.player2) > PLAYER_SIZE {
            self.score(PlayerTurn::Player1)
        }
    }
}