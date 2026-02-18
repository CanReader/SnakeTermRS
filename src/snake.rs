use std::collections::VecDeque;

use crate::config::*;

pub struct Snake {
    pub parts: VecDeque<(usize, usize)>,
    pub head: (usize, usize),
    pub food: (usize, usize),
    pub food_eaten: bool,
    pub is_dead: bool,
    pub length: usize,
    pub direction: Direction,
    pub input_queue: VecDeque<Direction>,
    world: [[u8; MAP_WIDTH]; MAP_HEIGHT],
}

impl Snake {
    pub fn new() -> Self {
        let mut snake = Snake {
            parts: VecDeque::new(),
            head: (0, 0),
            food: (0, 0),
            food_eaten: false,
            is_dead: false,
            length: INITIAL_SNAKE_LENGTH,
            direction: Direction::East,
            input_queue: VecDeque::new(),
            world: [[0; MAP_WIDTH]; MAP_HEIGHT],
        };
        snake.initialize();
        snake
    }

    pub fn reset(&mut self) {
        self.direction = Direction::East;
        self.input_queue.clear();
        self.food_eaten = false;
        self.is_dead = false;
        self.length = INITIAL_SNAKE_LENGTH;
        self.parts.clear();
        self.world = [[0; MAP_WIDTH]; MAP_HEIGHT];
        self.initialize();
    }

    fn initialize(&mut self) {
        let row = MAP_HEIGHT / 2;
        let start_col = MAP_WIDTH / 2 - INITIAL_SNAKE_LENGTH / 2;
        for i in 0..INITIAL_SNAKE_LENGTH {
            let pos = (row, start_col + i);
            self.parts.push_back(pos);
            self.world[pos.0][pos.1] = 1;
        }
        self.head = *self.parts.back().unwrap();
    }

    pub fn queue_direction(&mut self, dir: Direction) {
        if self.input_queue.len() < 3 {
            let last = self.input_queue.back().copied().unwrap_or(self.direction);
            if dir != last.opposite() && dir != last {
                self.input_queue.push_back(dir);
            }
        }
    }

    pub fn apply_queued_input(&mut self) {
        if let Some(next) = self.input_queue.pop_front() {
            if next != self.direction.opposite() {
                self.direction = next;
            }
        }
    }

    pub fn update_movement(&mut self, settings: &Settings) {
        let (dr, dc) = self.direction.delta();
        let new_row = self.head.0 as i32 + dr;
        let new_col = self.head.1 as i32 + dc;

        let (new_row, new_col) = if settings.disable_borders {
            (
                ((new_row % MAP_HEIGHT as i32) + MAP_HEIGHT as i32) as usize % MAP_HEIGHT,
                ((new_col % MAP_WIDTH as i32) + MAP_WIDTH as i32) as usize % MAP_WIDTH,
            )
        } else {
            if new_row < 0
                || new_row >= MAP_HEIGHT as i32
                || new_col < 0
                || new_col >= MAP_WIDTH as i32
            {
                self.is_dead = true;
                return;
            }
            (new_row as usize, new_col as usize)
        };

        self.head = (new_row, new_col);
        self.parts.push_back(self.head);

        self.food_eaten = self.head == self.food;
        if self.food_eaten {
            self.length += 1;
        } else {
            if let Some(tail) = self.parts.pop_front() {
                self.world[tail.0][tail.1] -= 1;
            }
        }

        self.world[self.head.0][self.head.1] += 1;
        if self.world[self.head.0][self.head.1] > 1 {
            self.is_dead = true;
        }
    }
}
