const { invoke } = window.__TAURI__.core;

// Titlebar drag to move window
const titlebar = document.getElementById("titlebar");
titlebar.addEventListener("mousedown", (e) => {
  if (e.button === 0) {
    invoke("cmd_start_drag");
  }
});

// -- Player controls --
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

// -- Playlist panel toggle --
const COLLAPSED_HEIGHT = 220;
const EXPANDED_HEIGHT = 620;
const playlistPanel = document.getElementById("playlist-panel");
let playlistVisible = true;

document.getElementById("btn-playlist").addEventListener("click", async () => {
  playlistVisible = !playlistVisible;
  if (playlistVisible) {
    playlistPanel.classList.remove("hidden");
    await invoke("cmd_set_window_height", { height: EXPANDED_HEIGHT });
  } else {
    playlistPanel.classList.add("hidden");
    await invoke("cmd_set_window_height", { height: COLLAPSED_HEIGHT });
  }
});

// -- Playlist logic (from playlist.js) --
const playlist = document.getElementById("playlist");
const playlistCount = document.getElementById("playlist-count");
const tabsContainer = document.getElementById("playlist-tabs");
const contextMenu = document.getElementById("context-menu");
const ctxRemove = document.getElementById("ctx-remove");
const btnMode = document.getElementById("btn-mode");

const playModes = ["cycle", "repeat_one", "shuffle"];
const modeIcons = { cycle: "🔁", repeat_one: "🔂", shuffle: "🔀" };
const modeTitles = { cycle: "Cycle", repeat_one: "Repeat One", shuffle: "Shuffle" };

btnMode.addEventListener("click", () => {
  const current = btnMode.dataset.mode || "cycle";
  const nextIdx = (playModes.indexOf(current) + 1) % playModes.length;
  const next = playModes[nextIdx];
  invoke("cmd_set_play_mode", { mode: next });
});

let contextTrackIndex = null;
let selectedIndices = new Set();
let lastClickedIndex = null;

function updateSelection() {
  for (let i = 0; i < playlist.children.length; i++) {
    playlist.children[i].classList.toggle("selected", selectedIndices.has(i));
  }
}

document.getElementById("btn-import-file").addEventListener("click", () => invoke("cmd_import_file"));
document.getElementById("btn-import-folder").addEventListener("click", () => invoke("cmd_import_folder"));
document.getElementById("btn-lyrics-dir").addEventListener("click", () => invoke("cmd_set_lyrics_dir"));
document.getElementById("btn-lyrics-script").addEventListener("click", () => invoke("cmd_set_lyrics_script"));

// Right-click context menu
playlist.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  const li = e.target.closest("li");
  if (!li) return;
  const idx = parseInt(li.dataset.index, 10);
  if (!selectedIndices.has(idx)) {
    selectedIndices.clear();
    selectedIndices.add(idx);
    lastClickedIndex = idx;
    updateSelection();
  }
  contextMenu.style.left = e.clientX + "px";
  contextMenu.style.top = e.clientY + "px";
  const count = selectedIndices.size;
  ctxRemove.textContent = count > 1 ? `Remove ${count} tracks` : "Remove from playlist";
  contextMenu.classList.remove("hidden");
});

ctxRemove.addEventListener("click", () => {
  if (selectedIndices.size > 0) {
    const indices = Array.from(selectedIndices);
    invoke("cmd_remove_tracks", { indices });
    selectedIndices.clear();
    lastClickedIndex = null;
    updateSelection();
  }
  contextMenu.classList.add("hidden");
});

document.addEventListener("click", () => {
  contextMenu.classList.add("hidden");
});

let lastTabState = "";

