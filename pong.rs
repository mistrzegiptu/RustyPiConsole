pub enum PongDirection {
    UpperRight,
    LowerRight,
    UpperLeft,
    LowerLeft
}

pub struct Point {
    pub x: i16,
    pub y: i16
}

pub struct Pong {
    pub ball: Point,
    pub ball_direction: PongDirection,
    pub player1: i16,
    pub player2: i16,
    pub player1_score: u8,
    pub player2_score: u8,
}

impl Pong {
    pub fn new(width: i16, height: i16) -> Self {
        let mut pong = Pong{
            ball: Point {x: width / 2, y: height / 2},
            ball_direction: PongDirection::UpperRight,
            player1: height / 2,
            player2: height / 2,
            player1_score: 0,
            player2_score: 0
        };
        pong.set_ball_direction(Self::random_direction());
        pong
    }

    pub fn random_direction() -> u8 {
        1 + (rand::random::<u8>() % 4)
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

    pub fn check_for_collision(mut self) {
        if self.ball.y == 0 || self.ball.y == height {
            self.change_at_wall();
            self.move_ball();
        }
    }
}