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
    world: Vec<Vec<u8>>,
    pub map_width: usize,
    pub map_height: usize,
}

impl Snake {
    pub fn new(map_width: usize, map_height: usize) -> Self {
        let mut snake = Snake {
            parts: VecDeque::new(),
            head: (0, 0),
            food: (0, 0),
            food_eaten: false,
            is_dead: false,
            length: INITIAL_SNAKE_LENGTH,
            direction: Direction::East,
            input_queue: VecDeque::new(),
            world: vec![vec![0u8; map_width]; map_height],
            map_width,
            map_height,
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
        for row in self.world.iter_mut() {
            row.fill(0);
        }
        self.initialize();
    }

    fn initialize(&mut self) {
        let row = self.map_height / 2;
        let start_col = self.map_width / 2 - INITIAL_SNAKE_LENGTH / 2;
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

    pub fn update_movement(&mut self, settings: &Settings, walls: &[(usize, usize)]) {
        let (dr, dc) = self.direction.delta();
        let new_row = self.head.0 as i32 + dr;
        let new_col = self.head.1 as i32 + dc;

        let h = self.map_height;
        let w = self.map_width;

        let (new_row, new_col) = if settings.disable_borders {
            (
                ((new_row % h as i32 + h as i32) as usize) % h,
                ((new_col % w as i32 + w as i32) as usize) % w,
            )
        } else {
            if new_row < 0 || new_row >= h as i32 || new_col < 0 || new_col >= w as i32 {
                self.is_dead = true;
                return;
            }
            (new_row as usize, new_col as usize)
        };

        if walls.contains(&(new_row, new_col)) {
            self.is_dead = true;
            return;
        }

        self.head = (new_row, new_col);
        self.parts.push_back(self.head);

        self.food_eaten = self.head == self.food;
        if self.food_eaten {
            self.length += 1;
        } else {
            if let Some(tail) = self.parts.pop_front() {
                self.world[tail.0][tail.1] = self.world[tail.0][tail.1].saturating_sub(1);
            }
        }

        self.world[self.head.0][self.head.1] += 1;
        if self.world[self.head.0][self.head.1] > 1 {
            self.is_dead = true;
        }
    }
}
