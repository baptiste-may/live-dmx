import type { FixtureMode, FixtureType } from "@/types/fixtures.ts";

export type PatchRequest = {
  name: string;
  fixtureType: FixtureType;
  mode: FixtureMode;
  universe: number;
  address: number;
};

export type PatchBatchRequest = {
  name: string;
  fixtureType: FixtureType;
  mode: FixtureMode;
  universe: number;
  startAddress: number;
  count: number;
  channelCount: number;
};

export type PatchedFixture = PatchRequest & {
  id: number;
  groups: number[];
};
