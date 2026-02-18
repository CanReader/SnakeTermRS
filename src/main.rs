mod config;
mod game_map;
mod highscore;
mod input;
mod replay;
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
use replay::{Player, Recorder};
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

    let result = if settings.replay.is_some() {
        run_replay(&settings, &mut stdout)
    } else {
        show_menu_and_play(&settings, &mut stdout)
    };

    let _ = stdout.execute(cursor::Show);
    let _ = stdout.execute(terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn show_menu_and_play(settings: &Settings, stdout: &mut io::Stdout) -> io::Result<()> {
    loop {
        let choice = show_start_menu(settings, stdout)?;
        match choice {
            MenuChoice::Play => {
                run_game(settings, stdout)?;
            }
            MenuChoice::Quit => return Ok(()),
        }
    }
}

enum MenuChoice {
    Play,
    Quit,
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
        buf.push_str(&format!(
            "{}",
            "  ╔═══════════════════════════════╗\r\n".with(Color::Green)
        ));
        buf.push_str(&format!(
            "{}",
            "  ║     SNAKE — Terminal Edition  ║\r\n".with(Color::Green)
        ));
        buf.push_str(&format!(
            "{}",
            "  ╚═══════════════════════════════╝\r\n".with(Color::Green)
        ));
        buf.push_str("\r\n");

        if high > 0 {
            buf.push_str(&format!(
                "  {}  {}\r\n\r\n",
                "High Score:".with(Color::DarkYellow),
                high.to_string().with(Color::Yellow)
            ));
        }

        let mode = if settings.multiplayer { "Multiplayer" } else { "Singleplayer" };
        buf.push_str(&format!("  Mode: {}\r\n", mode.with(Color::Cyan)));
        buf.push_str(&format!(
            "  Map: {}x{}\r\n\r\n",
            settings.map_width.to_string().with(Color::Cyan),
            settings.map_height.to_string().with(Color::Cyan)
        ));

        for (i, item) in items.iter().enumerate() {
            if i == selected {
                buf.push_str(&format!("  {} {}\r\n", ">".with(Color::Yellow), item.with(Color::Yellow)));
            } else {
                buf.push_str(&format!("    {}\r\n", item.with(Color::White)));
            }
        }

        buf.push_str(&format!(
            "\r\n  {}\r\n",
            "Use W/S or arrows to select, Enter to confirm".with(Color::DarkGrey)
        ));

        write!(stdout, "{buf}")?;
        stdout.flush()?;

        match poll_menu_input(Duration::from_millis(100)) {
            MenuInput::Up => {
                if selected > 0 {
                    selected -= 1;
                }
            }
            MenuInput::Down => {
                if selected < items.len() - 1 {
                    selected += 1;
                }
            }
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

    let mut snake1 = Snake::new(w, h);
    let mut snake2 = if settings.multiplayer {
        let mut s = Snake::new(w, h);
        // Place player 2 on the opposite side
        s.parts.clear();
        s.direction = config::Direction::West;
        let row = h / 2;
        let start_col = w / 2 + 2;
        for i in 0..config::INITIAL_SNAKE_LENGTH {
            let pos = (row, start_col - i);
            s.parts.push_back(pos);
        }
        s.head = *s.parts.back().unwrap();
        s.length = config::INITIAL_SNAKE_LENGTH;
        // Reset world for p2
        s.score = 0;
        Some(s)
    } else {
        None
    };

    let mut game_map = GameMap::new(w, h);
    let mut rng: StdRng = if settings.seed != 0 {
        StdRng::seed_from_u64(settings.seed)
    } else {
        StdRng::from_entropy()
    };

    game_map.place_food(&mut snake1, &mut rng);
    if settings.obstacles > 0 {
        game_map.place_walls(settings.obstacles, &snake1, &mut rng);
    }

    let mut recorder = settings.record.as_ref().map(|_| Recorder::new());
    let mut paused = false;
    let mut frame_count: usize = 0;

    loop {
        // Main game loop
        while !snake1.is_dead && snake2.as_ref().map_or(true, |s| !s.is_dead) {
            let input = poll_input(settings, Duration::from_millis(1));
            match &input {
                GameInput::Move(dir) => snake1.queue_direction(*dir),
                GameInput::MoveP2(dir) => {
                    if let Some(ref mut s2) = snake2 {
                        s2.queue_direction(*dir);
                    }
                }
                GameInput::Pause => {
                    paused = !paused;
                    // Consume lingering events
                    let _ = poll_input(settings, Duration::from_millis(1));
                }
                GameInput::Quit => {
                    if let (Some(rec), Some(path)) = (recorder.as_ref(), settings.record.as_ref()) {
                        let _ = rec.save(path);
                    }
                    return Ok(());
                }
                GameInput::None => {}
            }

            if paused {
                // Render with pause overlay
                stdout.execute(cursor::MoveTo(0, 0))?;
                stdout.execute(terminal::Clear(ClearType::All))?;
                let snakes_ref: Vec<&Snake> = if let Some(ref s2) = snake2 {
                    vec![&snake1, s2]
                } else {
                    vec![&snake1]
                };
                let frame = game_map.render(&snakes_ref, settings, true, frame_count);
                write!(stdout, "{frame}")?;
                stdout.flush()?;
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }

            // Record input
            if let Some(ref mut rec) = recorder {
                let dir_input = match &input {
                    GameInput::Move(d) => Some(*d),
                    _ => None,
                };
                rec.record_frame(dir_input);
            }

            snake1.apply_queued_input();
            if let Some(ref mut s2) = snake2 {
                s2.apply_queued_input();
            }

            let walls = game_map.walls.clone();
            let border_min = game_map.border_min;
            let border_max = game_map.border_max;

            snake1.update_movement(settings, &walls, border_min, border_max);
            if let Some(ref mut s2) = snake2 {
                s2.update_movement(settings, &walls, border_min, border_max);
                // Check P2 colliding with P1 body
                if snake1.parts.contains(&s2.head) {
                    s2.is_dead = true;
                }
                if s2.parts.contains(&snake1.head) {
                    snake1.is_dead = true;
                }
            }

            if snake1.is_dead || snake2.as_ref().map_or(false, |s| s.is_dead) {
                bell(stdout);
                break;
            }

            if snake1.food_eaten {
                bell(stdout);
                game_map.place_food(&mut snake1, &mut rng);
            }

            // Bonus food
            game_map.maybe_spawn_bonus(&snake1, &mut rng);
            game_map.tick_bonus();
            if game_map.check_bonus_eaten(&mut snake1) {
                bell(stdout);
            }

            // Shrinking border
            if settings.shrinking_border {
                game_map.update_shrinking_border(&snake1);
                // Check if snake is outside new border
                let (bmin_r, bmin_c) = game_map.border_min;
                let (bmax_r, bmax_c) = game_map.border_max;
                if snake1.head.0 < bmin_r || snake1.head.0 >= bmax_r
                    || snake1.head.1 < bmin_c || snake1.head.1 >= bmax_c
                {
                    snake1.is_dead = true;
                    bell(stdout);
                    break;
                }
            }

            frame_count += 1;

            // Render
            stdout.execute(cursor::MoveTo(0, 0))?;
            stdout.execute(terminal::Clear(ClearType::All))?;
            let snakes_ref: Vec<&Snake> = if let Some(ref s2) = snake2 {
                vec![&snake1, s2]
            } else {
                vec![&snake1]
            };
            let frame = game_map.render(&snakes_ref, settings, false, frame_count);
            write!(stdout, "{frame}")?;
            stdout.flush()?;

            // Frame delay with input polling
            let effective_speed = settings.effective_speed(snake1.length);
            let frame_duration = Duration::from_millis(effective_speed);
            let mut remaining = frame_duration;
            let poll_interval = Duration::from_millis(10);
            while remaining > Duration::ZERO {
                let wait = remaining.min(poll_interval);
                match poll_input(settings, wait) {
                    GameInput::Move(dir) => snake1.queue_direction(dir),
                    GameInput::MoveP2(dir) => {
                        if let Some(ref mut s2) = snake2 {
                            s2.queue_direction(dir);
                        }
                    }
                    GameInput::Pause => paused = !paused,
                    GameInput::Quit => {
                        if let (Some(rec), Some(path)) = (recorder.as_ref(), settings.record.as_ref()) {
                            let _ = rec.save(path);
                        }
                        return Ok(());
                    }
                    GameInput::None => {}
                }
                remaining = remaining.saturating_sub(wait);
            }
        }

        // Death animation (6 frames of flashing)
        {
            let snakes_ref: Vec<&Snake> = if let Some(ref s2) = snake2 {
                vec![&snake1, s2]
            } else {
                vec![&snake1]
            };
            for i in 0..6 {
                stdout.execute(cursor::MoveTo(0, 0))?;
                stdout.execute(terminal::Clear(ClearType::All))?;
                let frame = game_map.render_death_animation(&snakes_ref, settings, i);
                write!(stdout, "{frame}")?;
                stdout.flush()?;
                std::thread::sleep(Duration::from_millis(150));
            }
        }

        // Save recording
        if let (Some(rec), Some(path)) = (recorder.as_ref(), settings.record.as_ref()) {
            let _ = rec.save(path);
        }

        // Update high score
        let best_score = if let Some(ref s2) = snake2 {
            snake1.score.max(s2.score)
        } else {
            snake1.score
        };
        let (high, is_new) = update_high_score(best_score);

        // Game over screen
        stdout.execute(cursor::MoveTo(0, 0))?;
        stdout.execute(terminal::Clear(ClearType::All))?;
        {
            let snakes_ref: Vec<&Snake> = if let Some(ref s2) = snake2 {
                vec![&snake1, s2]
            } else {
                vec![&snake1]
            };
            let frame = game_map.render(&snakes_ref, settings, false, frame_count);
            write!(stdout, "{frame}")?;
        }

        if settings.auto_restart {
            write!(
                stdout,
                "\r\n  {}\r\n",
                "GAME OVER! Restarting...".with(Color::Red)
            )?;
            stdout.flush()?;
            std::thread::sleep(Duration::from_secs(1));
            snake1.reset();
            game_map.place_food(&mut snake1, &mut rng);
            game_map.border_min = (0, 0);
            game_map.border_max = (h, w);
            game_map.shrink_timer = 0;
            if let Some(ref mut s2) = snake2 {
                s2.reset();
            }
            frame_count = 0;
            recorder = settings.record.as_ref().map(|_| Recorder::new());
            continue;
        }

        write!(stdout, "\r\n")?;
        if snake2.is_some() {
            write!(
                stdout,
                "  {}  P1: {}  P2: {}\r\n",
                "GAME OVER!".with(Color::Red),
                snake1.score.to_string().with(Color::Green),
                snake2.as_ref().unwrap().score.to_string().with(Color::Cyan),
            )?;
        } else {
            write!(
                stdout,
                "  {}  Score: {}\r\n",
                "GAME OVER!".with(Color::Red),
                snake1.score.to_string().with(Color::Yellow),
            )?;
        }

        write!(
            stdout,
            "  High Score: {}{}\r\n",
            high.to_string().with(Color::Yellow),
            if is_new { " (NEW!)" } else { "" }
        )?;
        write!(
            stdout,
            "  {}\r\n",
            "Press 'r' to restart, 'm' for menu, or 'q' to quit".with(Color::DarkGrey)
        )?;
        stdout.flush()?;

        loop {
            match poll_game_over_input() {
                GameOverInput::Restart => {
                    snake1.reset();
                    game_map.place_food(&mut snake1, &mut rng);
                    game_map.border_min = (0, 0);
                    game_map.border_max = (h, w);
                    game_map.shrink_timer = 0;
                    game_map.bonus_food = None;
                    if let Some(ref mut s2) = snake2 {
                        s2.reset();
                    }
                    frame_count = 0;
                    recorder = settings.record.as_ref().map(|_| Recorder::new());
                    break;
                }
                GameOverInput::Menu => return Ok(()),
                GameOverInput::Quit => return Ok(()),
                GameOverInput::None => {}
            }
        }
    }
}

fn run_replay(settings: &Settings, stdout: &mut io::Stdout) -> io::Result<()> {
    let path = settings.replay.as_ref().unwrap();
    let mut player = match Player::load(path) {
        Ok(p) => p,
        Err(e) => {
            let _ = stdout.execute(cursor::Show);
            let _ = stdout.execute(terminal::LeaveAlternateScreen);
            let _ = terminal::disable_raw_mode();
            eprintln!("Failed to load replay: {e}");
            std::process::exit(1);
        }
    };

    let w = settings.map_width;
    let h = settings.map_height;
    let mut snake = Snake::new(w, h);
    let mut game_map = GameMap::new(w, h);
    let mut rng: StdRng = if settings.seed != 0 {
        StdRng::seed_from_u64(settings.seed)
    } else {
        StdRng::seed_from_u64(42) // replays need deterministic food
    };

    game_map.place_food(&mut snake, &mut rng);
    if settings.obstacles > 0 {
        game_map.place_walls(settings.obstacles, &snake, &mut rng);
    }

    let mut frame_count: usize = 0;

    while !snake.is_dead {
        // Check for quit
        match poll_input(settings, Duration::from_millis(1)) {
            GameInput::Quit => return Ok(()),
            _ => {}
        }

        match player.next_frame() {
            Some(Some(dir)) => snake.queue_direction(dir),
            Some(None) => {}
            None => break, // replay ended
        }

        snake.apply_queued_input();
        let walls = game_map.walls.clone();
        snake.update_movement(settings, &walls, game_map.border_min, game_map.border_max);

        if snake.is_dead {
            break;
        }

        if snake.food_eaten {
            game_map.place_food(&mut snake, &mut rng);
        }

        frame_count += 1;

        stdout.execute(cursor::MoveTo(0, 0))?;
        stdout.execute(terminal::Clear(ClearType::All))?;
        let frame = game_map.render(&[&snake], settings, false, frame_count);
        write!(stdout, "{frame}")?;
        write!(
            stdout,
            "  {}\r\n",
            "REPLAY — press Q to exit".with(Color::DarkGrey)
        )?;
        stdout.flush()?;

        std::thread::sleep(Duration::from_millis(settings.speed));
    }

    write!(
        stdout,
        "\r\n  {}  Final Score: {}\r\n",
        "Replay finished.".with(Color::Yellow),
        snake.score
    )?;
    write!(
        stdout,
        "  {}\r\n",
        "Press any key to exit".with(Color::DarkGrey)
    )?;
    stdout.flush()?;

    loop {
        match poll_input(settings, Duration::from_millis(100)) {
            GameInput::None => {}
            _ => return Ok(()),
        }
    }
}
