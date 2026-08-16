import type { FixtureMode, FixtureType } from "@/types/fixtures";
import type { PatchedFixture } from "@/types/patch";

export const mockMode: FixtureMode = {
  name: "Standard",
  shortName: "Std",
  channels: ["Dimmer", "Pan", "Tilt"],
};

export const mockFixtureType: FixtureType = {
  name: "Spot 500",
  categories: ["Moving Head", "Color Changer"],
  modes: [mockMode],
  fixtureKey: "spot-500",
  manufacturerKey: "generic",
  comment: "A versatile moving head spot",
};

export const mockTooltipFixture: FixtureType = {
  name: "Mini Spot",
  categories: ["Spot"],
  fixtureKey: "mini-spot",
  manufacturerKey: "cameo",
  comment: "A cool fixture",
  modes: [
    {
      name: "7-Channel",
      shortName: "7ch",
      channels: ["Dimmer", "Strobe", "Red", "Green", "Blue", "Pan", "Tilt"],
    },
  ],
};

export const mockPatchedFixture: PatchedFixture = {
  id: 1,
  name: "Spot 1",
  universe: 1,
  address: 1,
  groups: [0],
  fixtureType: mockFixtureType,
  mode: mockMode,
};

export const mockTableRowFixture: PatchedFixture = {
  id: 5,
  name: "Front Par 5",
  universe: 2,
  address: 12,
  groups: [0],
  fixtureType: {
    name: "LED Wash",
    categories: ["Wash"],
    modes: [],
    fixtureKey: "led-wash",
    manufacturerKey: "generic",
  },
  mode: {
    name: "Standard",
    shortName: "Std",
    channels: ["Red", "Green", "Blue"],
  },
};
