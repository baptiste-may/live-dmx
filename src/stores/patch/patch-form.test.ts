import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { mockFixturesData } from "@/stores/__mocks__/fixture-browser";
import { useFixtureBrowser } from "@/stores/fixture-browser";
import { useShowPatch } from "@/stores/show/show-patch";
import { ToastType, useToastStore } from "@/stores/toast";
import type { FixtureMode, FixtureType } from "@/types/fixtures";
import type { PatchedFixture } from "@/types/patch";

import {
  findPatchCollision,
  getBatchPatchedRanges,
  getNextAvailableAddress,
  usePatchForm,
} from "./patch-form";

describe("patch-form helpers", () => {
  describe("getNextAvailableAddress", () => {
    it("should return universe 1, address 1 for empty fixtures", () => {
      expect(getNextAvailableAddress([])).toEqual({ universe: 1, address: 1 });
    });

    it("should calculate next address in the same universe", () => {
      const fixtures: PatchedFixture[] = [
        {
          id: 1,
          name: "Spot 1",
          groups: [],
          fixtureType: {} as FixtureType,
          mode: { name: "4ch", channels: ["dimmer", "pan", "tilt", "strobe"] },
          universe: 1,
          address: 1,
        },
      ];

      expect(getNextAvailableAddress(fixtures)).toEqual({ universe: 1, address: 5 });
    });

    it("should roll over to next universe when universe is full", () => {
      const fixtures: PatchedFixture[] = [
        {
          id: 1,
          name: "Full Universe Fixture",
          groups: [],
          fixtureType: {} as FixtureType,
          mode: { name: "13ch", channels: new Array(13).fill("ch") },
          universe: 1,
          address: 500,
        },
      ];

      // 500 + 13 - 1 = 512, next available is U2, A1
      expect(getNextAvailableAddress(fixtures)).toEqual({ universe: 2, address: 1 });
    });

    it("should find the maximum address across multiple universes", () => {
      const fixtures: PatchedFixture[] = [
        {
          id: 1,
          name: "U1 Fixture",
          groups: [],
          fixtureType: {} as FixtureType,
          mode: { name: "10ch", channels: new Array(10).fill("ch") },
          universe: 1,
          address: 1,
        },
        {
          id: 2,
          name: "U3 Fixture",
          groups: [],
          fixtureType: {} as FixtureType,
          mode: { name: "6ch", channels: new Array(6).fill("ch") },
          universe: 3,
          address: 100,
        },
      ];

      // U3 address 100..105 -> next is U3 address 106
      expect(getNextAvailableAddress(fixtures)).toEqual({ universe: 3, address: 106 });
    });

    it("should handle default 1 channel when mode has empty channels", () => {
      const fixtures: PatchedFixture[] = [
        {
          id: 1,
          name: "Dimmer",
          groups: [],
          fixtureType: {} as FixtureType,
          mode: { name: "1ch" } as FixtureMode,
          universe: 1,
          address: 10,
        },
      ];

      expect(getNextAvailableAddress(fixtures)).toEqual({ universe: 1, address: 11 });
    });
  });

  describe("getBatchPatchedRanges", () => {
    it("should calculate ranges for multiple fixtures in batch", () => {
      const ranges = getBatchPatchedRanges({
        universe: 1,
        startAddress: 1,
        count: 3,
        channelCount: 5,
      });

      expect(ranges).toEqual([
        { universe: 1, startAddress: 1, endAddress: 5 },
        { universe: 1, startAddress: 6, endAddress: 10 },
        { universe: 1, startAddress: 11, endAddress: 15 },
      ]);
    });

    it("should rollover to next universe if fixture exceeds universe boundary", () => {
      const ranges = getBatchPatchedRanges({
        universe: 1,
        startAddress: 510,
        count: 2,
        channelCount: 4,
      });

      // 510..513 doesn't fit in 512, so rolls to U2 A1..4
      expect(ranges).toEqual([
        { universe: 2, startAddress: 1, endAddress: 4 },
        { universe: 2, startAddress: 5, endAddress: 8 },
      ]);
    });
  });

  describe("findPatchCollision", () => {
    const existingFixtures: PatchedFixture[] = [
      {
        id: 1,
        name: "Spot 1",
        groups: [],
        fixtureType: {} as FixtureType,
        mode: { name: "4ch", channels: ["ch1", "ch2", "ch3", "ch4"] },
        universe: 1,
        address: 10,
      },
    ];

    it("should return null when no collision occurs", () => {
      const collision = findPatchCollision(existingFixtures, {
        universe: 1,
        startAddress: 1,
        count: 1,
        channelCount: 5,
      });

      expect(collision).toBeNull();
    });

    it("should detect direct collision on same universe", () => {
      const collision = findPatchCollision(existingFixtures, {
        universe: 1,
        startAddress: 12,
        count: 1,
        channelCount: 2,
      });

      expect(collision).not.toBeNull();
      expect(collision?.collidingFixture.name).toBe("Spot 1");
      expect(collision?.newUniverse).toBe(1);
    });

    it("should not detect collision on different universe", () => {
      const collision = findPatchCollision(existingFixtures, {
        universe: 2,
        startAddress: 10,
        count: 1,
        channelCount: 4,
      });

      expect(collision).toBeNull();
    });

    it("should detect collision when a subsequent fixture in batch collides", () => {
      const collision = findPatchCollision(existingFixtures, {
        universe: 1,
        startAddress: 5,
        count: 2,
        channelCount: 5,
      });

      // Fixture 1 is 5..9 (no collision), Fixture 2 is 10..14 (collides with 10..13)
      expect(collision).not.toBeNull();
      expect(collision?.collidingFixture.name).toBe("Spot 1");
    });

    it("should detect universe limit exceedance", () => {
      const collision = findPatchCollision([], {
        universe: 513,
        startAddress: 1,
        count: 1,
        channelCount: 1,
      });

      expect(collision).not.toBeNull();
      expect(collision?.collidingFixture.id).toBe(-1);
    });
  });
});

