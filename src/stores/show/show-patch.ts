import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

import type { PatchBatchRequest, PatchedFixture, PatchRequest } from "@/types/patch";

export type ShowPatchState = {
  fixtures: PatchedFixture[];
};

export const defaultShowPatchState: ShowPatchState = {
  fixtures: [],
};

export const useShowPatch = defineStore("showPatch", {
  state: (): ShowPatchState => ({ fixtures: [...defaultShowPatchState.fixtures] }),
  actions: {
    async patchFixture(request: PatchRequest, groupId: number) {
      await invoke("show_patch_fixture", { request, groupId });
    },
    async patchBatch(request: PatchBatchRequest) {
      await invoke("show_patch_batch", { request });
    },
    async removeFixtures(ids: number[]) {
      await invoke("show_patch_remove_fixtures", { ids });
    },
    async updateFixturesName(ids: number[], name: string) {
      await invoke("show_patch_update_name", { ids, name });
    },
    async updateFixturesAddress(ids: number[], address: number) {
      await invoke("show_patch_update_address", { ids, address });
    },
    async updateFixturesUniverse(ids: number[], universe: number) {
      await invoke("show_patch_update_universe", { ids, universe });
    },
    async toggleGroup(ids: number[], groupId: number) {
      await invoke("show_patch_toggle_group", { ids, groupId });
    },
    getFixture(fixtureId: number): PatchedFixture | undefined {
      return this.fixtures.find((f) => f.id === fixtureId);
    },
  },
});
