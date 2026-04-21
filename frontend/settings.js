const { invoke } = window.__TAURI__.core;

const lyricsDirPath = document.getElementById("lyrics-dir-path");
const lyricsScriptPath = document.getElementById("lyrics-script-path");

document.getElementById("btn-browse-lyrics-dir").addEventListener("click", () => {
  invoke("cmd_set_lyrics_dir");
});

document.getElementById("btn-clear-lyrics-dir").addEventListener("click", () => {
  invoke("cmd_clear_lyrics_dir");
});

document.getElementById("btn-browse-lyrics-script").addEventListener("click", () => {
  invoke("cmd_set_lyrics_script");
});

document.getElementById("btn-clear-lyrics-script").addEventListener("click", () => {
  invoke("cmd_clear_lyrics_script");
});

async function pollSettings() {
  try {
    const state = await invoke("cmd_get_state");
    if (state.lyrics_dir) {
      lyricsDirPath.textContent = state.lyrics_dir;
      lyricsDirPath.title = state.lyrics_dir;
      lyricsDirPath.classList.add("active");
    } else {
      lyricsDirPath.textContent = "Not set";
      lyricsDirPath.title = "";
      lyricsDirPath.classList.remove("active");
    }
    if (state.lyrics_script) {
      lyricsScriptPath.textContent = state.lyrics_script;
      lyricsScriptPath.title = state.lyrics_script;
      lyricsScriptPath.classList.add("active");
    } else {
      lyricsScriptPath.textContent = "Not set";
      lyricsScriptPath.title = "";
      lyricsScriptPath.classList.remove("active");
    }
  } catch (e) {
    // ignore
  }
}

setInterval(pollSettings, 500);
pollSettings();
