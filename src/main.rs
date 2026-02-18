mod config;
mod game_map;
mod input;
mod snake;

use std::io::{self, Write};
use std::time::Duration;

use clap::Parser;
use crossterm::{
    cursor,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use rand::rngs::StdRng;
use rand::SeedableRng;

use config::Settings;
use game_map::GameMap;
use input::{poll_game_over_input, poll_input, GameInput, GameOverInput};
use snake::Snake;

fn main() {
    let settings = Settings::parse().resolve();

    // Set up terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    stdout
        .execute(terminal::EnterAlternateScreen)
        .expect("Failed to enter alternate screen");
    stdout
        .execute(cursor::Hide)
        .expect("Failed to hide cursor");

    let result = run_game(&settings, &mut stdout);

    // Restore terminal
    let _ = stdout.execute(cursor::Show);
    let _ = stdout.execute(terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_game(settings: &Settings, stdout: &mut io::Stdout) -> io::Result<()> {
    let mut snake = Snake::new();
    let mut game_map = GameMap::new();
    let frame_duration = Duration::from_millis(settings.speed);

    let mut rng: StdRng = if settings.seed != 0 {
        StdRng::seed_from_u64(settings.seed)
    } else {
        StdRng::from_entropy()
    };

    game_map.place_food(&mut snake, &mut rng);

    loop {
        // Main game loop
        while !snake.is_dead {
            // Process input (non-blocking with short timeout)
            match poll_input(settings, Duration::from_millis(1)) {
                GameInput::Move(dir) => snake.set_next_direction(dir),
                GameInput::Quit => return Ok(()),
                GameInput::None => {}
            }

            snake.validate_direction();
            snake.update_movement(settings);

            if snake.is_dead {
                break;
            }

            if snake.food_eaten {
                game_map.place_food(&mut snake, &mut rng);
            }

            // Render
            stdout.execute(cursor::MoveTo(0, 0))?;
            stdout.execute(terminal::Clear(ClearType::All))?;
            let frame = game_map.render(&snake, settings);
            write!(stdout, "{frame}")?;
            stdout.flush()?;

            // Frame delay (also polls input during the wait)
            let mut remaining = frame_duration;
            let poll_interval = Duration::from_millis(10);
            while remaining > Duration::ZERO {
                let wait = remaining.min(poll_interval);
                match poll_input(settings, wait) {
                    GameInput::Move(dir) => snake.set_next_direction(dir),
                    GameInput::Quit => return Ok(()),
                    GameInput::None => {}
                }
                remaining = remaining.saturating_sub(wait);
            }
        }

        // Game over
        stdout.execute(cursor::MoveTo(0, 0))?;
        stdout.execute(terminal::Clear(ClearType::All))?;
        let frame = game_map.render(&snake, settings);
        write!(stdout, "{frame}")?;

        if settings.auto_restart {
            write!(stdout, "\r\n  GAME OVER! Restarting...\r\n")?;
            stdout.flush()?;
            std::thread::sleep(Duration::from_secs(1));
            snake.reset();
            game_map.place_food(&mut snake, &mut rng);
            continue;
        }

        write!(stdout, "\r\n  GAME OVER!  Score: {}\r\n", snake.length)?;
        write!(stdout, "  Press 'r' to restart or 'q' to quit\r\n")?;
        stdout.flush()?;

        // Wait for restart or quit
        loop {
            match poll_game_over_input() {
                GameOverInput::Restart => {
                    snake.reset();
                    game_map.place_food(&mut snake, &mut rng);
                    break;
                }
                GameOverInput::Quit => return Ok(()),
                GameOverInput::None => {}
            }
        }
    }
}
