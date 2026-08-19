import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

export type FixtureGroup = {
  id: number;
  name: string;
};

export type ShowGroupsState = {
  groups: FixtureGroup[];
};

export const defaultShowGroupsState: ShowGroupsState = {
  groups: [
    {
      id: 0,
      name: "All Fixtures",
    },
  ],
};

export const useShowGroups = defineStore("showGroups", {
  state: (): ShowGroupsState => ({
    groups: [...defaultShowGroupsState.groups],
  }),
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
    get(id: number): FixtureGroup | undefined {
      return this.groups.find((g) => g.id === id);
    },
  },
});
