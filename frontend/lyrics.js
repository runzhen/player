const { invoke } = window.__TAURI__.core;

// Titlebar drag to move window
const titlebar = document.getElementById("titlebar");
titlebar.addEventListener("mousedown", (e) => {
  if (e.button === 0) {
    invoke("cmd_start_drag");
  }
});

const container = document.getElementById("lyrics-container");
let lastLyricCount = -1;
let lastActiveIndex = -1;

async function pollLyrics() {
  try {
    const state = await invoke("cmd_get_state");
    const lyrics = state.lyrics || [];
    const activeIdx = state.current_lyric_index;

    // Rebuild DOM if lyrics changed
    if (lyrics.length !== lastLyricCount) {
      lastLyricCount = lyrics.length;
      lastActiveIndex = -1;
      container.innerHTML = "";

      if (lyrics.length === 0) {
        container.innerHTML = '<div class="no-lyrics">No lyrics available</div>';
        return;
      }

      lyrics.forEach((text, i) => {
        const div = document.createElement("div");
        div.className = "lyric-line";
        div.textContent = text || "♪";
        div.dataset.index = i;
        container.appendChild(div);
      });
    }

    // Update highlight
    if (activeIdx !== lastActiveIndex && lyrics.length > 0) {
      lastActiveIndex = activeIdx;
      const lines = container.querySelectorAll(".lyric-line");
      lines.forEach((el, i) => {
        el.classList.toggle("active", i === activeIdx);
      });

      // Auto-scroll to active line
      if (activeIdx != null && lines[activeIdx]) {
        lines[activeIdx].scrollIntoView({ behavior: "smooth", block: "center" });
      }
    }
  } catch (e) {
    // ignore
  }
}

setInterval(pollLyrics, 200);
pollLyrics();
