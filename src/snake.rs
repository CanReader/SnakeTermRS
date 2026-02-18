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
    pub score: usize,
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
            score: 0,
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
        self.score = 0;
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
        // Buffer up to 3 inputs for smooth turning
        if self.input_queue.len() < 3 {
            // Check against the last queued direction (or current) to avoid reversals
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

    pub fn update_movement(&mut self, settings: &Settings, walls: &[(usize, usize)], border_min: (usize, usize), border_max: (usize, usize)) {
        let (dr, dc) = self.direction.delta();
        let new_row = self.head.0 as i32 + dr;
        let new_col = self.head.1 as i32 + dc;

        let (bmin_r, bmin_c) = border_min;
        let (bmax_r, bmax_c) = border_max;
        let eff_h = bmax_r - bmin_r;
        let eff_w = bmax_c - bmin_c;

        let (new_row, new_col) = if settings.disable_borders {
            (
                (((new_row - bmin_r as i32) % eff_h as i32 + eff_h as i32) as usize % eff_h) + bmin_r,
                (((new_col - bmin_c as i32) % eff_w as i32 + eff_w as i32) as usize % eff_w) + bmin_c,
            )
        } else {
            if new_row < bmin_r as i32
                || new_row >= bmax_r as i32
                || new_col < bmin_c as i32
                || new_col >= bmax_c as i32
            {
                self.is_dead = true;
                return;
            }
            (new_row as usize, new_col as usize)
        };

        // Check wall collision
        if walls.contains(&(new_row, new_col)) {
            self.is_dead = true;
            return;
        }

        self.head = (new_row, new_col);
        self.parts.push_back(self.head);

        self.food_eaten = self.head == self.food;
        if self.food_eaten {
            self.length += 1;
            self.score += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_snake_initial_length() {
        let snake = Snake::new(20, 20);
        assert_eq!(snake.parts.len(), INITIAL_SNAKE_LENGTH);
        assert_eq!(snake.length, INITIAL_SNAKE_LENGTH);
        assert!(!snake.is_dead);
    }

    #[test]
    fn test_snake_reset() {
        let mut snake = Snake::new(20, 20);
        snake.score = 10;
        snake.length = 15;
        snake.is_dead = true;
        snake.reset();
        assert_eq!(snake.length, INITIAL_SNAKE_LENGTH);
        assert_eq!(snake.score, 0);
        assert!(!snake.is_dead);
    }

    #[test]
    fn test_snake_direction_queue() {
        let mut snake = Snake::new(20, 20);
        // Initial direction is East
        // Can't queue West (opposite)
        snake.queue_direction(Direction::West);
        assert!(snake.input_queue.is_empty());
        // Can queue North
        snake.queue_direction(Direction::North);
        assert_eq!(snake.input_queue.len(), 1);
        // Can't queue same direction twice in a row
        snake.queue_direction(Direction::North);
        assert_eq!(snake.input_queue.len(), 1);
    }

    #[test]
    fn test_snake_movement_basic() {
        let settings = Settings::parse_from::<[&str; 0], &str>([]);
        let mut settings = settings.resolve();
        settings.map_width = 20;
        settings.map_height = 20;
        let mut snake = Snake::new(20, 20);
        let head_before = snake.head;
        snake.update_movement(&settings, &[], (0, 0), (20, 20));
        // Heading East: column should increase by 1
        assert_eq!(snake.head.0, head_before.0);
        assert_eq!(snake.head.1, head_before.1 + 1);
    }

    #[test]
    fn test_snake_wall_collision() {
        let settings = Settings::parse_from::<[&str; 0], &str>([]);
        let mut settings = settings.resolve();
        settings.map_width = 20;
        settings.map_height = 20;
        let mut snake = Snake::new(20, 20);
        // Place wall right in front of the snake
        let wall = (snake.head.0, snake.head.1 + 1);
        snake.update_movement(&settings, &[wall], (0, 0), (20, 20));
        assert!(snake.is_dead);
    }

    #[test]
    fn test_snake_border_death() {
        let settings = Settings::parse_from::<[&str; 0], &str>([]);
        let mut settings = settings.resolve();
        settings.map_width = 20;
        settings.map_height = 20;
        let mut snake = Snake::new(20, 20);
        // Move snake to right edge
        for _ in 0..20 {
            if snake.is_dead { break; }
            snake.update_movement(&settings, &[], (0, 0), (20, 20));
        }
        assert!(snake.is_dead);
    }

    #[test]
    fn test_snake_wrap_around() {
        let settings = Settings::parse_from(&["test", "--disable-borders"]);
        let mut settings = settings.resolve();
        settings.map_width = 20;
        settings.map_height = 20;
        let mut snake = Snake::new(20, 20);
        // Move snake to right edge and beyond — should wrap
        for _ in 0..20 {
            snake.update_movement(&settings, &[], (0, 0), (20, 20));
            if snake.is_dead { break; }
        }
        assert!(!snake.is_dead);
    }

    #[test]
    fn test_snake_food_eating() {
        let settings = Settings::parse_from::<[&str; 0], &str>([]);
        let mut settings = settings.resolve();
        settings.map_width = 20;
        settings.map_height = 20;
        let mut snake = Snake::new(20, 20);
        let old_length = snake.length;
        // Place food right in front
        snake.food = (snake.head.0, snake.head.1 + 1);
        snake.update_movement(&settings, &[], (0, 0), (20, 20));
        assert!(snake.food_eaten);
        assert_eq!(snake.length, old_length + 1);
        assert_eq!(snake.score, 1);
    }
}
