use rand::Rng;

use crate::config::*;
use crate::snake::Snake;

pub struct GameMap {
    grid: [[char; MAP_WIDTH]; MAP_HEIGHT],
}

impl GameMap {
    pub fn new() -> Self {
        GameMap {
            grid: [[MAP_CHAR; MAP_WIDTH]; MAP_HEIGHT],
        }
    }

    pub fn place_food<R: Rng>(&self, snake: &mut Snake, rng: &mut R) {
        loop {
            let r = rng.gen_range(0..MAP_HEIGHT);
            let c = rng.gen_range(0..MAP_WIDTH);
            if self.grid[r][c] == MAP_CHAR && !snake.parts.contains(&(r, c)) {
                snake.food = (r, c);
                snake.food_eaten = false;
                return;
            }
        }
    }

    pub fn render(&mut self, snake: &Snake, settings: &Settings) -> String {
        // Clear grid
        for row in self.grid.iter_mut() {
            row.fill(MAP_CHAR);
        }

        // Draw snake body
        for &(r, c) in &snake.parts {
            self.grid[r][c] = settings.body;
        }

        // Draw head with directional glyph
        self.grid[snake.head.0][snake.head.1] = settings.head_char(snake.direction);

        // Draw food
        self.grid[snake.food.0][snake.food.1] = settings.food;

        // Build output string (use \r\n because we're in raw mode)
        let map_display_width = MAP_WIDTH * 2 - 1;
        let mut buf = String::with_capacity((MAP_HEIGHT + 2) * (MAP_WIDTH * 2 + 2));

        // Score line
        if !settings.hide_score {
            let score = format!("Score: {}", snake.length);
            let padding = if score.len() < map_display_width {
                (map_display_width - score.len()) / 2
            } else {
                0
            };
            buf.push_str(&" ".repeat(padding));
            buf.push_str(&score);
            buf.push_str("\r\n");
        }

        // Map rows — match C++ output: "char space" for each cell
        for row in &self.grid {
            for &cell in row.iter() {
                buf.push(cell);
                buf.push(' ');
            }
            buf.push_str("\r\n");
        }

        buf
    }
}
