import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";

import type { UniverseOutputConfig } from "@/types/output.ts";

import { useLiveButtons } from "../live/live-buttons.ts";
import { useOutputUniverses } from "../output/output-universes.ts";
import { usePatchForm } from "../patch/patch-form.ts";
import { usePatchGroups } from "../patch/patch-groups.ts";
import { ToastType, useToastStore } from "../toast.ts";
import { defaultShowChangesState, useShowChanges } from "./show-changes.ts";
import { defaultShowGroupsState, type FixtureGroup, useShowGroups } from "./show-groups.ts";
import { defaultShowIdentityState, useShowIdentity } from "./show-identity.ts";
import { defaultShowPatchState, useShowPatch } from "./show-patch.ts";

export type ShowData = {
  identity: ReturnType<typeof useShowIdentity>["$state"];
  file: ReturnType<typeof useShowChanges>["$state"];
  patch: ReturnType<typeof useShowPatch>["$state"];
  groups: ReturnType<typeof useShowGroups>["$state"];
  output?: {
    universes: UniverseOutputConfig[];
  };
};

export const defaultShow: ShowData = {
  identity: defaultShowIdentityState,
  file: defaultShowChangesState,
  patch: defaultShowPatchState,
  groups: defaultShowGroupsState,
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
    groups: useShowGroups(),
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
              let shouldResetLiveState = false;

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
                  if (JSON.stringify(this.patch.fixtures) !== JSON.stringify(patch.fixtures)) {
                    shouldResetLiveState = true;
                  }
                  this.patch.fixtures = patch.fixtures;
                  const patchForm = usePatchForm();
                  if (!patchForm.isFixtureSelected()) {
                    patchForm.setDefaultLocation();
                  }
                }
              }
              if (event.payload.groups) {
                const showGroups = useShowGroups();
                const groupsPayload = event.payload.groups as {
                  groups?: FixtureGroup[];
                };
                if (groupsPayload.groups) {
                  if (JSON.stringify(showGroups.groups) !== JSON.stringify(groupsPayload.groups)) {
                    shouldResetLiveState = true;
                  }
                  showGroups.groups = groupsPayload.groups;
                  const patchGroups = usePatchGroups();
                  if (!showGroups.groups.some((g) => g.id === patchGroups.selectedGroup)) {
                    patchGroups.selectedGroup = 0;
                  }
                }
              }
              if (event.payload.output) {
                const outputPayload = event.payload.output as {
                  universes?: UniverseOutputConfig[];
                };
                if (outputPayload.universes) {
                  useOutputUniverses().universes = outputPayload.universes;
                }
              }
              if (
                typeof event.payload.status === "string" &&
                Object.values(ShowStatus).includes(event.payload.status as ShowStatus)
              ) {
                if (this.state !== event.payload.status) {
                  shouldResetLiveState = true;
                }
                this.state = event.payload.status as ShowStatus;
              }

              if (shouldResetLiveState) {
                useLiveButtons().$reset();
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
      const patchForm = usePatchForm();
      if (!patchForm.isFixtureSelected()) {
        patchForm.setDefaultLocation();
      }
      if (data.groups?.groups) {
        this.groups.groups = data.groups.groups;
      }
      if (data.output?.universes) {
        useOutputUniverses().universes = data.output.universes;
      } else {
        useOutputUniverses().$reset();
      }
      this.state = ShowStatus.LOADED;
      useLiveButtons().$reset();
      invoke("show_create").catch((error: unknown) => {
        useToastStore().addToast(`Failed to create show: ${String(error)}`, ToastType.ERROR);
      });
    },
    unload() {
      this.identity.$reset();
      this.file.$reset();
      this.patch.$reset();
      this.groups.$reset();
      usePatchGroups().$reset();
      useOutputUniverses().$reset();
      useLiveButtons().$reset();
      this.state = ShowStatus.NOT_LOADED;
      invoke("show_unload").catch((error: unknown) => {
        useToastStore().addToast(`Failed to unload show: ${String(error)}`, ToastType.ERROR);
      });
    },
  },
});
