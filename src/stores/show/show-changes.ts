import { invoke } from "@tauri-apps/api/core";
import { save as saveDialog } from "@tauri-apps/plugin-dialog";
import { defineStore } from "pinia";

import { useRecentShows } from "../recent-shows.ts";
import { ToastType, useToastStore } from "../toast.ts";
import { useShow } from "./show.ts";
import { useShowIdentity } from "./show-identity.ts";

export type ShowChangesState = {
  path: undefined | string;
  isDirty: boolean;
  isUnsavedChangesModalOpen: boolean;
  pendingAction: (() => Promise<void> | void) | null;
  canUndo: boolean;
  canRedo: boolean;
};

export const defaultShowChangesState: ShowChangesState = {
  path: undefined,
  isDirty: false,
  isUnsavedChangesModalOpen: false,
  pendingAction: null,
  canUndo: false,
  canRedo: false,
};

export const useShowChanges = defineStore("show-changes", {
  state: (): ShowChangesState => ({ ...defaultShowChangesState }),
  actions: {
    async save(saveAs = false): Promise<boolean> {
      let targetPath = this.path;
      const identity = useShowIdentity();

      if (!targetPath || saveAs) {
        try {
          const selected = await saveDialog({
            defaultPath: `${identity.displayName || "My show"}.ldmx`,
            filters: [
              {
                name: "Live DMX Show",
                extensions: ["ldmx", "json"],
              },
            ],
          });
          if (!selected) {
            return false;
          }
          targetPath = selected;
        } catch {
          return false;
        }
      }

      try {
        await invoke("show_save", { path: targetPath });
      } catch (err) {
        const toast = useToastStore();
        toast.addToast(`Failed to save show: ${String(err)}`, ToastType.ERROR);
        return false;
      }

      this.path = targetPath;
      this.isDirty = false;

      const recentShows = useRecentShows();
      recentShows.addOrUpdate(identity.displayName, targetPath);

      const toast = useToastStore();
      toast.addToast("Show saved successfully", ToastType.SUCCESS);

      return true;
    },

    confirmWithSaveGuard(action: () => Promise<void> | void) {
      const show = useShow();
      if (!show.hasUnsavedChanges) {
        return action();
      }
      this.pendingAction = action;
      this.isUnsavedChangesModalOpen = true;
    },

    handleCancel() {
      this.isUnsavedChangesModalOpen = false;
      this.pendingAction = null;
    },

    async discardAndExecute() {
      this.isUnsavedChangesModalOpen = false;
      this.isDirty = false;
      const action = this.pendingAction;
      this.pendingAction = null;
      if (action) {
        await action();
      }
    },

    async saveAndExecute() {
      const saved = await this.save();
      if (saved) {
        this.isUnsavedChangesModalOpen = false;
        const action = this.pendingAction;
        this.pendingAction = null;
        if (action) {
          await action();
        }
      }
    },

    async undo() {
      if (!this.canUndo) return;
      await invoke("show_undo").catch(() => {
        // Ignored in test / non-Tauri env
      });
    },

    async redo() {
      if (!this.canRedo) return;
      await invoke("show_redo").catch(() => {
        // Ignored in test / non-Tauri env
      });
    },
  },
});
