import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { onMounted, onUnmounted } from "vue";

import { useShow } from "@/stores/show/show";
import { useShowChanges } from "@/stores/show/show-changes";

export const SHORTCUT_UNDO = "CommandOrControl+Z";
export const SHORTCUT_REDO_SHIFT = "CommandOrControl+Shift+Z";
export const SHORTCUT_REDO_Y = "CommandOrControl+Y";
export const SHORTCUT_SAVE = "CommandOrControl+S";

export const SHOW_SHORTCUTS = [SHORTCUT_UNDO, SHORTCUT_REDO_SHIFT, SHORTCUT_REDO_Y, SHORTCUT_SAVE];

export function useGlobalShortcuts() {
  const show = useShow();
  const changes = useShowChanges();

  async function registerShortcuts() {
    try {
      await register(SHOW_SHORTCUTS, (event) => {
        if (event.state !== "Pressed" || !show.isLoaded) return;

        if (event.shortcut === SHORTCUT_UNDO) {
          changes.undo();
        } else if (event.shortcut === SHORTCUT_REDO_SHIFT || event.shortcut === SHORTCUT_REDO_Y) {
          changes.redo();
        } else if (event.shortcut === SHORTCUT_SAVE) {
          if (show.canSave) {
            changes.save();
          }
        }
      });
    } catch {
      // Ignored in non-Tauri / test environments
    }
  }

  async function unregisterShortcuts() {
    try {
      await unregisterAll();
    } catch {
      // Ignored in non-Tauri / test environments
    }
  }

  onMounted(() => {
    registerShortcuts();
  });

  onUnmounted(() => {
    unregisterShortcuts();
  });

  return {
    registerShortcuts,
    unregisterShortcuts,
  };
}
