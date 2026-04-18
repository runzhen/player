use crate::lrc;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::time::Duration;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum PlayMode {
    Cycle,
    RepeatOne,
    Shuffle,
}

impl Default for PlayMode {
    fn default() -> Self {
        PlayMode::Cycle
    }
}

#[derive(Serialize, Deserialize)]
struct SavedPlaylist {
    name: String,
    tracks: Vec<PathBuf>,
    #[serde(default)]
    play_mode: PlayMode,
}

#[derive(Serialize, Deserialize)]
struct SavedState {
    playlists: Vec<SavedPlaylist>,
    active_playlist: usize,
}

pub struct Playlist {
    pub name: String,
    pub tracks: Vec<PathBuf>,
    pub play_mode: PlayMode,
}

impl Playlist {
    pub fn new(name: String) -> Self {
        Playlist {
            name,
            tracks: Vec::new(),
            play_mode: PlayMode::Cycle,
        }
    }
}

pub struct AudioPlayer {
    playlists: Vec<Playlist>,
    active_playlist: usize,
    current_index: Option<usize>,
    current_duration: Option<Duration>,
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    is_playing: bool,
    save_path: Option<PathBuf>,
    current_lyrics: Vec<(f64, String)>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) =
            OutputStream::try_default().expect("Failed to open audio output");
        let sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink");
        sink.pause();

        AudioPlayer {
            playlists: vec![Playlist::new("Default".into())],
            active_playlist: 0,
            current_index: None,
            current_duration: None,
            sink,
            _stream: stream,
            _stream_handle: stream_handle,
            is_playing: false,
            save_path: None,
            current_lyrics: Vec::new(),
        }
    }

    pub fn set_save_path(&mut self, path: PathBuf) {
        self.save_path = Some(path);
    }

    pub fn load_saved_state(&mut self) {
        let path = match &self.save_path {
            Some(p) => p,
            None => return,
        };
        if let Ok(data) = std::fs::read_to_string(path) {
            if let Ok(saved) = serde_json::from_str::<SavedState>(&data) {
                self.playlists = saved
                    .playlists
                    .into_iter()
                    .map(|sp| Playlist {
                        name: sp.name,
                        tracks: sp.tracks,
                        play_mode: sp.play_mode,
                    })
                    .collect();
                if self.playlists.is_empty() {
                    self.playlists.push(Playlist::new("Default".into()));
                }
                self.active_playlist = saved.active_playlist.min(self.playlists.len() - 1);
            }
        }
    }

    pub fn save_state(&self) {
        let path = match &self.save_path {
            Some(p) => p,
            None => return,
        };
        let saved = SavedState {
            playlists: self
                .playlists
                .iter()
                .map(|p| SavedPlaylist {
                    name: p.name.clone(),
                    tracks: p.tracks.clone(),
                    play_mode: p.play_mode,
                })
                .collect(),
            active_playlist: self.active_playlist,
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&saved) {
            let _ = std::fs::write(path, json);
        }
    }

    fn tracks(&self) -> &Vec<PathBuf> {
        &self.playlists[self.active_playlist].tracks
    }

    fn tracks_mut(&mut self) -> &mut Vec<PathBuf> {
        &mut self.playlists[self.active_playlist].tracks
    }

    pub fn create_playlist(&mut self, name: String) {
        self.playlists.push(Playlist::new(name));
    }

    pub fn switch_playlist(&mut self, index: usize) {
        if index < self.playlists.len() && index != self.active_playlist {
            self.stop();
            self.active_playlist = index;
            self.current_index = None;
        }
    }

    pub fn remove_track(&mut self, index: usize) {
        let tracks = self.tracks_mut();
        if index >= tracks.len() {
            return;
        }
        tracks.remove(index);
        // Adjust current_index
        match self.current_index {
            Some(ci) if ci == index => {
                // Currently playing track was removed — stop
                self.stop();
                self.current_index = if self.tracks().is_empty() {
                    None
                } else {
                    Some(ci.min(self.tracks().len() - 1))
                };
            }
            Some(ci) if ci > index => {
                self.current_index = Some(ci - 1);
            }
            _ => {}
        }
    }

    pub fn remove_tracks(&mut self, indices: &[usize]) {
        let mut sorted: Vec<usize> = indices.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        for &idx in sorted.iter().rev() {
            self.remove_track(idx);
        }
    }

    pub fn get_active_playlist(&self) -> usize {
        self.active_playlist
    }

    pub fn get_playlists_info(&self) -> Vec<(String, usize)> {
        self.playlists
            .iter()
            .map(|p| (p.name.clone(), p.tracks.len()))
            .collect()
    }

    pub fn get_play_mode(&self) -> PlayMode {
        self.playlists[self.active_playlist].play_mode
    }

    pub fn set_play_mode(&mut self, mode: PlayMode) {
        self.playlists[self.active_playlist].play_mode = mode;
    }

    pub fn load_file(&mut self, path: PathBuf) {
        if path
            .extension()
            .map_or(false, |e| e.eq_ignore_ascii_case("mp3"))
            && !self.tracks().contains(&path)
        {
            self.tracks_mut().push(path);
        }
    }

    pub fn load_folder(&mut self, folder: PathBuf) {
        for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
            let path = entry.into_path();
            if path.is_file()
                && path
                    .extension()
                    .map_or(false, |e| e.eq_ignore_ascii_case("mp3"))
                && !self.tracks().contains(&path)
            {
                self.tracks_mut().push(path);
            }
        }
    }

    pub fn play(&mut self) {
        if self.tracks().is_empty() {
            return;
        }

        if self.current_index.is_some() && !self.sink.empty() {
            self.sink.play();
            self.is_playing = true;
            return;
        }

        let idx = self.current_index.unwrap_or(0);
        self.play_index(idx);
    }

    pub fn pause(&mut self) {
        self.sink.pause();
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.is_playing = false;
        let sink =
            Sink::try_new(&self._stream_handle).expect("Failed to create audio sink");
        self.sink = sink;
    }

    pub fn next_track(&mut self) {
        if self.tracks().is_empty() {
            return;
        }
        let idx = match self.playlists[self.active_playlist].play_mode {
            PlayMode::RepeatOne => self.current_index.unwrap_or(0),
            PlayMode::Shuffle => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                use std::time::SystemTime;
                let mut hasher = DefaultHasher::new();
                SystemTime::now().hash(&mut hasher);
                self.current_index.hash(&mut hasher);
                hasher.finish() as usize % self.tracks().len()
            }
            PlayMode::Cycle => match self.current_index {
                Some(i) => (i + 1) % self.tracks().len(),
                None => 0,
            },
        };
        self.play_index(idx);
    }

    pub fn previous_track(&mut self) {
        if self.tracks().is_empty() {
            return;
        }
        let idx = match self.playlists[self.active_playlist].play_mode {
            PlayMode::RepeatOne => self.current_index.unwrap_or(0),
            PlayMode::Shuffle => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                use std::time::SystemTime;
                let mut hasher = DefaultHasher::new();
                SystemTime::now().hash(&mut hasher);
                self.current_index.hash(&mut hasher);
                hasher.finish() as usize % self.tracks().len()
            }
            PlayMode::Cycle => {
                let len = self.tracks().len();
                match self.current_index {
                    Some(i) => (i + len - 1) % len,
                    None => 0,
                }
            }
        };
        self.play_index(idx);
    }

    pub fn check_auto_advance(&mut self) -> bool {
        if self.is_playing && self.sink.empty() && !self.tracks().is_empty() {
            self.next_track();
            return true;
        }
        false
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn get_current_track_name(&self) -> Option<String> {
        self.current_index.and_then(|i| {
            self.tracks().get(i).map(|p| {
                p.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
        })
    }

    pub fn playlist_len(&self) -> usize {
        self.tracks().len()
    }

    pub fn get_playlist_names(&self) -> Vec<String> {
        self.tracks()
            .iter()
            .map(|p| {
                p.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .collect()
    }

    pub fn get_playlist_durations(&self) -> Vec<f64> {
        self.tracks()
            .iter()
            .map(|p| {
                mp3_duration::from_path(p)
                    .map(|d| d.as_secs_f64())
                    .unwrap_or(0.0)
            })
            .collect()
    }

    pub fn get_current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn get_position_secs(&self) -> f64 {
        self.sink.get_pos().as_secs_f64()
    }

    pub fn get_duration_secs(&self) -> f64 {
        self.current_duration.map_or(0.0, |d| d.as_secs_f64())
    }

    pub fn seek(&self, position_secs: f64) {
        let _ = self.sink.try_seek(Duration::from_secs_f64(position_secs));
    }

    pub fn play_index(&mut self, idx: usize) {
        if idx >= self.tracks().len() {
            return;
        }
        self.sink.stop();
        let sink =
            Sink::try_new(&self._stream_handle).expect("Failed to create audio sink");
        self.sink = sink;

        let path = self.tracks()[idx].clone();
        // Load LRC file if it exists next to the audio file
        let lrc_path = path.with_extension("lrc");
        self.current_lyrics = lrc::parse_lrc(&lrc_path);
        // Compute duration
        self.current_duration = mp3_duration::from_path(&path).ok();
        if let Ok(file) = File::open(&path) {
            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                self.sink.append(source);
                self.sink.play();
                self.current_index = Some(idx);
                self.is_playing = true;
            }
        }
    }

    pub fn get_current_lyrics(&self) -> &[(f64, String)] {
        &self.current_lyrics
    }

    pub fn get_current_lyric_index(&self) -> Option<usize> {
        if self.current_lyrics.is_empty() || !self.is_playing {
            return None;
        }
        let pos = self.sink.get_pos().as_secs_f64();
        let mut result = None;
        for (i, (time, _)) in self.current_lyrics.iter().enumerate() {
            if *time <= pos {
                result = Some(i);
            } else {
                break;
            }
        }
        result
    }
}
