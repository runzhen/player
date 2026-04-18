use std::fs;
use std::path::Path;

/// Parse an LRC file into a sorted vec of (seconds, lyric_text).
pub fn parse_lrc(path: &Path) -> Vec<(f64, String)> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

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
                // Check if it's a timestamp (starts with digit)
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
