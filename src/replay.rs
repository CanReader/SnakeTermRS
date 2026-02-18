use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::config::Direction;

pub struct Recorder {
    frames: Vec<Option<Direction>>,
}

impl Recorder {
    pub fn new() -> Self {
        Recorder { frames: Vec::new() }
    }

    pub fn record_frame(&mut self, dir: Option<Direction>) {
        self.frames.push(dir);
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let mut f = fs::File::create(path)?;
        for frame in &self.frames {
            let ch = match frame {
                Some(Direction::North) => 'N',
                Some(Direction::South) => 'S',
                Some(Direction::East) => 'E',
                Some(Direction::West) => 'W',
                None => '.',
            };
            writeln!(f, "{ch}")?;
        }
        Ok(())
    }
}

pub struct Player {
    frames: Vec<Option<Direction>>,
    index: usize,
}

impl Player {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let f = fs::File::open(path)?;
        let reader = BufReader::new(f);
        let mut frames = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let dir = match line.trim() {
                "N" => Some(Direction::North),
                "S" => Some(Direction::South),
                "E" => Some(Direction::East),
                "W" => Some(Direction::West),
                _ => None,
            };
            frames.push(dir);
        }
        Ok(Player { frames, index: 0 })
    }

    pub fn next_frame(&mut self) -> Option<Option<Direction>> {
        if self.index < self.frames.len() {
            let val = self.frames[self.index];
            self.index += 1;
            Some(val)
        } else {
            None // replay finished
        }
    }
}
