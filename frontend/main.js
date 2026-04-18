const { invoke } = window.__TAURI__.core;

// Titlebar drag to move window
const titlebar = document.getElementById("titlebar");
titlebar.addEventListener("mousedown", (e) => {
  if (e.button === 0) {
    invoke("cmd_start_drag");
  }
});

const trackName = document.getElementById("track-name");
const playlist = document.getElementById("playlist");
const playlistCount = document.getElementById("playlist-count");
const btnPlay = document.getElementById("btn-play");
const tabsContainer = document.getElementById("playlist-tabs");
const contextMenu = document.getElementById("context-menu");
const ctxRemove = document.getElementById("ctx-remove");
const progressBar = document.getElementById("progress-bar");
const progressFill = document.getElementById("progress-fill");
const timeCurrent = document.getElementById("time-current");
const timeTotal = document.getElementById("time-total");
const btnMode = document.getElementById("btn-mode");

const playModes = ["cycle", "repeat_one", "shuffle"];
const modeIcons = { cycle: "🔁", repeat_one: "🔂", shuffle: "🔀" };
const modeTitles = { cycle: "Cycle", repeat_one: "Repeat One", shuffle: "Shuffle" };

btnMode.addEventListener("click", () => {
  // Cycle through modes: cycle -> repeat_one -> shuffle -> cycle
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

document.getElementById("btn-prev").addEventListener("click", () => invoke("cmd_previous"));
document.getElementById("btn-play").addEventListener("click", () => invoke("cmd_play"));
document.getElementById("btn-stop").addEventListener("click", () => invoke("cmd_stop"));
document.getElementById("btn-next").addEventListener("click", () => invoke("cmd_next"));
document.getElementById("btn-import-file").addEventListener("click", () => invoke("cmd_import_file"));
document.getElementById("btn-import-folder").addEventListener("click", () => invoke("cmd_import_folder"));
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
    // Update track name
    trackName.textContent = state.current_track || "No track";
    // Update play button
    btnPlay.textContent = state.is_playing ? "⏸" : "⏯";
    btnPlay.title = state.is_playing ? "Pause" : "Play";
    // Update play mode button
    const mode = state.play_mode || "cycle";
    btnMode.textContent = modeIcons[mode] || "🔁";
    btnMode.title = modeTitles[mode] || "Cycle";
    btnMode.dataset.mode = mode;
    // Update playlist count
    playlistCount.textContent = state.names.length + " tracks";

    // Update progress bar
    currentDuration = state.duration_secs || 0;
    const pos = state.position_secs || 0;
    const pct = currentDuration > 0 ? (pos / currentDuration) * 100 : 0;
    progressFill.style.width = Math.min(pct, 100) + "%";
    timeCurrent.textContent = formatTime(pos);
    timeTotal.textContent = formatTime(currentDuration);

    // Update playlist tabs
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
      // Add "+" button
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

    // Update playlist list
    if (playlist.children.length !== state.names.length) {
      playlist.innerHTML = "";
      state.names.forEach((name, i) => {
        const li = document.createElement("li");
        li.textContent = name;
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
