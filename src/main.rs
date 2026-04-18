mod icon;
mod lrc;
mod player;

use player::{AudioPlayer, PlayMode};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    path::BaseDirectory,
    tray::TrayIconBuilder,
    AppHandle, Manager, State, WebviewWindow, WindowEvent,
};

const TRAY_ID: &str = "qqplayer-tray";

enum PlayerCommand {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    LoadFile(PathBuf),
    LoadFolder(PathBuf),
    CreatePlaylist(String),
    SwitchPlaylist(usize),
    RemoveTrack(usize),
    RemoveTracks(Vec<usize>),
    Seek(f64),
    PlayIndex(usize),
    SetPlayMode(PlayMode),
}

#[derive(Serialize, Clone, Default)]
struct PlayerState {
    names: Vec<String>,
    current_index: Option<usize>,
    current_track: Option<String>,
    is_playing: bool,
    position_secs: f64,
    duration_secs: f64,
    playlist_names: Vec<String>,
    playlist_track_counts: Vec<usize>,
    active_playlist: usize,
    play_mode: String,
    lyrics: Vec<String>,
    lyric_times: Vec<f64>,
    current_lyric_index: Option<usize>,
}

struct CommandSender(mpsc::Sender<PlayerCommand>);
struct SharedState(Arc<Mutex<PlayerState>>);

// -- Tauri commands --

#[tauri::command]
fn cmd_play(sender: State<CommandSender>) {
    let _ = sender.0.send(PlayerCommand::Play);
}

#[tauri::command]
fn cmd_pause(sender: State<CommandSender>) {
    let _ = sender.0.send(PlayerCommand::Pause);
}

#[tauri::command]
fn cmd_stop(sender: State<CommandSender>) {
    let _ = sender.0.send(PlayerCommand::Stop);
}

#[tauri::command]
fn cmd_next(sender: State<CommandSender>) {
    let _ = sender.0.send(PlayerCommand::Next);
}

#[tauri::command]
fn cmd_previous(sender: State<CommandSender>) {
    let _ = sender.0.send(PlayerCommand::Previous);
}

#[tauri::command]
fn cmd_import_file(sender: State<CommandSender>) {
    let file = rfd::FileDialog::new()
        .add_filter("MP3 Files", &["mp3"])
        .pick_file();
    if let Some(path) = file {
        let _ = sender.0.send(PlayerCommand::LoadFile(path));
    }
}

#[tauri::command]
fn cmd_import_folder(sender: State<CommandSender>) {
    let folder = rfd::FileDialog::new().pick_folder();
    if let Some(path) = folder {
        let _ = sender.0.send(PlayerCommand::LoadFolder(path));
    }
}

#[tauri::command]
fn cmd_create_playlist(sender: State<CommandSender>, name: String) {
    let _ = sender.0.send(PlayerCommand::CreatePlaylist(name));
}

#[tauri::command]
fn cmd_switch_playlist(sender: State<CommandSender>, index: usize) {
    let _ = sender.0.send(PlayerCommand::SwitchPlaylist(index));
}

#[tauri::command]
fn cmd_remove_track(sender: State<CommandSender>, index: usize) {
    let _ = sender.0.send(PlayerCommand::RemoveTrack(index));
}

#[tauri::command]
fn cmd_remove_tracks(sender: State<CommandSender>, indices: Vec<usize>) {
    let _ = sender.0.send(PlayerCommand::RemoveTracks(indices));
}

#[tauri::command]
fn cmd_seek(sender: State<CommandSender>, position: f64) {
    let _ = sender.0.send(PlayerCommand::Seek(position));
}

#[tauri::command]
fn cmd_play_index(sender: State<CommandSender>, index: usize) {
    let _ = sender.0.send(PlayerCommand::PlayIndex(index));
}

#[tauri::command]
fn cmd_set_play_mode(sender: State<CommandSender>, mode: String) {
    let play_mode = match mode.as_str() {
        "repeat_one" => PlayMode::RepeatOne,
        "shuffle" => PlayMode::Shuffle,
        _ => PlayMode::Cycle,
    };
    let _ = sender.0.send(PlayerCommand::SetPlayMode(play_mode));
}

#[tauri::command]
fn cmd_start_drag(window: WebviewWindow) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn cmd_minimize_to_tray(window: WebviewWindow) {
    // Hide the window and show the tray
    let _ = window.hide();
    if let Some(tray) = window.app_handle().tray_by_id(TRAY_ID) {
        let _ = tray.set_visible(true);
    }
}

#[tauri::command]
fn cmd_quit() {
    std::process::exit(0);
}

#[tauri::command]
fn cmd_get_state(shared: State<SharedState>) -> PlayerState {
    shared.0.lock().unwrap().clone()
}

