mod config;
mod game_map;
mod highscore;
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
use highscore::update_high_score;
use input::*;
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

    let result = show_menu_and_play(&settings, &mut stdout);

    let _ = stdout.execute(cursor::Show);
    let _ = stdout.execute(terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

enum MenuChoice {
    Play,
    Quit,
}

fn show_menu_and_play(settings: &Settings, stdout: &mut io::Stdout) -> io::Result<()> {
    loop {
        match show_start_menu(settings, stdout)? {
            MenuChoice::Play => run_game(settings, stdout)?,
            MenuChoice::Quit => return Ok(()),
        }
    }
}

fn show_start_menu(settings: &Settings, stdout: &mut io::Stdout) -> io::Result<MenuChoice> {
    let items = ["Start Game", "Quit"];
    let mut selected = 0usize;
    let high = highscore::load_high_score();

    loop {
        stdout.execute(cursor::MoveTo(0, 0))?;
        stdout.execute(terminal::Clear(ClearType::All))?;

        let mut buf = String::new();
        buf.push_str("\r\n");
        buf.push_str(&format!("{}", "  ╔═══════════════════════════════╗\r\n".with(Color::Green)));
        buf.push_str(&format!("{}", "  ║     SNAKE — Terminal Edition  ║\r\n".with(Color::Green)));
        buf.push_str(&format!("{}", "  ╚═══════════════════════════════╝\r\n".with(Color::Green)));
        buf.push_str("\r\n");

        if high > 0 {
            buf.push_str(&format!("  {}  {}\r\n\r\n",
                "High Score:".with(Color::DarkYellow),
                high.to_string().with(Color::Yellow)));
        }

        buf.push_str(&format!("  Map: {}x{}\r\n\r\n",
            settings.map_width.to_string().with(Color::Cyan),
            settings.map_height.to_string().with(Color::Cyan)));

        for (i, item) in items.iter().enumerate() {
            if i == selected {
                buf.push_str(&format!("  {} {}\r\n", ">".with(Color::Yellow), item.with(Color::Yellow)));
            } else {
                buf.push_str(&format!("    {}\r\n", item.with(Color::White)));
            }
        }

        buf.push_str(&format!("\r\n  {}\r\n",
            "Use W/S or arrows to select, Enter to confirm".with(Color::DarkGrey)));

        write!(stdout, "{buf}")?;
        stdout.flush()?;

        match poll_menu_input(Duration::from_millis(100)) {
            MenuInput::Up => { if selected > 0 { selected -= 1; } }
            MenuInput::Down => { if selected < items.len() - 1 { selected += 1; } }
            MenuInput::Enter => {
                return Ok(match selected {
                    0 => MenuChoice::Play,
                    _ => MenuChoice::Quit,
                });
            }
            MenuInput::Quit => return Ok(MenuChoice::Quit),
            MenuInput::None => {}
        }
    }
}

fn run_game(settings: &Settings, stdout: &mut io::Stdout) -> io::Result<()> {
    let w = settings.map_width;
    let h = settings.map_height;
    let mut snake = Snake::new(w, h);
    let mut game_map = GameMap::new(w, h);

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
    let mut frame_count: usize = 0;

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
                let frame = game_map.render(&snake, settings, frame_count);
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

            game_map.maybe_spawn_bonus(&snake, &mut rng);
            game_map.tick_bonus();
            if game_map.check_bonus_eaten(&mut snake) {
                bell(stdout);
            }

            frame_count += 1;

            stdout.execute(cursor::MoveTo(0, 0))?;
            stdout.execute(terminal::Clear(ClearType::All))?;
            let frame = game_map.render(&snake, settings, frame_count);
            write!(stdout, "{frame}")?;
            stdout.flush()?;

            let effective_speed = settings.effective_speed(snake.length);
            let frame_duration = Duration::from_millis(effective_speed);
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

        // Death animation
        for i in 0..6 {
            stdout.execute(cursor::MoveTo(0, 0))?;
            stdout.execute(terminal::Clear(ClearType::All))?;
            let frame = game_map.render_death_animation(&snake, settings, i);
            write!(stdout, "{frame}")?;
            stdout.flush()?;
            std::thread::sleep(Duration::from_millis(150));
        }

        // High score
        let (high, is_new) = update_high_score(snake.score);

        // Game over
        stdout.execute(cursor::MoveTo(0, 0))?;
        stdout.execute(terminal::Clear(ClearType::All))?;
        let frame = game_map.render(&snake, settings, frame_count);
        write!(stdout, "{frame}")?;

        if settings.auto_restart {
            write!(stdout, "\r\n  {}\r\n", "GAME OVER! Restarting...".with(Color::Red))?;
            stdout.flush()?;
            std::thread::sleep(Duration::from_secs(1));
            snake.reset();
            game_map.place_food(&mut snake, &mut rng);
            game_map.bonus_food = None;
            frame_count = 0;
            continue;
        }

        write!(stdout, "\r\n  {}  Score: {}\r\n",
            "GAME OVER!".with(Color::Red),
            snake.score.to_string().with(Color::Yellow))?;
        write!(stdout, "  High Score: {}{}\r\n",
            high.to_string().with(Color::Yellow),
            if is_new { " (NEW!)" } else { "" })?;
        write!(stdout, "  {}\r\n",
            "Press 'r' to restart, 'm' for menu, or 'q' to quit".with(Color::DarkGrey))?;
        stdout.flush()?;

        loop {
            match poll_game_over_input() {
                GameOverInput::Restart => {
                    snake.reset();
                    game_map.place_food(&mut snake, &mut rng);
                    game_map.bonus_food = None;
                    frame_count = 0;
                    break;
                }
                GameOverInput::Menu => return Ok(()),
                GameOverInput::Quit => return Ok(()),
                GameOverInput::None => {}
            }
        }
    }
}
