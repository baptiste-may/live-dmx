import { defineStore } from "pinia";

import { useFixtureBrowser } from "@/stores/fixture-browser.ts";
import { useShowPatch } from "@/stores/show/show-patch.ts";
import type { FixtureMode, FixtureType } from "@/types/fixtures.ts";

export type PatchForm = {
  selectedFixture: undefined | FixtureType;
  selectedMode: undefined | FixtureMode;
  universe: number;
  address: number;
  count: number;
  name: string;
};

export const usePatchForm = defineStore("patch-form", {
  state: (): PatchForm => ({
    selectedFixture: undefined,
    selectedMode: undefined,
    universe: 1,
    address: 1,
    count: 1,
    name: "",
  }),
  actions: {
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
    patch() {
      if (!this.selectedFixture || !this.selectedMode) {
        return;
      }
      const show = useShowPatch();
      show.patchBatch({
        name: this.name,
        fixtureType: this.selectedFixture,
        mode: this.selectedMode,
        universe: this.universe,
        startAddress: this.address,
        count: this.count,
        channelCount: this.selectedMode.channels?.length ?? 1,
      });
      this.$reset();
    },
  },
});
