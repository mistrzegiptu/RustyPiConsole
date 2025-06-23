use heapless::Vec;
use oorandom::Rand32;

pub const SNAKE_INITIAL_LENGTH: usize = 3;
pub const MAX_VEC_SIZE: usize = 100;

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
    pub body: Vec<Point,MAX_VEC_SIZE>,
    pub direction: Direction,
    pub score: u8,
    pub alive: bool,
    pub ate: bool,
    pub food: Vec<Point,MAX_VEC_SIZE>,
    pub won: bool,
    pub rng: Rand32
}


impl Snake {
    pub fn new(width: i16, height: i16, seed: u64) -> Self {
        let mut body = Vec::new();

        let mid_x = width/2;
        let mid_y = height/2;

        body.push(Point { x:mid_x, y: mid_y }).unwrap();
        body.push(Point { x:mid_x + 1, y: mid_y }).unwrap();
        body.push(Point { x:mid_x + 2, y: mid_y }).unwrap();


        Snake {
            width,
            height,
            head_position: Point { x: width / 2, y: height / 2 },
            body,
            direction: Direction::Left,
            score: SNAKE_INITIAL_LENGTH as u8,
            alive: true,
            ate: false,
            food: Vec::new(),
            won: false,
            rng: Rand32::new(seed)
        }
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

        if self.body.insert(0, new_head).is_err(){
            self.won = true;
            return;
        }

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
        
        let mut index = None;

        for (i,f) in self.food.iter().enumerate() {
            if *f == self.head_position {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            self.food.remove(i);
        }
    }

    //Losowanie pozycji jedzenia, napisać jakiś test
    pub fn random_food_position(&mut self) {

        loop {
             let x = self.rng.rand_range(0..self.width as u32) as i16;
             let y = self.rng.rand_range(0..self.height as u32) as i16;
             let point = Point { x, y};

             if !self.body.contains(&point) && !self.food.contains(&point) {
                 self.food.push(point);
                 break;
             }
        }

        let p1 = Point { x: 3, y: 0 };
        let p2 = Point { x: 1, y: 1 };
        let p3 = Point { x: 2, y: 2 };

        self.food.push(p1);
        self.food.push(p2);
        self.food.push(p3);
    }
}