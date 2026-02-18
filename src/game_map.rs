use crossterm::style::{Color, StyledContent, Stylize};
use rand::Rng;

use crate::config::*;
use crate::snake::Snake;

#[derive(Clone)]
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

pub struct BonusFood {
    pub pos: (usize, usize),
    pub lifetime: usize, // frames remaining
}

pub struct GameMap {
    pub width: usize,
    pub height: usize,
    grid: Vec<Vec<Cell>>,
    pub walls: Vec<(usize, usize)>,
    pub bonus_food: Option<BonusFood>,
    pub border_min: (usize, usize),
    pub border_max: (usize, usize),
    pub shrink_timer: usize,
}

impl GameMap {
    pub fn new(width: usize, height: usize) -> Self {
        GameMap {
            width,
            height,
            grid: vec![vec![Cell::empty(); width]; height],
            walls: Vec::new(),
            bonus_food: None,
            border_min: (0, 0),
            border_max: (height, width),
            shrink_timer: 0,
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
        let (bmin_r, bmin_c) = self.border_min;
        let (bmax_r, bmax_c) = self.border_max;
        loop {
            let r = rng.gen_range(bmin_r..bmax_r);
            let c = rng.gen_range(bmin_c..bmax_c);
            if !snake.parts.contains(&(r, c)) && !self.walls.contains(&(r, c)) {
                snake.food = (r, c);
                snake.food_eaten = false;
                return;
            }
        }
    }

    pub fn maybe_spawn_bonus<R: Rng>(&mut self, snake: &Snake, rng: &mut R) {
        if self.bonus_food.is_some() {
            return;
        }
        // ~5% chance per frame
        if rng.gen_range(0..20) != 0 {
            return;
        }
        let (bmin_r, bmin_c) = self.border_min;
        let (bmax_r, bmax_c) = self.border_max;
        for _ in 0..50 {
            let r = rng.gen_range(bmin_r..bmax_r);
            let c = rng.gen_range(bmin_c..bmax_c);
            if !snake.parts.contains(&(r, c))
                && !self.walls.contains(&(r, c))
                && (r, c) != snake.food
            {
                self.bonus_food = Some(BonusFood {
                    pos: (r, c),
                    lifetime: BONUS_FOOD_LIFETIME,
                });
                return;
            }
        }
    }

    pub fn tick_bonus(&mut self) {
        if let Some(ref mut bonus) = self.bonus_food {
            bonus.lifetime = bonus.lifetime.saturating_sub(1);
            if bonus.lifetime == 0 {
                self.bonus_food = None;
            }
        }
    }

    pub fn check_bonus_eaten(&mut self, snake: &mut Snake) -> bool {
        if let Some(ref bonus) = self.bonus_food {
            if snake.head == bonus.pos {
                snake.score += BONUS_FOOD_SCORE;
                snake.length += 1;
                self.bonus_food = None;
                return true;
            }
        }
        false
    }

    pub fn update_shrinking_border(&mut self, snake: &Snake) {
        self.shrink_timer += 1;
        // Shrink every 50 frames
        if self.shrink_timer % 50 != 0 {
            return;
        }
        let (min_r, min_c) = self.border_min;
        let (max_r, max_c) = self.border_max;
        let eff_h = max_r - min_r;
        let eff_w = max_c - min_c;
        // Don't shrink below 6x6
        if eff_h <= 6 || eff_w <= 6 {
            return;
        }
        // Alternate shrinking sides
        let step = self.shrink_timer / 50;
        match step % 4 {
            0 => self.border_min.0 = (min_r + 1).min(max_r.saturating_sub(6)),
            1 => self.border_max.1 = max_c.saturating_sub(1).max(min_c + 6),
            2 => self.border_max.0 = max_r.saturating_sub(1).max(min_r + 6),
            3 => self.border_min.1 = (min_c + 1).min(max_c.saturating_sub(6)),
            _ => {}
        }
        // Remove walls outside new borders
        self.walls.retain(|&(r, c)| {
            r >= self.border_min.0 && r < self.border_max.0
            && c >= self.border_min.1 && c < self.border_max.1
        });
        let _ = snake; // snake position checked elsewhere
    }

    pub fn render(
        &mut self,
        snakes: &[&Snake],
        settings: &Settings,
        paused: bool,
        frame_count: usize,
    ) -> String {
        // Clear grid
        for r in 0..self.height {
            for c in 0..self.width {
                let (bmin_r, bmin_c) = self.border_min;
                let (bmax_r, bmax_c) = self.border_max;
                if r < bmin_r || r >= bmax_r || c < bmin_c || c >= bmax_c {
                    self.grid[r][c] = Cell::wall();
                } else {
                    self.grid[r][c] = Cell::empty();
                }
            }
        }

        // Draw walls
        for &(r, c) in &self.walls {
            self.grid[r][c] = Cell::wall();
        }

        // Draw snake(s)
        let snake_colors = [Color::Green, Color::Cyan];
        let head_colors = [Color::Yellow, Color::Magenta];

        for (idx, snake) in snakes.iter().enumerate() {
            let body_color = snake_colors[idx % snake_colors.len()];
            let hd_color = head_colors[idx % head_colors.len()];

            for &(r, c) in &snake.parts {
                if r < self.height && c < self.width {
                    self.grid[r][c] = Cell { ch: settings.body, color: body_color };
                }
            }
            // Head
            if snake.head.0 < self.height && snake.head.1 < self.width {
                self.grid[snake.head.0][snake.head.1] = Cell {
                    ch: settings.head_char(snake.direction),
                    color: hd_color,
                };
            }
        }

        // Draw food (from first snake)
        if let Some(s) = snakes.first() {
            if s.food.0 < self.height && s.food.1 < self.width {
                self.grid[s.food.0][s.food.1] = Cell { ch: settings.food, color: Color::Red };
            }
        }

        // Draw bonus food
        if let Some(ref bonus) = self.bonus_food {
            let (r, c) = bonus.pos;
            if r < self.height && c < self.width {
                // Blink effect: alternate color every few frames
                let blink_color = if (frame_count / 3) % 2 == 0 { Color::Magenta } else { Color::Yellow };
                self.grid[r][c] = Cell { ch: BONUS_FOOD_CHAR, color: blink_color };
            }
        }

        // Build output string with ANSI colors
        let mut buf = String::with_capacity((self.height + 4) * (self.width * 2 + 20));

        // Score line
        if !settings.hide_score {
            let score_text = if snakes.len() > 1 {
                format!("P1: {}  P2: {}", snakes[0].score, snakes[1].score)
            } else {
                format!("Score: {}", snakes[0].score)
            };
            let map_display_width = self.width * 2;
            let padding = if score_text.len() < map_display_width {
                (map_display_width - score_text.len()) / 2
            } else {
                0
            };
            buf.push_str(&" ".repeat(padding));
            let styled: StyledContent<&str> = score_text.as_str().with(Color::White);
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

        if paused {
            let pause_msg = "  ** PAUSED — press P or Space to resume **";
            let styled: StyledContent<&str> = pause_msg.with(Color::Yellow);
            buf.push_str(&format!("{styled}\r\n"));
        }

        buf
    }

    pub fn render_death_animation(
        &mut self,
        snakes: &[&Snake],
        settings: &Settings,
        frame: usize,
    ) -> String {
        // Flash snake between red and dark on alternating frames
        // Clear grid
        for r in 0..self.height {
            for c in 0..self.width {
                let (bmin_r, bmin_c) = self.border_min;
                let (bmax_r, bmax_c) = self.border_max;
                if r < bmin_r || r >= bmax_r || c < bmin_c || c >= bmax_c {
                    self.grid[r][c] = Cell::wall();
                } else {
                    self.grid[r][c] = Cell::empty();
                }
            }
        }

        for &(r, c) in &self.walls {
            self.grid[r][c] = Cell::wall();
        }

        let flash_color = if frame % 2 == 0 { Color::Red } else { Color::DarkRed };

        for snake in snakes {
            for &(r, c) in &snake.parts {
                if r < self.height && c < self.width {
                    self.grid[r][c] = Cell { ch: settings.body, color: flash_color };
                }
            }
            if snake.head.0 < self.height && snake.head.1 < self.width {
                self.grid[snake.head.0][snake.head.1] = Cell {
                    ch: 'X',
                    color: flash_color,
                };
            }
        }

        // Food
        if let Some(s) = snakes.first() {
            if s.food.0 < self.height && s.food.1 < self.width {
                self.grid[s.food.0][s.food.1] = Cell { ch: settings.food, color: Color::Red };
            }
        }

        let mut buf = String::with_capacity((self.height + 4) * (self.width * 2 + 20));

        if !settings.hide_score {
            let score_text = if snakes.len() > 1 {
                format!("P1: {}  P2: {}", snakes[0].score, snakes[1].score)
            } else {
                format!("Score: {}", snakes[0].score)
            };
            let map_display_width = self.width * 2;
            let padding = if score_text.len() < map_display_width {
                (map_display_width - score_text.len()) / 2
            } else {
                0
            };
            buf.push_str(&" ".repeat(padding));
            let styled: StyledContent<&str> = score_text.as_str().with(Color::White);
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
