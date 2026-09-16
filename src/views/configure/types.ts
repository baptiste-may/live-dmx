export type ColorPreference = "Color Mix" | "Color Wheel" | "CCT" | "Color presets";

export type GroupSettings = {
  colorPreference: ColorPreference;
  panMin: number;
  panMax: number;
  tiltMin: number;
  tiltMax: number;
  inversePan: boolean;
  inverseTilt: boolean;
};

export type GroupRow = {
  id: number;
  name: string;
  fixtureCount: number;
  children: GroupRow[];
};

export type GroupConfigureInfo = {
  id: number;
  name: string;
  fixtureCount: number;
  hasPan: boolean;
  hasTilt: boolean;
  hasColor: boolean;
  panBounds: [number, number];
  tiltBounds: [number, number];
  colorOptions: ColorPreference[];
  settings: GroupSettings;
};

export type ConfigureOverview = {
  rows: GroupRow[];
  groups: Record<number, GroupConfigureInfo>;
};
