export type ColorPreference = "Color Mix" | "Color Wheel" | "CCT" | "Color presets";
export type PixelPreference = "Single" | "Individual" | "Grouped";

export type GroupSettings = {
  colorPreference: ColorPreference;
  pixelPreference: PixelPreference;
  pixelGroupCount: number;
  panMin: number;
  panMax: number;
  tiltMin: number;
  tiltMax: number;
  inversePan: boolean;
  inverseTilt: boolean;
  maxPower: number;
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
  hasPixels: boolean;
  panBounds: [number, number];
  tiltBounds: [number, number];
  colorOptions: ColorPreference[];
  pixelOptions: PixelPreference[];
  maxPixels: number;
  settings: GroupSettings;
};

export type ConfigureOverview = {
  rows: GroupRow[];
  groups: Record<number, GroupConfigureInfo>;
};
