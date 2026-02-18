use crossterm::style::{Color, StyledContent, Stylize};
use rand::Rng;

use crate::config::*;
use crate::snake::Snake;

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub color: Color,
}

impl Cell {
    fn empty() -> Self {
        Cell { ch: MAP_CHAR, color: Color::DarkGrey }
    }
}

pub struct GameMap {
    grid: [[Cell; MAP_WIDTH]; MAP_HEIGHT],
}

impl GameMap {
    pub fn new() -> Self {
        GameMap {
            grid: [[Cell::empty(); MAP_WIDTH]; MAP_HEIGHT],
        }
    }

    pub fn place_food<R: Rng>(&self, snake: &mut Snake, rng: &mut R) {
        loop {
            let r = rng.gen_range(0..MAP_HEIGHT);
            let c = rng.gen_range(0..MAP_WIDTH);
            if self.grid[r][c].ch == MAP_CHAR && !snake.parts.contains(&(r, c)) {
                snake.food = (r, c);
                snake.food_eaten = false;
                return;
            }
        }
    }

    pub fn render(&mut self, snake: &Snake, settings: &Settings) -> String {
        // Clear grid
        for row in self.grid.iter_mut() {
            row.fill(Cell::empty());
        }

        // Draw snake body
        for &(r, c) in &snake.parts {
            self.grid[r][c] = Cell { ch: settings.body, color: Color::Green };
        }

        // Draw head with directional glyph
        self.grid[snake.head.0][snake.head.1] = Cell {
            ch: settings.head_char(snake.direction),
            color: Color::Yellow,
        };

        // Draw food
        self.grid[snake.food.0][snake.food.1] = Cell { ch: settings.food, color: Color::Red };

        // Build output string with ANSI colors
        let map_display_width = MAP_WIDTH * 2 - 1;
        let mut buf = String::with_capacity((MAP_HEIGHT + 2) * (MAP_WIDTH * 2 + 20));

        // Score line
        if !settings.hide_score {
            let score = format!("Score: {}", snake.length);
            let padding = if score.len() < map_display_width {
                (map_display_width - score.len()) / 2
            } else {
                0
            };
            buf.push_str(&" ".repeat(padding));
            let styled: StyledContent<&str> = score.as_str().with(Color::White);
            buf.push_str(&format!("{styled}"));
            buf.push_str("\r\n");
        }

        // Map rows
        for row in &self.grid {
            for cell in row.iter() {
                let styled: StyledContent<String> = cell.ch.to_string().with(cell.color);
                buf.push_str(&format!("{styled} "));
            }
            buf.push_str("\r\n");
        }

        buf
    }
}
