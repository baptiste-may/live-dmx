import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { mockFixture1, mockFixture2 } from "@/stores/show/__mocks__/show-patch";
import { useShowPatch } from "@/stores/show/show-patch";

import { usePatchGroups } from "./patch-groups";

describe("patch-groups store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("should open modal and populate fixture name", () => {
    const patchGroups = usePatchGroups();
    const showPatch = useShowPatch();
    showPatch.fixtures = [mockFixture1, mockFixture2];

    patchGroups.openModal([1, 2]);

    expect(patchGroups.table.selectedFixtures).toEqual([1, 2]);
    expect(patchGroups.table.modal.nameInput).toBe("Spot 1");
    expect(patchGroups.table.modal.isOpen).toBe(true);
  });

  it("should update selected fixtures name via invoke", async () => {
    const patchGroups = usePatchGroups();
    patchGroups.table.selectedFixtures = [1, 2];
    patchGroups.table.modal.nameInput = "New Spot";

    await patchGroups.updateSelectedFixturesName();

    expect(invoke).toHaveBeenCalledWith("show_patch_update_name", {
      ids: [1, 2],
      name: "New Spot",
    });
  });

  it("should update selected fixtures address via invoke", async () => {
    const patchGroups = usePatchGroups();
    patchGroups.table.selectedFixtures = [1, 2];
    patchGroups.table.modal.startAddress = 55;

    await patchGroups.updateSelectedFixturesAddress();

    expect(invoke).toHaveBeenCalledWith("show_patch_update_address", {
      ids: [1, 2],
      address: 55,
    });
  });

  it("should update selected fixtures universe via invoke", async () => {
    const patchGroups = usePatchGroups();
    patchGroups.table.selectedFixtures = [1, 2];
    patchGroups.table.modal.startUniverse = 3;

    await patchGroups.updateSelectedFixturesUniverse();

    expect(invoke).toHaveBeenCalledWith("show_patch_update_universe", {
      ids: [1, 2],
      universe: 3,
    });
  });

  it("should toggle fixture group via invoke", async () => {
    const patchGroups = usePatchGroups();
    patchGroups.table.selectedFixtures = [1, 2];

    await patchGroups.toggleFixtureGroup(2);

    expect(invoke).toHaveBeenCalledWith("show_patch_toggle_group", {
      ids: [1, 2],
      groupId: 2,
    });
  });

  it("should correctly calculate isGroupSelected", () => {
    const patchGroups = usePatchGroups();
    const showPatch = useShowPatch();
    showPatch.fixtures = [mockFixture1, mockFixture2];

    patchGroups.table.selectedFixtures = [1, 2];
    expect(patchGroups.isGroupSelected(1)).toBe(true);
    expect(patchGroups.isGroupSelected(2)).toBe(false);

    patchGroups.table.selectedFixtures = [1];
    expect(patchGroups.isGroupSelected(2)).toBe(true);
  });

  it("should select group and clear selected fixtures", () => {
    const patchGroups = usePatchGroups();
    patchGroups.table.selectedFixtures = [1, 2];
    patchGroups.selectedGroup = 0;

    patchGroups.select(2);

    expect(patchGroups.selectedGroup).toBe(2);
    expect(patchGroups.table.selectedFixtures).toEqual([]);
    expect(invoke).not.toHaveBeenCalled();
  });
});
