use std::fs;
use std::path::Path;
use std::process::Command;

/// Parse an LRC file into a sorted vec of (seconds, lyric_text).
pub fn parse_lrc(path: &Path) -> Vec<(f64, String)> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    parse_lrc_content(&content)
}

/// Parse LRC content string into sorted vec of (seconds, lyric_text).
pub fn parse_lrc_content(content: &str) -> Vec<(f64, String)> {
    let mut lyrics = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse lines like [mm:ss.xx]text
        let mut rest = line;
        let mut timestamps = Vec::new();

        while rest.starts_with('[') {
            let close = match rest.find(']') {
                Some(i) => i,
                None => break,
            };
            let tag = &rest[1..close];
            rest = &rest[close + 1..];

            // Skip metadata tags like [ti:...], [ar:...], [al:...], [by:...]
            if tag.contains(':') && tag.chars().next().map_or(false, |c| c.is_alphabetic()) {
                continue;
            }

            // Parse timestamp mm:ss.xx
            if let Some(secs) = parse_timestamp(tag) {
                timestamps.push(secs);
            }
        }

        let text = rest.trim().to_string();
        for ts in timestamps {
            lyrics.push((ts, text.clone()));
        }
    }

    lyrics.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    lyrics
}

fn parse_timestamp(s: &str) -> Option<f64> {
    // Format: mm:ss.xx or mm:ss
    let parts: Vec<&str> = s.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    let minutes: f64 = parts[0].parse().ok()?;
    let seconds: f64 = parts[1].parse().ok()?;
    Some(minutes * 60.0 + seconds)
}

/// Find and load lyrics for a track.
///
/// Search order:
/// 1. `<lyrics_dir>/<stem>.lrc` if lyrics_dir is set
/// 2. `<track>.lrc` (next to the audio file)
/// 3. Run lyrics_script to download, save to lyrics_dir, then parse
pub fn find_lyrics(
    track_path: &Path,
    lyrics_dir: Option<&Path>,
    lyrics_script: Option<&Path>,
) -> Vec<(f64, String)> {
    let stem = track_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // 1. Check lyrics folder
    if let Some(dir) = lyrics_dir {
        let lrc_in_dir = dir.join(format!("{}.lrc", stem));
        if lrc_in_dir.exists() {
            let result = parse_lrc(&lrc_in_dir);
            if !result.is_empty() {
                return result;
            }
        }
    }

    // 2. Check next to the audio file
    let lrc_beside = track_path.with_extension("lrc");
    if lrc_beside.exists() {
        let result = parse_lrc(&lrc_beside);
        if !result.is_empty() {
            return result;
        }
    }

    // 3. Try lyrics script
    if let Some(script) = lyrics_script {
        if script.exists() {
            let save_dir = lyrics_dir.unwrap_or_else(|| track_path.parent().unwrap_or(Path::new(".")));
            if let Some(result) = run_lyrics_script(script, track_path, &stem, save_dir) {
                return result;
            }
        }
    }

    Vec::new()
}

/// Run a lyrics script to fetch lyrics.
///
/// The script is called with:
///   <script> <track_path> <track_stem> <save_dir>
///
/// The script should save the .lrc file to <save_dir>/<track_stem>.lrc
/// and print the path to stdout. If it prints LRC content directly to stdout
/// (lines starting with '['), that is also accepted.
fn run_lyrics_script(
    script: &Path,
    track_path: &Path,
    stem: &str,
    save_dir: &Path,
) -> Option<Vec<(f64, String)>> {
    let _ = fs::create_dir_all(save_dir);

    let output = Command::new("python3")
        .arg(script)
        .arg(track_path)
        .arg(stem)
        .arg(save_dir)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Check if the script saved an LRC file
    let saved_lrc = save_dir.join(format!("{}.lrc", stem));
    if saved_lrc.exists() {
        let result = parse_lrc(&saved_lrc);
        if !result.is_empty() {
            return Some(result);
        }
    }

    // Otherwise try to parse stdout as LRC content
    let stdout = String::from_utf8_lossy(&output.stdout);
    let content = stdout.trim();
    if !content.is_empty() && content.contains('[') {
        let result = parse_lrc_content(content);
        if !result.is_empty() {
            return Some(result);
        }
    }

    None
}
