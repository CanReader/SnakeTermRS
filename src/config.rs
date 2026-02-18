use clap::Parser;

pub const MAP_WIDTH: usize = 20;
pub const MAP_HEIGHT: usize = 20;
pub const MAP_CHAR: char = '.';
pub const INITIAL_SNAKE_LENGTH: usize = 3;

#[derive(Parser, Debug, Clone)]
#[command(name = "snake-term", about = "Terminal Snake game written in Rust")]
pub struct Settings {
    /// Frame delay in milliseconds (smaller = faster)
    #[arg(long, default_value_t = 200)]
    pub speed: u64,

    /// Snake body character
    #[arg(long, default_value_t = '@')]
    pub body: char,

    /// Head glyph when moving west (left)
    #[arg(long, default_value_t = '<')]
    pub head_w: char,

    /// Head glyph when moving north (up)
    #[arg(long, default_value_t = '^')]
    pub head_n: char,

    /// Head glyph when moving east (right)
    #[arg(long, default_value_t = '>')]
    pub head_e: char,

    /// Head glyph when moving south (down)
    #[arg(long, default_value_t = 'v')]
    pub head_s: char,

    /// Set all 4 head chars as WNES sequence (e.g. '<^>v')
    #[arg(long)]
    pub head: Option<String>,

    /// Food glyph
    #[arg(long, default_value_t = '*')]
    pub food: char,

    /// RNG seed (0 = use time)
    #[arg(long, default_value_t = 0)]
    pub seed: u64,

    /// Hide the score display
    #[arg(long)]
    pub hide_score: bool,

    /// Automatically restart on game over
    #[arg(long)]
    pub auto_restart: bool,

    /// Invert movement controls
    #[arg(long)]
    pub invert_controls: bool,

    /// Enable wrap-around (pass from edge to opposite)
    #[arg(long)]
    pub disable_borders: bool,
}

impl Settings {
    pub fn resolve(mut self) -> Self {
        if let Some(ref h) = self.head {
            let chars: Vec<char> = h.chars().collect();
            if chars.len() >= 4 {
                self.head_w = chars[0];
                self.head_n = chars[1];
                self.head_e = chars[2];
                self.head_s = chars[3];
            }
        }
        self
    }

    pub fn head_char(&self, dir: Direction) -> char {
        match dir {
            Direction::West => self.head_w,
            Direction::North => self.head_n,
            Direction::East => self.head_e,
            Direction::South => self.head_s,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    West,
    North,
    East,
    South,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::West => Direction::East,
            Direction::East => Direction::West,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
        }
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::West => (0, -1),
            Direction::East => (0, 1),
            Direction::North => (-1, 0),
            Direction::South => (1, 0),
        }
    }
}
