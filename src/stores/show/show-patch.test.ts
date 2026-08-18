import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import type { PatchBatchRequest } from "@/types/patch";

import { mockFixtureType, mockMode, mockPatchNode } from "./__mocks__/show-patch";
import { useShowPatch } from "./show-patch";

describe("show-patch store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with empty fixtures array", () => {
    const store = useShowPatch();

    expect(store.fixtures).toEqual([]);
  });

  it("should invoke backend command when patchFixture is called", async () => {
    const store = useShowPatch();

    await store.patchFixture(mockPatchNode, 1);

    expect(invoke).toHaveBeenCalledWith("show_patch_fixture", {
      request: mockPatchNode,
      groupId: 1,
    });
  });

  it("should invoke backend command when patchBatch is called", async () => {
    const store = useShowPatch();
    const batchRequest: PatchBatchRequest = {
      name: "Spot 1",
      fixtureType: mockFixtureType,
      mode: mockMode,
      universe: 1,
      startAddress: 1,
      count: 4,
      channelCount: 2,
    };

    await store.patchBatch(batchRequest);

    expect(invoke).toHaveBeenCalledWith("show_patch_batch", {
      request: batchRequest,
    });
  });

  it("should invoke backend command when removeFixtures is called", async () => {
    const store = useShowPatch();

    await store.removeFixtures([1, 2]);

    expect(invoke).toHaveBeenCalledWith("show_patch_remove_fixtures", {
      ids: [1, 2],
    });
  });

  it("should invoke backend command when updateFixturesName is called", async () => {
    const store = useShowPatch();

    await store.updateFixturesName([1, 2], "New Name");

    expect(invoke).toHaveBeenCalledWith("show_patch_update_name", {
      ids: [1, 2],
      name: "New Name",
    });
  });

  it("should invoke backend command when updateFixturesAddress is called", async () => {
    const store = useShowPatch();

    await store.updateFixturesAddress([1, 2], 100);

    expect(invoke).toHaveBeenCalledWith("show_patch_update_address", {
      ids: [1, 2],
      address: 100,
    });
  });

  it("should invoke backend command when updateFixturesUniverse is called", async () => {
    const store = useShowPatch();

    await store.updateFixturesUniverse([1, 2], 2);

    expect(invoke).toHaveBeenCalledWith("show_patch_update_universe", {
      ids: [1, 2],
      universe: 2,
    });
  });

  it("should invoke backend command when toggleGroup is called", async () => {
    const store = useShowPatch();

    await store.toggleGroup([1, 2], 3);

    expect(invoke).toHaveBeenCalledWith("show_patch_toggle_group", {
      ids: [1, 2],
      groupId: 3,
    });
  });

  it("should find fixture by id via getFixture", () => {
    const store = useShowPatch();
    store.fixtures = [mockPatchNode];

    expect(store.getFixture(1)).toEqual(mockPatchNode);
    expect(store.getFixture(999)).toBeUndefined();
  });
});
