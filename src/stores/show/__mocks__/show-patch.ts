import type { FixtureMode, FixtureType } from "@/types/fixtures";
import type { PatchedFixture } from "@/types/patch";

export const mockMode: FixtureMode = {
  name: "Mode",
  channels: ["ch1", "ch2"],
};

export const mockFixtureType: FixtureType = {
  name: "Fixture type",
  categories: ["cat1", "cat2"],
  fixtureKey: "fixtureKey",
  manufacturerKey: "manufacturerKey",
  modes: [mockMode],
};

export const mockPatchNode: PatchedFixture = {
  id: 1,
  name: "PAR 1",
  fixtureType: mockFixtureType,
  mode: mockFixtureType.modes[0],
  universe: 1,
  address: 1,
  groups: [0, 2],
};

export const mockFixture1: PatchedFixture = {
  id: 1,
  name: "Spot 1",
  groups: [1, 2],
  fixtureType: mockFixtureType,
  mode: mockMode,
  universe: 1,
  address: 1,
};

export const mockFixture2: PatchedFixture = {
  id: 2,
  name: "Spot 2",
  groups: [1],
  fixtureType: mockFixtureType,
  mode: mockMode,
  universe: 1,
  address: 11,
};
