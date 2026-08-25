import { defineStore } from "pinia";

import { useFixtureBrowser } from "@/stores/fixture-browser.ts";
import { useShowPatch } from "@/stores/show/show-patch.ts";
import { ToastType, useToastStore } from "@/stores/toast.ts";
import type { FixtureMode, FixtureType } from "@/types/fixtures.ts";
import type { CleanFixtureMode, CleanFixtureType, PatchedFixture } from "@/types/patch.ts";

export type PatchForm = {
  selectedFixture: undefined | FixtureType;
  selectedMode: undefined | FixtureMode;
  universe: number;
  address: number;
  count: number;
  name: string;
};

export function getNextAvailableAddress(fixtures: PatchedFixture[]): {
  universe: number;
  address: number;
} {
  if (!fixtures || fixtures.length === 0) {
    return { universe: 1, address: 1 };
  }
  let maxEndAbs = 0;
  for (const f of fixtures) {
    const channelCount = Math.max(1, f.mode?.channels?.length ?? 1);
    const endAbs = (f.universe - 1) * 512 + (f.address - 1) + channelCount;
    if (endAbs > maxEndAbs) {
      maxEndAbs = endAbs;
    }
  }
  const universe = Math.min(512, Math.floor(maxEndAbs / 512) + 1);
  const address = Math.min(512, (maxEndAbs % 512) + 1);
  return { universe, address };
}

export function getBatchPatchedRanges(request: {
  universe: number;
  startAddress: number;
  count: number;
  channelCount: number;
}): { universe: number; startAddress: number; endAddress: number }[] {
  const ranges: { universe: number; startAddress: number; endAddress: number }[] = [];
  let currentAbs = (request.universe - 1) * 512 + (request.startAddress - 1);
  const channelCount = Math.max(1, request.channelCount);

  for (let i = 0; i < request.count; i++) {
    const offsetInUniverse = currentAbs % 512;
    if (offsetInUniverse + channelCount > 512) {
      currentAbs += 512 - offsetInUniverse;
    }
    const u = Math.floor(currentAbs / 512) + 1;
    const a = (currentAbs % 512) + 1;
    ranges.push({
      universe: u,
      startAddress: a,
      endAddress: a + channelCount - 1,
    });
    currentAbs += channelCount;
  }
  return ranges;
}

export function findPatchCollision(
  existingFixtures: PatchedFixture[],
  request: {
    universe: number;
    startAddress: number;
    count: number;
    channelCount: number;
  },
): { collidingFixture: PatchedFixture; newUniverse: number; newAddress: number } | null {
  const newRanges = getBatchPatchedRanges(request);

  for (const newRange of newRanges) {
    if (newRange.universe > 512 || newRange.endAddress > 512) {
      return {
        collidingFixture: {
          id: -1,
          name: "Universe limit exceeded",
          groups: [],
          fixtureType: {} as FixtureType,
          mode: {} as FixtureMode,
          universe: newRange.universe,
          address: newRange.startAddress,
        },
        newUniverse: newRange.universe,
        newAddress: newRange.startAddress,
      };
    }

    for (const existing of existingFixtures) {
      if (existing.universe !== newRange.universe) {
        continue;
      }
      const existingChannelCount = Math.max(1, existing.mode?.channels?.length ?? 1);
      const existingStart = existing.address;
      const existingEnd = existing.address + existingChannelCount - 1;

      if (existingStart <= newRange.endAddress && newRange.startAddress <= existingEnd) {
        return {
          collidingFixture: existing,
          newUniverse: newRange.universe,
          newAddress: newRange.startAddress,
        };
      }
    }
  }

  return null;
}

export const usePatchForm = defineStore("patch-form", {
  state: (): PatchForm => {
    let universe = 1;
    let address = 1;
    try {
      const show = useShowPatch();
      const next = getNextAvailableAddress(show.fixtures);
      universe = next.universe;
      address = next.address;
    } catch {
      // Pinia might not be ready yet in some contexts
    }
    return {
      selectedFixture: undefined,
      selectedMode: undefined,
      universe,
      address,
      count: 1,
      name: "",
    };
  },
  actions: {
    setDefaultLocation() {
      const show = useShowPatch();
      const next = getNextAvailableAddress(show.fixtures);
      this.universe = next.universe;
      this.address = next.address;
    },
    select(manufacturerKey: string, fixtureKey: string, modeIndex: number) {
      const fixture = useFixtureBrowser().getFixture(manufacturerKey, fixtureKey);
      if (fixture === null) {
        return;
      }
      this.selectedFixture = fixture;
      this.selectedMode = fixture.modes[modeIndex];
      this.name = `${fixture.name} 1`;
    },
    isFixtureSelected() {
      return this.selectedFixture !== undefined && this.selectedMode !== undefined;
    },
    reset() {
      this.selectedFixture = undefined;
      this.selectedMode = undefined;
      this.count = 1;
      this.name = "";
      this.setDefaultLocation();
    },
    async patch() {
      if (!this.selectedFixture || !this.selectedMode) {
        return;
      }
      const show = useShowPatch();
      const channelCount = this.selectedMode.channels?.length ?? 1;

      const collision = findPatchCollision(show.fixtures, {
        universe: this.universe,
        startAddress: this.address,
        count: this.count,
        channelCount,
      });

      if (collision) {
        const msg =
          collision.collidingFixture.id === -1
            ? "Cannot patch: exceeds DMX universe limits."
            : `Cannot patch: address collision on Universe ${collision.newUniverse} with '${collision.collidingFixture.name}'.`;
        useToastStore().addToast(msg, ToastType.ERROR);
        return;
      }

      const cleanFixtureType: CleanFixtureType = {
        name: this.selectedFixture.name,
        ...(this.selectedFixture.shortName ? { shortName: this.selectedFixture.shortName } : {}),
        ...(this.selectedFixture.fixtureKey ? { fixtureKey: this.selectedFixture.fixtureKey } : {}),
        ...(this.selectedFixture.manufacturerKey
          ? { manufacturerKey: this.selectedFixture.manufacturerKey }
          : {}),
      };

      const cleanMode: CleanFixtureMode = {
        name: this.selectedMode.name,
        ...(this.selectedMode.shortName ? { shortName: this.selectedMode.shortName } : {}),
        channels: (this.selectedMode.channels || []).map((ch) => {
          if (typeof ch === "string") return ch;
          if (ch && typeof ch === "object") {
            if ("name" in ch && typeof ch.name === "string") return ch.name;
            if ("channel" in ch && typeof ch.channel === "string") return ch.channel;
          }
          return "";
        }),
      };

      try {
        await show.patchBatch({
          name: this.name,
          fixtureType: cleanFixtureType,
          mode: cleanMode,
          universe: this.universe,
          startAddress: this.address,
          count: this.count,
          channelCount,
        });
        this.reset();
      } catch (error) {
        useToastStore().addToast(`Failed to patch fixtures: ${String(error)}`, ToastType.ERROR);
      }
    },
  },
});
