const { invoke } = window.__TAURI__.core;

// Titlebar drag to move window
const titlebar = document.getElementById("titlebar");
titlebar.addEventListener("mousedown", (e) => {
  if (e.button === 0) {
    invoke("cmd_start_drag");
  }
});

const trackName = document.getElementById("track-name");
const btnPlay = document.getElementById("btn-play");
const progressBar = document.getElementById("progress-bar");
const progressFill = document.getElementById("progress-fill");
const timeCurrent = document.getElementById("time-current");
const timeTotal = document.getElementById("time-total");

document.getElementById("btn-prev").addEventListener("click", () => invoke("cmd_previous"));
document.getElementById("btn-play").addEventListener("click", () => invoke("cmd_play"));
document.getElementById("btn-stop").addEventListener("click", () => invoke("cmd_stop"));
document.getElementById("btn-next").addEventListener("click", () => invoke("cmd_next"));
document.getElementById("btn-playlist").addEventListener("click", () => invoke("cmd_toggle_playlist_window"));
document.getElementById("btn-lyrics").addEventListener("click", () => invoke("cmd_toggle_lyrics_window"));

// Progress bar click to seek
progressBar.addEventListener("click", (e) => {
  const rect = progressBar.getBoundingClientRect();
  const ratio = (e.clientX - rect.left) / rect.width;
  const position = ratio * currentDuration;
  if (currentDuration > 0) {
    invoke("cmd_seek", { position });
  }
});

let currentDuration = 0;

function formatTime(secs) {
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return m + ":" + (s < 10 ? "0" : "") + s;
}

async function pollState() {
  try {
    const state = await invoke("cmd_get_state");
    // Update track name
    trackName.textContent = state.current_track || "No track";
    // Update play button
    btnPlay.textContent = state.is_playing ? "⏸" : "⏯";
    btnPlay.title = state.is_playing ? "Pause" : "Play";

    // Update progress bar
    currentDuration = state.duration_secs || 0;
    const pos = state.position_secs || 0;
    const pct = currentDuration > 0 ? (pos / currentDuration) * 100 : 0;
    progressFill.style.width = Math.min(pct, 100) + "%";
    timeCurrent.textContent = formatTime(pos);
    timeTotal.textContent = formatTime(currentDuration);
  } catch (e) {
    // ignore errors during startup
  }
}

setInterval(pollState, 250);
pollState();
