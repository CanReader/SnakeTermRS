use std::fs;
use std::path::PathBuf;

fn highscore_path() -> PathBuf {
    if let Some(data_dir) = dirs::data_local_dir() {
        let dir = data_dir.join("snake-term");
        let _ = fs::create_dir_all(&dir);
        dir.join("highscores.txt")
    } else {
        PathBuf::from(".snake-term-highscores.txt")
    }
}

pub fn load_high_score() -> usize {
    let path = highscore_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

pub fn save_high_score(score: usize) {
    let path = highscore_path();
    let _ = fs::write(path, score.to_string());
}

pub fn update_high_score(score: usize) -> (usize, bool) {
    let current = load_high_score();
    if score > current {
        save_high_score(score);
        (score, true)
    } else {
        (current, false)
    }
}