describe("patch-form store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with max possible address when fixtures exist", () => {
    const showPatch = useShowPatch();
    showPatch.fixtures = [
      {
        id: 1,
        name: "Existing Fixture",
        groups: [],
        fixtureType: {} as FixtureType,
        mode: { name: "4ch", channels: ["ch1", "ch2", "ch3", "ch4"] },
        universe: 1,
        address: 1,
      },
    ];

    const store = usePatchForm();
    store.setDefaultLocation();

    expect(store.universe).toBe(1);
    expect(store.address).toBe(5);
  });

  it("should update selected fixture data without overwriting custom address/universe", () => {
    const fixtureBrowser = useFixtureBrowser();
    fixtureBrowser.manufacturers = mockFixturesData;

    const store = usePatchForm();
    store.universe = 3;
    store.address = 42;

    store.select("chauvet", "gig-bar-2", 0);

    expect(store.selectedFixture?.manufacturerKey).toBe("chauvet");
    expect(store.selectedFixture?.fixtureKey).toBe("gig-bar-2");
    expect(store.selectedMode?.name).toBe("Standard");
    expect(store.universe).toBe(3);
    expect(store.address).toBe(42);
  });

  it("should reset form and recalculate max address on reset", () => {
    const showPatch = useShowPatch();
    showPatch.fixtures = [
      {
        id: 1,
        name: "Existing",
        groups: [],
        fixtureType: {} as FixtureType,
        mode: { name: "2ch", channels: ["ch1", "ch2"] },
        universe: 2,
        address: 20,
      },
    ];

    const store = usePatchForm();
    store.name = "Custom Name";
    store.count = 5;

    store.reset();

    expect(store.selectedFixture).toBeUndefined();
    expect(store.selectedMode).toBeUndefined();
    expect(store.name).toBe("");
    expect(store.count).toBe(1);
    expect(store.universe).toBe(2);
    expect(store.address).toBe(22);
  });

  it("should prevent patch and show toast error when collision occurs without resetting inputs", async () => {
    const toastStore = useToastStore();
    const showPatch = useShowPatch();
    const patchBatchSpy = vi.spyOn(showPatch, "patchBatch");

    showPatch.fixtures = [
      {
        id: 1,
        name: "Colliding Spot",
        groups: [],
        fixtureType: {} as FixtureType,
        mode: { name: "4ch", channels: ["ch1", "ch2", "ch3", "ch4"] },
        universe: 1,
        address: 10,
      },
    ];

    const fixtureBrowser = useFixtureBrowser();
    fixtureBrowser.manufacturers = mockFixturesData;

    const store = usePatchForm();
    store.select("chauvet", "gig-bar-2", 0);

    // Manually set address to cause collision
    store.universe = 1;
    store.address = 10;
    store.name = "My Fixture";

    await store.patch();

    // Patch must NOT be called
    expect(patchBatchSpy).not.toHaveBeenCalled();

    // Inputs must STAY IN PLACE
    expect(store.selectedFixture).toBeDefined();
    expect(store.selectedMode).toBeDefined();
    expect(store.name).toBe("My Fixture");
    expect(store.universe).toBe(1);
    expect(store.address).toBe(10);

    // Toast error should be added
    expect(toastStore.toasts.length).toBeGreaterThan(0);
    expect(toastStore.toasts[0].type).toBe(ToastType.ERROR);
    expect(toastStore.toasts[0].message).toContain("Colliding Spot");
  });

  it("should successfully patch and reset form when no collision occurs", async () => {
    const showPatch = useShowPatch();
    const patchBatchSpy = vi.spyOn(showPatch, "patchBatch").mockResolvedValue(undefined as never);

    showPatch.fixtures = [
      {
        id: 1,
        name: "Existing",
        groups: [],
        fixtureType: {} as FixtureType,
        mode: { name: "2ch", channels: ["ch1", "ch2"] },
        universe: 1,
        address: 1,
      },
    ];

    const fixtureBrowser = useFixtureBrowser();
    fixtureBrowser.manufacturers = mockFixturesData;

    const store = usePatchForm();
    store.setDefaultLocation();
    store.select("chauvet", "gig-bar-2", 0);

    expect(store.universe).toBe(1);
    expect(store.address).toBe(3);

    await store.patch();

    expect(patchBatchSpy).toHaveBeenCalledTimes(1);
    expect(store.selectedFixture).toBeUndefined();
    expect(store.selectedMode).toBeUndefined();
  });

  it("should send cleaned fixtureType and mode when patching", async () => {
    const showPatch = useShowPatch();
    const patchBatchSpy = vi.spyOn(showPatch, "patchBatch").mockResolvedValue(undefined as never);

    const fixtureBrowser = useFixtureBrowser();
    fixtureBrowser.manufacturers = mockFixturesData;

    const store = usePatchForm();
    store.select("chauvet", "gig-bar-2", 0);

    await store.patch();

    expect(patchBatchSpy).toHaveBeenCalledTimes(1);
    const callArg = patchBatchSpy.mock.calls[0][0];
    expect(callArg.fixtureType).toEqual({
      name: "GigBAR 2",
      fixtureKey: "gig-bar-2",
      manufacturerKey: "chauvet",
    });
    expect(callArg.mode).toEqual({
      name: "Standard",
      channels: ["Dimmer", "Red", "Green", "Blue"],
    });
  });

  it("should do nothing when patch is called without selected fixture", async () => {
    const showPatch = useShowPatch();
    const patchBatchSpy = vi.spyOn(showPatch, "patchBatch");

    const store = usePatchForm();
    await store.patch();

    expect(patchBatchSpy).not.toHaveBeenCalled();
  });
});
