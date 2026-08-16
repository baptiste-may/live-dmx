import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { defineStore } from "pinia";

import { useFixtureBrowser } from "./fixture-browser.ts";
import { useNavigation } from "./navigation.ts";
import { usePatchForm } from "./patch/patch-form.ts";
import { useRecentShows } from "./recent-shows.ts";
import { ShowStatus, useShow } from "./show/show.ts";
import { useShowChanges } from "./show/show-changes.ts";
import { ToastType, useToastStore } from "./toast.ts";

export const useApp = defineStore("app", {
  state: () => ({
    show: useShow(),
    navigation: useNavigation(),
    fixtureBrowser: useFixtureBrowser(),
    patchForm: usePatchForm(),
  }),
  getters: {
    isShowLoaded: (state) => state.show.isLoaded,
  },
  actions: {
    async createShow() {
      const changes = useShowChanges();
      return changes.confirmWithSaveGuard(async () => {
        try {
          await invoke("show_create");
          this.show.file.path = undefined;
          this.show.file.isDirty = false;
          this.show.state = ShowStatus.LOADED;
        } catch (error: unknown) {
          this.show.state = ShowStatus.NOT_LOADED;
          useToastStore().addToast(`Failed to create show: ${String(error)}`, ToastType.ERROR);
        }
      });
    },
    async askLoadShow() {
      const changes = useShowChanges();
      return changes.confirmWithSaveGuard(async () => {
        try {
          const selected = await openDialog({
            multiple: false,
            directory: false,
            filters: [
              {
                name: "Live DMX Show",
                extensions: ["ldmx", "json"],
              },
            ],
          });
          if (selected && typeof selected === "string") {
            await this.loadShow(selected);
          }
        } catch {
          // Ignored in non-Tauri / test environments
        }
      });
    },
    async loadShow(path: string) {
      const changes = useShowChanges();
      return changes.confirmWithSaveGuard(async () => {
        const previousState = this.show.state;
        this.show.state = ShowStatus.LOADING;
        try {
          await invoke("show_load", { path });
          this.show.file.path = path;
          this.show.file.isDirty = false;
          this.show.state = ShowStatus.LOADED;

          const recentShows = useRecentShows();
          recentShows.addOrUpdate(this.show.identity.displayName, path);
        } catch (error: unknown) {
          this.show.state = previousState;
          useToastStore().addToast(`Failed to load show: ${String(error)}`, ToastType.ERROR);
        }
      });
    },
    closeShow() {
      const changes = useShowChanges();
      return changes.confirmWithSaveGuard(() => {
        this.show.unload();
        this.navigation.$reset();
        this.patchForm.$reset();
        this.fixtureBrowser.clearSearch();
      });
    },
  },
});