#[tauri::command]
fn cmd_toggle_lyrics_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("lyrics") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            // Keep focus on main window
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.set_focus();
            }
        }
    } else {
        // Position lyrics window right next to the main window
        let (pos_x, pos_y, outer_w) = app
            .get_webview_window("main")
            .and_then(|w| {
                let pos = w.outer_position().ok()?;
                let outer = w.outer_size().ok()?;
                let scale = w.scale_factor().ok().unwrap_or(1.0);
                Some((
                    pos.x as f64 / scale,
                    pos.y as f64 / scale,
                    outer.width as f64 / scale,
                ))
            })
            .unwrap_or((100.0, 100.0, 400.0));

        let mut builder = tauri::WebviewWindowBuilder::new(&app, "lyrics", tauri::WebviewUrl::App("lyrics.html".into()))
            .title("Lyrics")
            .inner_size(400.0, 500.0)
            .position(pos_x + outer_w, pos_y)
            .always_on_top(true)
            .focused(false);

        #[cfg(target_os = "macos")]
        {
            builder = builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true);
        }

        let _ = builder.build();
    }
}

// -- Tray menu helpers --

fn build_menu(app: &AppHandle, player: &AudioPlayer) -> tauri::Result<Menu<tauri::Wry>> {
    let track_name = player
        .get_current_track_name()
        .unwrap_or_else(|| "No track".into());
    let play_label = if player.is_playing() {
        "⏸ Pause"
    } else {
        "⏯ Play"
    };
    let playlist_info = format!("{} tracks loaded", player.playlist_len());

    let track_item = MenuItem::with_id(app, "track", &track_name, false, None::<&str>)?;
    let info_item = MenuItem::with_id(app, "info", &playlist_info, false, None::<&str>)?;
    let play_item = MenuItem::with_id(app, "play_pause", play_label, true, None::<&str>)?;
    let stop_item = MenuItem::with_id(app, "stop", "⏹ Stop", true, None::<&str>)?;
    let next_item = MenuItem::with_id(app, "next", "⏭ Next", true, None::<&str>)?;
    let prev_item = MenuItem::with_id(app, "previous", "⏮ Previous", true, None::<&str>)?;
    let import_file = MenuItem::with_id(app, "import_file", "📁 Import File…", true, None::<&str>)?;
    let import_folder =
        MenuItem::with_id(app, "import_folder", "📂 Import Folder…", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show_window", "Show Window", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;

    Menu::with_items(
        app,
        &[
            &show_item,
            &sep1,
            &track_item,
            &info_item,
            &sep2,
            &play_item,
            &stop_item,
            &prev_item,
            &next_item,
            &sep3,
            &import_file,
            &import_folder,
            &sep4,
            &quit_item,
        ],
    )
}

fn rebuild_menu(app: &AppHandle, player: &AudioPlayer) {
    if let Ok(menu) = build_menu(app, player) {
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn update_shared_state(shared: &Arc<Mutex<PlayerState>>, player: &AudioPlayer) {
    let mut state = shared.lock().unwrap();
    state.names = player.get_playlist_names();
    state.current_index = player.get_current_index();
    state.current_track = player.get_current_track_name();
    state.is_playing = player.is_playing();
    state.position_secs = player.get_position_secs();
    state.duration_secs = player.get_duration_secs();
    let info = player.get_playlists_info();
    state.playlist_names = info.iter().map(|(n, _)| n.clone()).collect();
    state.playlist_track_counts = info.iter().map(|(_, c)| *c).collect();
    state.active_playlist = player.get_active_playlist();
    state.play_mode = match player.get_play_mode() {
        PlayMode::Cycle => "cycle".into(),
        PlayMode::RepeatOne => "repeat_one".into(),
        PlayMode::Shuffle => "shuffle".into(),
    };
    let lyrics_data = player.get_current_lyrics();
    state.lyrics = lyrics_data.iter().map(|(_, text)| text.clone()).collect();
    state.lyric_times = lyrics_data.iter().map(|(time, _)| *time).collect();
    state.current_lyric_index = player.get_current_lyric_index();
}

fn main() {
    let (cmd_tx, cmd_rx) = mpsc::channel::<PlayerCommand>();
    let shared_state = Arc::new(Mutex::new(PlayerState::default()));

    let tx_tray = cmd_tx.clone();
    let shared_for_manage = shared_state.clone();

    tauri::Builder::default()
        .manage(CommandSender(cmd_tx))
        .manage(SharedState(shared_for_manage.clone()))
        .invoke_handler(tauri::generate_handler![
            cmd_play,
            cmd_pause,
            cmd_stop,
            cmd_next,
            cmd_previous,
            cmd_import_file,
            cmd_import_folder,
            cmd_create_playlist,
            cmd_switch_playlist,
            cmd_remove_track,
            cmd_remove_tracks,
            cmd_seek,
            cmd_play_index,
            cmd_set_play_mode,
            cmd_start_drag,
            cmd_minimize_to_tray,
            cmd_quit,
            cmd_get_state,
            cmd_toggle_lyrics_window,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() != "main" {
                    // Let non-main windows (like lyrics) close normally
                    return;
                }
                api.prevent_close();
                let app = window.app_handle().clone();
                let label = window.label().to_string();
                std::thread::spawn(move || {
                    let choice = rfd::MessageDialog::new()
                        .set_title("Close QQPlayer")
                        .set_description("Minimize the QQPlayer to tray?")
                        .set_buttons(rfd::MessageButtons::YesNoCancel)
                        .set_level(rfd::MessageLevel::Info)
                        .show();
                    match choice {
                        rfd::MessageDialogResult::Yes => {
                            // Minimize to tray
                            if let Some(win) = app.get_webview_window(&label) {
                                let _ = win.hide();
                            }
                            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                                let _ = tray.set_visible(true);
                            }
                        }
                        rfd::MessageDialogResult::No => {
                            // Quit
                            std::process::exit(0);
                        }
                        _ => {
                            // Cancel — do nothing
                        }
                    }
                });
            }
        })
        .setup(move |app| {
            let icon_png = icon::generate_icon_png();
            let icon = Image::from_bytes(&icon_png)?;

            let temp_player = AudioPlayer::new();
            let menu = build_menu(&app.handle().clone(), &temp_player)?;
            drop(temp_player);

            let tx = tx_tray.clone();
            let app_handle_for_tray = app.handle().clone();
            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .menu(&menu)
                .tooltip("QQPlayer")
                .on_menu_event(move |_app, event| {
                    let cmd = match event.id.as_ref() {
                        "show_window" => {
                            // Show the main window and hide tray
                            if let Some(win) = app_handle_for_tray.get_webview_window("main") {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                            if let Some(tray) = app_handle_for_tray.tray_by_id(TRAY_ID) {
                                let _ = tray.set_visible(false);
                            }
                            None
                        }
                        "play_pause" => Some(PlayerCommand::Play),
                        "stop" => Some(PlayerCommand::Stop),
                        "next" => Some(PlayerCommand::Next),
                        "previous" => Some(PlayerCommand::Previous),
                        "import_file" => {
                            let file = rfd::FileDialog::new()
                                .add_filter("MP3 Files", &["mp3"])
                                .pick_file();
                            file.map(PlayerCommand::LoadFile)
                        }
                        "import_folder" => {
                            let folder = rfd::FileDialog::new().pick_folder();
                            folder.map(PlayerCommand::LoadFolder)
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => None,
                    };
                    if let Some(c) = cmd {
                        let _ = tx.send(c);
                    }
                })
                .build(app)?;

            // Hide tray initially (window is visible)
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_visible(false);
            }

            let app_handle = app.handle().clone();
            let shared = shared_state.clone();
            let data_dir = app.path().resolve("playlists.json", BaseDirectory::AppData).ok();
            std::thread::spawn(move || {
                let mut player = AudioPlayer::new();
                if let Some(path) = data_dir {
                    player.set_save_path(path);
                    player.load_saved_state();
                }

                loop {
                    let mut changed = false;
                    while let Ok(cmd) = cmd_rx.try_recv() {
                        match cmd {
                            PlayerCommand::Play => {
                                if player.is_playing() {
                                    player.pause();
                                } else {
                                    player.play();
                                }
                            }
                            PlayerCommand::Pause => {
                                player.pause();
                            }
                            PlayerCommand::Stop => {
                                player.stop();
                            }
                            PlayerCommand::Next => {
                                player.next_track();
                            }
                            PlayerCommand::Previous => {
                                player.previous_track();
                            }
                            PlayerCommand::LoadFile(path) => {
                                player.load_file(path);
                            }
                            PlayerCommand::LoadFolder(path) => {
                                player.load_folder(path);
                            }
                            PlayerCommand::CreatePlaylist(name) => {
                                player.create_playlist(name);
                            }
                            PlayerCommand::SwitchPlaylist(index) => {
                                player.switch_playlist(index);
                            }
                            PlayerCommand::RemoveTrack(index) => {
                                player.remove_track(index);
                            }
                            PlayerCommand::RemoveTracks(indices) => {
                                player.remove_tracks(&indices);
                            }
                            PlayerCommand::Seek(position) => {
                                player.seek(position);
                            }
                            PlayerCommand::PlayIndex(index) => {
                                player.play_index(index);
                            }
                            PlayerCommand::SetPlayMode(mode) => {
                                player.set_play_mode(mode);
                            }
                        }
                        changed = true;
                    }

                    if player.check_auto_advance() {
                        changed = true;
                    }

                    if changed {
                        rebuild_menu(&app_handle, &player);
                        player.save_state();
                    }

                    // Always update shared state so position stays current
                    update_shared_state(&shared, &player);

                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
