import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

import { groupHasColorCapability } from "@/libs/fixture.ts";
import { useShowPatch } from "@/stores/show/show-patch.ts";
import type { GroupSettings } from "@/views/configure/types.ts";

export type FixtureGroup = {
  id: number;
  name: string;
};

export type ShowGroupsState = {
  groups: FixtureGroup[];
  /** Per-group settings keyed by group id. Only entries the user has edited are stored. */
  settings: Record<number, GroupSettings>;
};

export const defaultShowGroupsState: ShowGroupsState = {
  groups: [
    {
      id: 0,
      name: "All",
    },
  ],
  settings: {},
};

export const useShowGroups = defineStore("showGroups", {
  state: (): ShowGroupsState => ({
    groups: [...defaultShowGroupsState.groups],
    settings: {},
  }),
  getters: {
    colorGroups: (state): FixtureGroup[] => {
      const showPatch = useShowPatch();
      return state.groups.filter(
        (g) => g.id !== 0 && groupHasColorCapability(g.id, showPatch.fixtures),
      );
    },
    hasColorCapability:
      () =>
      (groupId: number): boolean => {
        const showPatch = useShowPatch();
        return groupHasColorCapability(groupId, showPatch.fixtures);
      },
  },
  actions: {
    async create(name: string): Promise<number> {
      return await invoke<number>("show_group_create", { name });
    },
    async remove(id: number): Promise<void> {
      if (id === 0) return;
      await invoke("show_group_remove", { id });
    },
    async rename(id: number, newName: string): Promise<void> {
      if (id === 0) return;
      await invoke("show_group_rename", { id, newName });
    },
    async updateSettings(groupId: number, settings: GroupSettings): Promise<void> {
      await invoke("show_group_update_settings", { groupId, settings });
      this.settings[groupId] = settings;
    },
    get(id: number): FixtureGroup | undefined {
      return this.groups.find((g) => g.id === id);
    },
  },
});
