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
    fn wall() -> Self {
        Cell { ch: WALL_CHAR, color: Color::White }
    }
}

pub struct GameMap {
    pub width: usize,
    pub height: usize,
    grid: Vec<Vec<Cell>>,
    pub walls: Vec<(usize, usize)>,
}

impl GameMap {
    pub fn new(width: usize, height: usize) -> Self {
        GameMap {
            width,
            height,
            grid: vec![vec![Cell::empty(); width]; height],
            walls: Vec::new(),
        }
    }

    pub fn place_walls<R: Rng>(&mut self, count: usize, snake: &Snake, rng: &mut R) {
        self.walls.clear();
        for _ in 0..count {
            loop {
                let r = rng.gen_range(0..self.height);
                let c = rng.gen_range(0..self.width);
                if !snake.parts.contains(&(r, c))
                    && (r, c) != snake.food
                    && !self.walls.contains(&(r, c))
                {
                    self.walls.push((r, c));
                    break;
                }
            }
        }
    }

    pub fn place_food<R: Rng>(&self, snake: &mut Snake, rng: &mut R) {
        loop {
            let r = rng.gen_range(0..self.height);
            let c = rng.gen_range(0..self.width);
            if !snake.parts.contains(&(r, c)) && !self.walls.contains(&(r, c)) {
                snake.food = (r, c);
                snake.food_eaten = false;
                return;
            }
        }
    }

    pub fn render(&mut self, snake: &Snake, settings: &Settings) -> String {
        for row in self.grid.iter_mut() {
            row.fill(Cell::empty());
        }

        for &(r, c) in &self.walls {
            self.grid[r][c] = Cell::wall();
        }

        for &(r, c) in &snake.parts {
            self.grid[r][c] = Cell { ch: settings.body, color: Color::Green };
        }

        self.grid[snake.head.0][snake.head.1] = Cell {
            ch: settings.head_char(snake.direction),
            color: Color::Yellow,
        };

        self.grid[snake.food.0][snake.food.1] = Cell { ch: settings.food, color: Color::Red };

        let map_display_width = self.width * 2;
        let mut buf = String::with_capacity((self.height + 4) * (self.width * 2 + 20));

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
