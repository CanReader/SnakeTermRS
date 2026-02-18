use clap::Parser;
use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT_MAP_WIDTH: usize = 20;
pub const DEFAULT_MAP_HEIGHT: usize = 20;
pub const MAP_CHAR: char = '.';
pub const WALL_CHAR: char = '#';
pub const INITIAL_SNAKE_LENGTH: usize = 3;
pub const BONUS_FOOD_CHAR: char = '$';
pub const BONUS_FOOD_SCORE: usize = 3;
pub const BONUS_FOOD_LIFETIME: usize = 30; // frames

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

    /// Number of random obstacles on the map
    #[arg(long, default_value_t = 0)]
    pub obstacles: usize,

    /// Enable multiplayer (player 2 uses arrow keys)
    #[arg(long)]
    pub multiplayer: bool,

    /// Enable speed increase as snake grows
    #[arg(long)]
    pub progressive_speed: bool,

    /// Enable shrinking border mode
    #[arg(long)]
    pub shrinking_border: bool,

    /// Map width (0 = auto-detect from terminal)
    #[arg(long, default_value_t = 0)]
    pub map_width: usize,

    /// Map height (0 = auto-detect from terminal)
    #[arg(long, default_value_t = 0)]
    pub map_height: usize,

    /// Path to TOML config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Record game to a replay file
    #[arg(long)]
    pub record: Option<PathBuf>,

    /// Play back a recorded replay file
    #[arg(long)]
    pub replay: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
pub struct FileConfig {
    pub speed: Option<u64>,
    pub body: Option<String>,
    pub head_w: Option<String>,
    pub head_n: Option<String>,
    pub head_e: Option<String>,
    pub head_s: Option<String>,
    pub head: Option<String>,
    pub food: Option<String>,
    pub seed: Option<u64>,
    pub hide_score: Option<bool>,
    pub auto_restart: Option<bool>,
    pub invert_controls: Option<bool>,
    pub disable_borders: Option<bool>,
    pub obstacles: Option<usize>,
    pub multiplayer: Option<bool>,
    pub progressive_speed: Option<bool>,
    pub shrinking_border: Option<bool>,
    pub map_width: Option<usize>,
    pub map_height: Option<usize>,
}

impl Settings {
    pub fn resolve(mut self) -> Self {
        // Load TOML config file if specified (CLI args override file values)
        if let Some(ref path) = self.config {
            if let Ok(contents) = std::fs::read_to_string(path) {
                if let Ok(fc) = toml::from_str::<FileConfig>(&contents) {
                    self.apply_file_config(&fc);
                }
            }
        }

        if let Some(ref h) = self.head {
            let chars: Vec<char> = h.chars().collect();
            if chars.len() >= 4 {
                self.head_w = chars[0];
                self.head_n = chars[1];
                self.head_e = chars[2];
                self.head_s = chars[3];
            }
        }

        // Auto-detect terminal size if map dimensions are 0
        if self.map_width == 0 || self.map_height == 0 {
            if let Ok((cols, rows)) = crossterm::terminal::size() {
                if self.map_width == 0 {
                    // Each cell is "char space" = 2 columns, leave margin
                    self.map_width = ((cols as usize).saturating_sub(4) / 2)
                        .min(40)
                        .max(10);
                }
                if self.map_height == 0 {
                    // Leave room for score line + game over text
                    self.map_height = ((rows as usize).saturating_sub(6))
                        .min(30)
                        .max(10);
                }
            } else {
                if self.map_width == 0 {
                    self.map_width = DEFAULT_MAP_WIDTH;
                }
                if self.map_height == 0 {
                    self.map_height = DEFAULT_MAP_HEIGHT;
                }
            }
        }

        self
    }

    fn apply_file_config(&mut self, fc: &FileConfig) {
        // File config only applies if CLI didn't override (check defaults)
        if let Some(v) = fc.speed { if self.speed == 200 { self.speed = v; } }
        if let Some(ref v) = fc.body { if self.body == '@' { self.body = v.chars().next().unwrap_or('@'); } }
        if let Some(ref v) = fc.head_w { if self.head_w == '<' { self.head_w = v.chars().next().unwrap_or('<'); } }
        if let Some(ref v) = fc.head_n { if self.head_n == '^' { self.head_n = v.chars().next().unwrap_or('^'); } }
        if let Some(ref v) = fc.head_e { if self.head_e == '>' { self.head_e = v.chars().next().unwrap_or('>'); } }
        if let Some(ref v) = fc.head_s { if self.head_s == 'v' { self.head_s = v.chars().next().unwrap_or('v'); } }
        if let Some(ref v) = fc.head { if self.head.is_none() { self.head = Some(v.clone()); } }
        if let Some(ref v) = fc.food { if self.food == '*' { self.food = v.chars().next().unwrap_or('*'); } }
        if let Some(v) = fc.seed { if self.seed == 0 { self.seed = v; } }
        if let Some(v) = fc.hide_score { if !self.hide_score { self.hide_score = v; } }
        if let Some(v) = fc.auto_restart { if !self.auto_restart { self.auto_restart = v; } }
        if let Some(v) = fc.invert_controls { if !self.invert_controls { self.invert_controls = v; } }
        if let Some(v) = fc.disable_borders { if !self.disable_borders { self.disable_borders = v; } }
        if let Some(v) = fc.obstacles { if self.obstacles == 0 { self.obstacles = v; } }
        if let Some(v) = fc.multiplayer { if !self.multiplayer { self.multiplayer = v; } }
        if let Some(v) = fc.progressive_speed { if !self.progressive_speed { self.progressive_speed = v; } }
        if let Some(v) = fc.shrinking_border { if !self.shrinking_border { self.shrinking_border = v; } }
        if let Some(v) = fc.map_width { if self.map_width == 0 { self.map_width = v; } }
        if let Some(v) = fc.map_height { if self.map_height == 0 { self.map_height = v; } }
    }

    pub fn head_char(&self, dir: Direction) -> char {
        match dir {
            Direction::West => self.head_w,
            Direction::North => self.head_n,
            Direction::East => self.head_e,
            Direction::South => self.head_s,
        }
    }

    pub fn effective_speed(&self, snake_length: usize) -> u64 {
        if self.progressive_speed {
            let reduction = ((snake_length.saturating_sub(INITIAL_SNAKE_LENGTH)) as u64) * 5;
            self.speed.saturating_sub(reduction).max(50)
        } else {
            self.speed
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
