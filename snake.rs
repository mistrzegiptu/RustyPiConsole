use rand::Rng;

pub const SNAKE_INITIAL_LENGTH: usize = 3;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq,Clone, Copy, Debug)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

#[derive(Debug)]
pub struct Snake {
    pub width: i16,
    pub height: i16,
    pub head_position: Point,
    pub body: Vec<Point>,
    pub direction: Direction,
    pub score: u8,
    pub alive: bool,
    pub ate: bool,
    pub food: Vec<Point>,
}


impl Snake {
    pub fn new(width: i16, height: i16) -> Self {
        let mut snake = Snake {
            width,
            height,
            head_position: Point { x: width / 2, y: height / 2 },
            body: vec![Point { x: width / 2, y: height / 2 }, Point { x: width / 2 + 1, y: height / 2 }, Point { x: width / 2 + 2, y: height / 2 }],
            direction: Direction::Left,
            score: SNAKE_INITIAL_LENGTH as u8,
            alive: true,
            ate: false,
            food: Vec::new(),
        };

        snake
    }

    pub fn change_direction(&mut self, new_direction: Direction) {
        match (self.direction ,new_direction) {
            (Direction::Up, Direction::Down) => return,
            (Direction::Down, Direction::Up) => return,
            (Direction::Left, Direction::Right) => return,
            (Direction::Right, Direction::Left) => return,
            _ => self.direction = new_direction,
        }
    }

    pub fn move_snake(&mut self) {
        let new_head = match self.direction {
            Direction::Up => Point { x: self.head_position.x, y: self.head_position.y + 1 },
            Direction::Down => Point { x: self.head_position.x, y: self.head_position.y - 1 },
            Direction::Left => Point { x: self.head_position.x - 1, y: self.head_position.y },
            Direction::Right => Point { x: self.head_position.x + 1, y: self.head_position.y },
        };

        if new_head.x < 0 || new_head.x >= self.width || new_head.y < 0 || new_head.y >= self.height {
            self.alive = false;
            return;
        }

        if self.body.contains(&new_head) {
            self.alive = false;
            return;
        }

        self.body.insert(0, new_head);
        if !self.ate {
            self.body.pop();
        } else {
            self.ate = false;
        }
        self.head_position = new_head;
    }


    // Napisać dodając do głowy i jak jest na skraju to skręca odpowiednio
    pub fn eat(&mut self) {
        self.score += 1;
        self.ate = true;
    }

    //Losowanie pozycji jedzenia
    pub fn random_food_position(&mut self) {
        let mut rng = rand::thread_rng();

        loop {
            let x = rng.gen_range(0..self.width);
            let y = rng.gen_range(0..self.height);
            let point = Point { x, y };

            if !self.body.contains(&point) && !self.food.contains(&point) {
                self.food.push(point);
                break;
            }
        }
    }
}