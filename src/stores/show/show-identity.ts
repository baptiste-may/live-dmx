import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

import { ToastType, useToastStore } from "../toast.ts";
import { useShowChanges } from "./show-changes.ts";

export type ShowIdentityState = {
  displayName: string;
  nameForm: {
    isEditing: boolean;
    nameInput: string;
  };
};

export const defaultShowIdentityState: ShowIdentityState = {
  displayName: "My show",
  nameForm: {
    isEditing: false,
    nameInput: "",
  },
};

export const useShowIdentity = defineStore("showIdentity", {
  state: (): ShowIdentityState => ({
    ...defaultShowIdentityState,
    nameForm: { ...defaultShowIdentityState.nameForm },
  }),
  actions: {
    async updateDisplayName(newName: string) {
      try {
        await invoke("show_identity_update", { displayName: newName });
        this.displayName = newName;
        const changes = useShowChanges();
        changes.isDirty = true;
      } catch (error: unknown) {
        useToastStore().addToast(`Failed to update show name: ${String(error)}`, ToastType.ERROR);
      }
    },
  },
});
