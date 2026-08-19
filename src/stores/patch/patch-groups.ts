import { defineStore } from "pinia";

import type { FixtureGroup } from "../show/show-groups";
import { useShowPatch } from "../show/show-patch";

export type { FixtureGroup };

export type PatchGroupsState = {
  selectedGroup: number;
  table: {
    selectedFixtures: number[];
    modal: {
      isOpen: boolean;
      nameInput: string;
      startAddress: number;
      startUniverse: number;
    };
  };
};

export const defaultPatchGroupsState: PatchGroupsState = {
  selectedGroup: 0,
  table: {
    selectedFixtures: [],
    modal: {
      isOpen: false,
      nameInput: "",
      startAddress: 1,
      startUniverse: 1,
    },
  },
};

export const usePatchGroups = defineStore("patch-groups", {
  state: (): PatchGroupsState => ({
    ...defaultPatchGroupsState,
    table: {
      ...defaultPatchGroupsState.table,
      selectedFixtures: [...defaultPatchGroupsState.table.selectedFixtures],
      modal: { ...defaultPatchGroupsState.table.modal },
    },
  }),
  actions: {
    select(id: number) {
      this.table.selectedFixtures = [];
      this.selectedGroup = id;
    },
    openModal(selectedFixturesId: number[]) {
      if (selectedFixturesId.length === 0) return;
      const showPatch = useShowPatch();

      const firstFixture = showPatch.getFixture(selectedFixturesId[0]);
      if (firstFixture === undefined) {
        return;
      }

      this.table.selectedFixtures = selectedFixturesId;
      this.table.modal.nameInput = firstFixture.name;
      this.table.modal.startAddress = firstFixture.address;
      this.table.modal.startUniverse = firstFixture.universe;
      this.table.modal.isOpen = true;
    },
    async updateSelectedFixturesName(name?: string) {
      if (this.table.selectedFixtures.length === 0) return;
      const showPatch = useShowPatch();
      const nameToUpdate = typeof name === "string" ? name : this.table.modal.nameInput;
      await showPatch.updateFixturesName(this.table.selectedFixtures, nameToUpdate);
    },
    async updateSelectedFixturesAddress(address?: number) {
      if (this.table.selectedFixtures.length === 0) return;
      const showPatch = useShowPatch();
      const addressToUpdate = typeof address === "number" ? address : this.table.modal.startAddress;
      await showPatch.updateFixturesAddress(this.table.selectedFixtures, addressToUpdate);
    },
    async updateSelectedFixturesUniverse(universe?: number) {
      if (this.table.selectedFixtures.length === 0) return;
      const showPatch = useShowPatch();
      const universeToUpdate =
        typeof universe === "number" ? universe : this.table.modal.startUniverse;
      await showPatch.updateFixturesUniverse(this.table.selectedFixtures, universeToUpdate);
    },
    async toggleFixtureGroup(groupId: number) {
      if (this.table.selectedFixtures.length === 0 || groupId === 0) return;
      const showPatch = useShowPatch();
      await showPatch.toggleGroup(this.table.selectedFixtures, groupId);
    },
    isGroupSelected(groupId: number): boolean {
      if (this.table.selectedFixtures.length === 0) return false;
      const showPatch = useShowPatch();
      const selected = showPatch.fixtures.filter((f) => this.table.selectedFixtures.includes(f.id));
      if (selected.length === 0) return false;
      return selected.every((f) => f.groups.includes(groupId));
    },
  },
});
