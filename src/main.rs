mod config;
mod game_map;
mod input;
mod snake;

use std::io::{self, Write};
use std::time::Duration;

use clap::Parser;
use crossterm::{
    cursor,
    style::{Color, Stylize},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use rand::rngs::StdRng;
use rand::SeedableRng;

use config::Settings;
use game_map::GameMap;
use input::{poll_game_over_input, poll_input, GameInput, GameOverInput};
use snake::Snake;

fn bell(stdout: &mut io::Stdout) {
    let _ = write!(stdout, "\x07");
    let _ = stdout.flush();
}

fn main() {
    let settings = Settings::parse().resolve();

    let mut stdout = io::stdout();
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    stdout
        .execute(terminal::EnterAlternateScreen)
        .expect("Failed to enter alternate screen");
    stdout
        .execute(cursor::Hide)
        .expect("Failed to hide cursor");

    let result = run_game(&settings, &mut stdout);

    let _ = stdout.execute(cursor::Show);
    let _ = stdout.execute(terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run_game(settings: &Settings, stdout: &mut io::Stdout) -> io::Result<()> {
    let w = settings.map_width;
    let h = settings.map_height;
    let mut snake = Snake::new(w, h);
    let mut game_map = GameMap::new(w, h);
    let frame_duration = Duration::from_millis(settings.speed);

    let mut rng: StdRng = if settings.seed != 0 {
        StdRng::seed_from_u64(settings.seed)
    } else {
        StdRng::from_entropy()
    };

    game_map.place_food(&mut snake, &mut rng);
    if settings.obstacles > 0 {
        game_map.place_walls(settings.obstacles, &snake, &mut rng);
    }

    let mut paused = false;

    loop {
        while !snake.is_dead {
            let input = poll_input(settings, Duration::from_millis(1));
            match &input {
                GameInput::Move(dir) => snake.queue_direction(*dir),
                GameInput::Pause => {
                    paused = !paused;
                    let _ = poll_input(settings, Duration::from_millis(1));
                }
                GameInput::Quit => return Ok(()),
                GameInput::None => {}
            }

            if paused {
                stdout.execute(cursor::MoveTo(0, 0))?;
                stdout.execute(terminal::Clear(ClearType::All))?;
                let frame = game_map.render(&snake, settings);
                write!(stdout, "{frame}")?;
                write!(stdout, "  {}\r\n",
                    "** PAUSED — press P or Space to resume **".with(Color::Yellow))?;
                stdout.flush()?;
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }

            snake.apply_queued_input();
            let walls = game_map.walls.clone();
            snake.update_movement(settings, &walls);

            if snake.is_dead {
                bell(stdout);
                break;
            }

            if snake.food_eaten {
                bell(stdout);
                game_map.place_food(&mut snake, &mut rng);
            }

            stdout.execute(cursor::MoveTo(0, 0))?;
            stdout.execute(terminal::Clear(ClearType::All))?;
            let frame = game_map.render(&snake, settings);
            write!(stdout, "{frame}")?;
            stdout.flush()?;

            let mut remaining = frame_duration;
            let poll_interval = Duration::from_millis(10);
            while remaining > Duration::ZERO {
                let wait = remaining.min(poll_interval);
                match poll_input(settings, wait) {
                    GameInput::Move(dir) => snake.queue_direction(dir),
                    GameInput::Pause => paused = !paused,
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
            write!(stdout, "\r\n  {}\r\n", "GAME OVER! Restarting...".with(Color::Red))?;
            stdout.flush()?;
            std::thread::sleep(Duration::from_secs(1));
            snake.reset();
            game_map.place_food(&mut snake, &mut rng);
            continue;
        }

        write!(stdout, "\r\n  {}  Score: {}\r\n",
            "GAME OVER!".with(Color::Red),
            snake.length.to_string().with(Color::Yellow))?;
        write!(stdout, "  {}\r\n",
            "Press 'r' to restart or 'q' to quit".with(Color::DarkGrey))?;
        stdout.flush()?;

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
