import { getTauriVersion, getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { defineStore } from "pinia";

import { ToastType, useToastStore } from "./toast";

export type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "installing"
  | "up-to-date"
  | "error";

export interface UpdateInfo {
  version: string;
  body?: string;
  date?: string;
}

export interface UpdatesState {
  appVersion: string;
  tauriVersion: string;
  status: UpdateStatus;
  downloaded: number;
  contentLength: number;
  error: string | null;
  info: UpdateInfo | null;
  pendingUpdate: Update | null;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const useUpdates = defineStore("updates", {
  state: (): UpdatesState => ({
    appVersion: "",
    tauriVersion: "",
    status: "idle",
    downloaded: 0,
    contentLength: 0,
    error: null,
    info: null,
    pendingUpdate: null,
  }),
  getters: {
    progress(state): number {
      if (state.contentLength > 0) {
        return Math.min(state.downloaded / state.contentLength, 1);
      }
      return 0;
    },
    hasUpdate(state): boolean {
      return state.status === "available";
    },
    isBusy(state): boolean {
      return (
        state.status === "checking" ||
        state.status === "downloading" ||
        state.status === "installing"
      );
    },
  },
  actions: {
    async init() {
      if (this.appVersion) return;
      try {
        this.appVersion = await getVersion();
      } catch {
        this.appVersion = "0.0.0";
      }
      try {
        this.tauriVersion = await getTauriVersion();
      } catch {
        this.tauriVersion = "";
      }
    },

    async checkForUpdates() {
      const toast = useToastStore();
      this.status = "checking";
      this.error = null;
      this.info = null;
      this.pendingUpdate = null;
      try {
        const update = await check();
        if (update) {
          this.status = "available";
          this.pendingUpdate = update;
          this.info = {
            version: update.version,
            body: update.body,
            date: update.date,
          };
          toast.addToast(`Update v${update.version} is available`, ToastType.INFO);
        } else {
          this.status = "up-to-date";
          toast.addToast(`You're up to date (v${this.appVersion})`, ToastType.SUCCESS);
        }
      } catch (error) {
        this.status = "error";
        this.error = errorMessage(error);
        toast.addToast(`Update check failed: ${this.error}`, ToastType.ERROR);
      }
    },

    async downloadAndInstall() {
      const toast = useToastStore();
      if (!this.pendingUpdate) return;

      const update = this.pendingUpdate;
      this.status = "downloading";
      this.downloaded = 0;
      this.contentLength = 0;
      this.error = null;

      const onEvent = (event: DownloadEvent) => {
        switch (event.event) {
          case "Started":
            this.contentLength = event.data.contentLength ?? 0;
            break;
          case "Progress":
            this.downloaded += event.data.chunkLength;
            break;
          case "Finished":
            break;
        }
      };

      try {
        await update.downloadAndInstall(onEvent);
        this.status = "installing";
        await relaunch();
      } catch (error) {
        this.status = "error";
        this.error = errorMessage(error);
        toast.addToast(`Update failed: ${this.error}`, ToastType.ERROR);
      }
    },

    async requestUpdate() {
      if (this.hasUpdate) {
        return this.downloadAndInstall();
      }
      return this.checkForUpdates();
    },
  },
});
