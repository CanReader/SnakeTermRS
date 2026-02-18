use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::config::{Direction, Settings};

pub enum GameInput {
    Move(Direction),
    Quit,
    None,
}

pub fn poll_input(settings: &Settings, timeout: Duration) -> GameInput {
    if !event::poll(timeout).unwrap_or(false) {
        return GameInput::None;
    }

    match event::read() {
        Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) => {
            // Ctrl+C to quit
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                return GameInput::Quit;
            }

            let dir = match code {
                KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => {
                    if settings.invert_controls {
                        Direction::South
                    } else {
                        Direction::North
                    }
                }
                KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                    if settings.invert_controls {
                        Direction::North
                    } else {
                        Direction::South
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => {
                    if settings.invert_controls {
                        Direction::East
                    } else {
                        Direction::West
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => {
                    if settings.invert_controls {
                        Direction::West
                    } else {
                        Direction::East
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    return GameInput::Quit;
                }
                _ => return GameInput::None,
            };

            GameInput::Move(dir)
        }
        _ => GameInput::None,
    }
}

pub enum GameOverInput {
    Restart,
    Quit,
    None,
}

pub fn poll_game_over_input() -> GameOverInput {
    if !event::poll(Duration::from_millis(100)).unwrap_or(false) {
        return GameOverInput::None;
    }

    match event::read() {
        Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) => {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                return GameOverInput::Quit;
            }
            match code {
                KeyCode::Char('r') | KeyCode::Char('R') => GameOverInput::Restart,
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => GameOverInput::Quit,
                _ => GameOverInput::None,
            }
        }
        _ => GameOverInput::None,
    }
}
