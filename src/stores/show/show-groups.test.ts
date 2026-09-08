import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { defaultShowGroupsState, useShowGroups } from "./show-groups";
import { useShowPatch } from "./show-patch";

describe("show-groups store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("should initialize with default 'All' group", () => {
    const store = useShowGroups();

    expect(store.groups).toEqual(defaultShowGroupsState.groups);
    expect(store.groups).toHaveLength(1);
    expect(store.groups[0]).toEqual({ id: 0, name: "All" });
  });

  it("should invoke backend command when create is called", async () => {
    const store = useShowGroups();

    await store.create("Front Truss");

    expect(invoke).toHaveBeenCalledWith("show_group_create", {
      name: "Front Truss",
    });
  });

  it("should invoke backend command when rename is called", async () => {
    const store = useShowGroups();

    await store.rename(1, "Back Truss");

    expect(invoke).toHaveBeenCalledWith("show_group_rename", {
      id: 1,
      newName: "Back Truss",
    });
  });

  it("should not rename group 0", async () => {
    const store = useShowGroups();

    await store.rename(0, "New Name");

    expect(invoke).not.toHaveBeenCalledWith("show_group_rename", expect.anything());
  });

  it("should invoke backend command when remove is called", async () => {
    const store = useShowGroups();

    await store.remove(1);

    expect(invoke).toHaveBeenCalledWith("show_group_remove", {
      id: 1,
    });
  });

  it("should not remove group 0", async () => {
    const store = useShowGroups();

    await store.remove(0);

    expect(invoke).not.toHaveBeenCalledWith("show_group_remove", expect.anything());
  });

  it("should find group by id via get", () => {
    const store = useShowGroups();
    store.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Back Truss" },
    ];

    expect(store.get(1)).toEqual({ id: 1, name: "Back Truss" });
    expect(store.get(999)).toBeUndefined();
  });

  it("should return only non-zero groups with color capability in colorGroups", () => {
    const groupsStore = useShowGroups();
    const patchStore = useShowPatch();

    groupsStore.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots (Dimmer)" },
      { id: 2, name: "Wash (RGB)" },
      { id: 3, name: "Empty Group" },
    ];

    patchStore.fixtures = [
      {
        id: 1,
        groups: [1],
        name: "Spot 1",
        universe: 1,
        address: 1,
        fixtureType: { name: "Dimmer" },
        mode: { name: "1ch", channels: ["Dimmer"] },
      },
      {
        id: 2,
        groups: [2],
        name: "Wash 1",
        universe: 1,
        address: 10,
        fixtureType: { name: "RGB" },
        mode: { name: "3ch", channels: ["Red", "Green", "Blue"] },
      },
    ];

    expect(groupsStore.colorGroups).toEqual([{ id: 2, name: "Wash (RGB)" }]);
    expect(groupsStore.hasColorCapability(2)).toBe(true);
    expect(groupsStore.hasColorCapability(1)).toBe(false);
    expect(groupsStore.hasColorCapability(3)).toBe(false);
    expect(groupsStore.hasColorCapability(0)).toBe(false);
  });
});
