import type { FixtureMode, FixtureType } from "@/types/fixtures.ts";

export type CleanFixtureType = {
  name: string;
  shortName?: string;
  fixtureKey?: string;
  manufacturerKey?: string;
  categories?: string[];
};

export type CleanFixtureMode = {
  name?: string;
  shortName?: string;
  channels: (string | null)[];
  panBounds?: [number, number];
  tiltBounds?: [number, number];
};

export type PatchRequest = {
  name: string;
  fixtureType: FixtureType | CleanFixtureType;
  mode: FixtureMode | CleanFixtureMode;
  universe: number;
  address: number;
};

export type PatchBatchRequest = {
  name: string;
  fixtureType: FixtureType | CleanFixtureType;
  mode: FixtureMode | CleanFixtureMode;
  universe: number;
  startAddress: number;
  count: number;
  channelCount: number;
};

export type PatchedFixture = {
  id: number;
  groups: number[];
  name: string;
  fixtureType: FixtureType | CleanFixtureType;
  mode: FixtureMode | CleanFixtureMode;
  universe: number;
  address: number;
};
