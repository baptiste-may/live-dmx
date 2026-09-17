import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { defineStore } from "pinia";

import { useFixtureBrowser } from "./fixture-browser.ts";
import { useLiveButtons } from "./live/live-buttons.ts";
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
          await useLiveButtons().resetAll();
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
        } catch (error: unknown) {
          useToastStore().addToast(
            `Failed to open show file dialog: ${String(error)}`,
            ToastType.ERROR,
          );
        }
      });
    },
    async askImportMvr() {
      const changes = useShowChanges();
      return changes.confirmWithSaveGuard(async () => {
        try {
          const selected = await openDialog({
            multiple: false,
            directory: false,
            filters: [
              {
                name: "My Virtual Rig",
                extensions: ["mvr"],
              },
            ],
          });
          if (selected && typeof selected === "string") {
            await this.importMvr(selected);
          }
        } catch (error: unknown) {
          useToastStore().addToast(
            `Failed to open MVR file dialog: ${String(error)}`,
            ToastType.ERROR,
          );
        }
      });
    },
    async importMvr(path: string) {
      const changes = useShowChanges();
      return changes.confirmWithSaveGuard(async () => {
        const previousState = this.show.state;
        this.show.state = ShowStatus.LOADING;
        try {
          await invoke("show_import_mvr", { path });
          this.show.file.path = undefined;
          this.show.file.isDirty = true;
          this.show.state = ShowStatus.LOADED;
          await useLiveButtons().resetAll();
          useToastStore().addToast("Show imported from MVR successfully", ToastType.SUCCESS);
        } catch (error: unknown) {
          this.show.state = previousState;
          useToastStore().addToast(`Failed to import MVR: ${String(error)}`, ToastType.ERROR);
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
          await useLiveButtons().resetAll();

          const recentShows = useRecentShows();
          recentShows.addOrUpdate(this.show.identity.displayName, path);
        } catch (error: unknown) {
          this.show.state = previousState;
          useToastStore().addToast(`Failed to load show: ${String(error)}`, ToastType.ERROR);
        }
      });
    },
    async closeShow() {
      const changes = useShowChanges();
      return changes.confirmWithSaveGuard(async () => {
        this.show.unload();
        this.navigation.$reset();
        this.patchForm.reset();
        this.fixtureBrowser.clearSearch();
        await useLiveButtons().resetAll();
      });
    },
  },
});
