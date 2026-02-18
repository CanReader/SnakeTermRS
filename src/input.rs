use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::config::{Direction, Settings};

pub enum GameInput {
    Move(Direction),
    MoveP2(Direction),
    Pause,
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
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                return GameInput::Quit;
            }

            match code {
                KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => {
                    return GameInput::Pause;
                }
                KeyCode::Char('w') | KeyCode::Char('W') => {
                    let dir = if settings.invert_controls { Direction::South } else { Direction::North };
                    return GameInput::Move(dir);
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    let dir = if settings.invert_controls { Direction::North } else { Direction::South };
                    return GameInput::Move(dir);
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    let dir = if settings.invert_controls { Direction::East } else { Direction::West };
                    return GameInput::Move(dir);
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    let dir = if settings.invert_controls { Direction::West } else { Direction::East };
                    return GameInput::Move(dir);
                }
                KeyCode::Up => {
                    let dir = if settings.invert_controls { Direction::South } else { Direction::North };
                    return if settings.multiplayer { GameInput::MoveP2(dir) } else { GameInput::Move(dir) };
                }
                KeyCode::Down => {
                    let dir = if settings.invert_controls { Direction::North } else { Direction::South };
                    return if settings.multiplayer { GameInput::MoveP2(dir) } else { GameInput::Move(dir) };
                }
                KeyCode::Left => {
                    let dir = if settings.invert_controls { Direction::East } else { Direction::West };
                    return if settings.multiplayer { GameInput::MoveP2(dir) } else { GameInput::Move(dir) };
                }
                KeyCode::Right => {
                    let dir = if settings.invert_controls { Direction::West } else { Direction::East };
                    return if settings.multiplayer { GameInput::MoveP2(dir) } else { GameInput::Move(dir) };
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    return GameInput::Quit;
                }
                _ => return GameInput::None,
            }
        }
        _ => GameInput::None,
    }
}

pub enum MenuInput {
    Enter,
    Up,
    Down,
    Quit,
    None,
}

pub fn poll_menu_input(timeout: Duration) -> MenuInput {
    if !event::poll(timeout).unwrap_or(false) {
        return MenuInput::None;
    }

    match event::read() {
        Ok(Event::Key(KeyEvent {
            code, modifiers, ..
        })) => {
            if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                return MenuInput::Quit;
            }
            match code {
                KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => MenuInput::Up,
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => MenuInput::Down,
                KeyCode::Enter | KeyCode::Char(' ') => MenuInput::Enter,
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => MenuInput::Quit,
                _ => MenuInput::None,
            }
        }
        _ => MenuInput::None,
    }
}

pub enum GameOverInput {
    Restart,
    Quit,
    Menu,
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
                KeyCode::Char('m') | KeyCode::Char('M') => GameOverInput::Menu,
                _ => GameOverInput::None,
            }
        }
        _ => GameOverInput::None,
    }
}
