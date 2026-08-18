import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

import type { RecentShow } from "@/types/shows.ts";

import { ToastType, useToastStore } from "./toast.ts";

export const useRecentShows = defineStore("recentShows", {
  state: () => ({
    shows: [] as RecentShow[],
  }),
  actions: {
    async load() {
      try {
        const loaded = await invoke<RecentShow[]>("get_recent_shows");
        if (Array.isArray(loaded)) {
          this.shows = loaded;
        }
      } catch (error: unknown) {
        useToastStore().addToast(`Failed to load recent shows: ${String(error)}`, ToastType.ERROR);
      }
    },
    async addOrUpdate(displayName: string, path: string) {
      try {
        const updated = await invoke<RecentShow[]>("save_recent_show", {
          displayName,
          path,
        });
        if (Array.isArray(updated)) {
          this.shows = updated;
        }
      } catch (error: unknown) {
        useToastStore().addToast(`Failed to save recent show: ${String(error)}`, ToastType.ERROR);
      }
    },
    async remove(path: string) {
      try {
        const updated = await invoke<RecentShow[]>("remove_recent_show", {
          path,
        });
        if (Array.isArray(updated)) {
          this.shows = updated;
        }
      } catch (error: unknown) {
        useToastStore().addToast(`Failed to remove recent show: ${String(error)}`, ToastType.ERROR);
      }
    },
  },
});
