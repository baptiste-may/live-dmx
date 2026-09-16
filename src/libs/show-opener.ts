import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { onMounted, onUnmounted } from "vue";

import { useApp } from "@/stores/app";

export const OPEN_SHOW_EVENT = "open-show-request";

export function useShowOpener() {
  const app = useApp();
  let openingPath: string | null = null;
  let cleanup: (() => void) | null = null;

  async function openPath(path: string | null) {
    if (!path || typeof path !== "string") return;
    if (app.isShowLoaded && app.show.file.path === path) return;
    if (path === openingPath) return;
    openingPath = path;
    try {
      await app.loadShow(path);
    } finally {
      openingPath = null;
    }
  }

  async function start() {
    try {
      cleanup = await listen<string>(OPEN_SHOW_EVENT, (event) => {
        void openPath(event.payload);
      });
    } catch {
      // Ignored in non-Tauri / test environments
    }

    try {
      const pending = await invoke<string | null>("get_pending_open_path");
      await openPath(pending);
    } catch {
      // Ignored in non-Tauri / test environments
    }
  }

  function stop() {
    if (cleanup) {
      cleanup();
      cleanup = null;
    }
  }

  onMounted(() => {
    void start();
  });

  onUnmounted(() => {
    stop();
  });

  return {
    start,
    stop,
    openPath,
  };
}