async function pollState() {
  try {
    const state = await invoke("cmd_get_state");

    // -- Player UI --
    trackName.textContent = state.current_track || "No track";
    btnPlay.textContent = state.is_playing ? "⏸" : "⏯";
    btnPlay.title = state.is_playing ? "Pause" : "Play";

    currentDuration = state.duration_secs || 0;
    const pos = state.position_secs || 0;
    const pct = currentDuration > 0 ? (pos / currentDuration) * 100 : 0;
    progressFill.style.width = Math.min(pct, 100) + "%";
    timeCurrent.textContent = formatTime(pos);
    timeTotal.textContent = formatTime(currentDuration);

    // -- Playlist UI --
    // Update lyrics settings buttons
    const btnLyricsDir = document.getElementById("btn-lyrics-dir");
    const btnLyricsScript = document.getElementById("btn-lyrics-script");
    if (state.lyrics_dir) {
      btnLyricsDir.textContent = "📁 " + state.lyrics_dir.split("/").pop();
      btnLyricsDir.title = state.lyrics_dir;
      btnLyricsDir.classList.add("active");
    } else {
      btnLyricsDir.textContent = "📁 Lyrics Folder";
      btnLyricsDir.classList.remove("active");
    }
    if (state.lyrics_script) {
      btnLyricsScript.textContent = "🐍 " + state.lyrics_script.split("/").pop();
      btnLyricsScript.title = state.lyrics_script;
      btnLyricsScript.classList.add("active");
    } else {
      btnLyricsScript.textContent = "🐍 Lyrics Script";
      btnLyricsScript.classList.remove("active");
    }
    // Play mode button
    const mode = state.play_mode || "cycle";
    btnMode.textContent = modeIcons[mode] || "🔁";
    btnMode.title = modeTitles[mode] || "Cycle";
    btnMode.dataset.mode = mode;

    // Playlist count
    playlistCount.textContent = state.names.length + " tracks";

    // Playlist tabs
    const tabKey = state.playlist_names.join("|") + ":" + state.active_playlist;
    if (tabKey !== lastTabState) {
      lastTabState = tabKey;
      tabsContainer.innerHTML = "";
      state.playlist_names.forEach((name, i) => {
        const tab = document.createElement("button");
        tab.className = "playlist-tab" + (i === state.active_playlist ? " active" : "");
        tab.textContent = name + " (" + (state.playlist_track_counts[i] || 0) + ")";
        tab.addEventListener("click", () => invoke("cmd_switch_playlist", { index: i }));
        tabsContainer.appendChild(tab);
      });
      const addBtn = document.createElement("button");
      addBtn.className = "playlist-tab add-tab";
      addBtn.textContent = "+";
      addBtn.title = "Create new playlist";
      addBtn.addEventListener("click", () => {
        const name = prompt("New playlist name:");
        if (name && name.trim()) {
          invoke("cmd_create_playlist", { name: name.trim() });
        }
      });
      tabsContainer.appendChild(addBtn);
    }

    // Playlist track list
    if (playlist.children.length !== state.names.length) {
      playlist.innerHTML = "";
      state.names.forEach((name, i) => {
        const li = document.createElement("li");
        const nameSpan = document.createElement("span");
        nameSpan.className = "track-name";
        nameSpan.textContent = (i + 1) + "." + name;
        const durSpan = document.createElement("span");
        durSpan.className = "track-duration";
        const dur = state.durations[i] || 0;
        const m = Math.floor(dur / 60);
        const s = Math.floor(dur % 60);
        durSpan.textContent = m + ":" + (s < 10 ? "0" : "") + s;
        li.appendChild(nameSpan);
        li.appendChild(durSpan);
        li.dataset.index = i;
        li.addEventListener("click", (e) => {
          if (e.shiftKey && lastClickedIndex !== null) {
            const start = Math.min(lastClickedIndex, i);
            const end = Math.max(lastClickedIndex, i);
            if (!e.metaKey) selectedIndices.clear();
            for (let j = start; j <= end; j++) {
              selectedIndices.add(j);
            }
          } else if (e.metaKey) {
            if (selectedIndices.has(i)) {
              selectedIndices.delete(i);
            } else {
              selectedIndices.add(i);
            }
            lastClickedIndex = i;
          } else {
            selectedIndices.clear();
            selectedIndices.add(i);
            lastClickedIndex = i;
          }
          updateSelection();
        });
        li.addEventListener("dblclick", () => {
          invoke("cmd_play_index", { index: i });
        });
        playlist.appendChild(li);
      });
    }
    // Highlight current playing + selected
    for (let i = 0; i < playlist.children.length; i++) {
      playlist.children[i].classList.toggle("playing", i === state.current_index);
      playlist.children[i].classList.toggle("selected", selectedIndices.has(i));
    }
  } catch (e) {
    // ignore errors during startup
  }
}

setInterval(pollState, 250);
pollState();
