import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";

import { type FixtureGroup, usePatchGroups } from "../patch/patch-groups.ts";
import { ToastType, useToastStore } from "../toast.ts";
import { defaultShowChangesState, useShowChanges } from "./show-changes.ts";
import { defaultShowIdentityState, useShowIdentity } from "./show-identity.ts";
import { defaultShowPatchState, useShowPatch } from "./show-patch.ts";

export type ShowData = {
  identity: ReturnType<typeof useShowIdentity>["$state"];
  file: ReturnType<typeof useShowChanges>["$state"];
  patch: ReturnType<typeof useShowPatch>["$state"];
};

export const defaultShow: ShowData = {
  identity: defaultShowIdentityState,
  file: defaultShowChangesState,
  patch: defaultShowPatchState,
};

export enum ShowStatus {
  NOT_LOADED = "NOT_LOADED",
  LOADING = "LOADING",
  SAVING = "SAVING",
  LOADED = "LOADED",
}

let unlistenShowChanged: (() => void) | null = null;

export const useShow = defineStore("show", {
  state: () => ({
    state: ShowStatus.NOT_LOADED,
    progress: 0,
    identity: useShowIdentity(),
    file: useShowChanges(),
    patch: useShowPatch(),
  }),
  getters: {
    hasUnsavedChanges: (state) => state.state === ShowStatus.LOADED && state.file.isDirty,
    canSave: (state) =>
      state.state === ShowStatus.LOADED && (state.file.isDirty || !state.file.path),
    canUndo: (state) => state.state === ShowStatus.LOADED && state.file.canUndo,
    canRedo: (state) => state.state === ShowStatus.LOADED && state.file.canRedo,
    isLoading: (state) => state.state === ShowStatus.LOADING,
    isSaving: (state) => state.state === ShowStatus.SAVING,
    isLoaded: (state) => state.state === ShowStatus.LOADED,
  },
  actions: {
    async initEventListener() {
      if (unlistenShowChanged) {
        unlistenShowChanged();
        unlistenShowChanged = null;
      }
      try {
        unlistenShowChanged = await listen<Record<string, unknown>>(
          "show-state-changed",
          (event) => {
            if (event.payload) {
              if (event.payload.identity) {
                const identity = event.payload.identity as ShowData["identity"];
                if (identity.displayName !== undefined) {
                  this.identity.displayName = identity.displayName;
                }
              }
              if (event.payload.file) {
                const file = event.payload.file as ShowData["file"];
                if (file.path !== undefined) {
                  this.file.path = file.path;
                }
                if (file.isDirty !== undefined) {
                  this.file.isDirty = file.isDirty;
                }
                if (file.isUnsavedChangesModalOpen !== undefined) {
                  this.file.isUnsavedChangesModalOpen = file.isUnsavedChangesModalOpen;
                }
                if (file.canUndo !== undefined) {
                  this.file.canUndo = file.canUndo;
                }
                if (file.canRedo !== undefined) {
                  this.file.canRedo = file.canRedo;
                }
              }
              if (event.payload.patch) {
                const patch = event.payload.patch as ShowData["patch"];
                if (patch.fixtures) {
                  this.patch.fixtures = patch.fixtures;
                }
              }
              if (event.payload.groups) {
                const groupsStore = usePatchGroups();
                const groupsPayload = event.payload.groups as {
                  groups?: FixtureGroup[];
                  selectedGroup?: number;
                };
                if (groupsPayload.groups) {
                  groupsStore.groups = groupsPayload.groups;
                }
                if (typeof groupsPayload.selectedGroup === "number") {
                  groupsStore.selectedGroup = groupsPayload.selectedGroup;
                }
              }
              if (
                typeof event.payload.status === "string" &&
                Object.values(ShowStatus).includes(event.payload.status as ShowStatus)
              ) {
                this.state = event.payload.status as ShowStatus;
              }
            }
          },
        );
      } catch {
        // Ignored in test / non-Tauri env
      }
    },
    load(data: ShowData) {
      this.identity.displayName = data.identity.displayName;
      this.file.path = data.file.path;
      this.file.isDirty = data.file.isDirty ?? false;
      this.file.isUnsavedChangesModalOpen = data.file.isUnsavedChangesModalOpen;
      this.file.pendingAction = data.file.pendingAction ?? null;
      this.file.canUndo = data.file.canUndo ?? false;
      this.file.canRedo = data.file.canRedo ?? false;
      this.patch.fixtures = data.patch.fixtures;
      this.state = ShowStatus.LOADED;
      invoke("show_create").catch((error: unknown) => {
        useToastStore().addToast(`Failed to create show: ${String(error)}`, ToastType.ERROR);
      });
    },
    unload() {
      this.identity.$reset();
      this.file.$reset();
      this.patch.$reset();
      this.state = ShowStatus.NOT_LOADED;
      invoke("show_unload").catch((error: unknown) => {
        useToastStore().addToast(`Failed to unload show: ${String(error)}`, ToastType.ERROR);
      });
    },
  },
});
