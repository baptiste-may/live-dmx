export interface FixtureMeta {
  authors: string[];
  createDate: string;
  lastModifyDate: string;
  importPlugin?: {
    plugin: string;
    date: string;
    comment?: string;
  };
}

export interface FixtureLinks {
  manual?: string[];
  productPage?: string[];
  video?: string[];
  forum?: string[];
}

export interface FixtureBulb {
  type?: string;
  colorTemperature?: number;
  lumens?: number;
}

export interface FixtureLens {
  name?: string;
  degreesMinMax?: [number, number];
}

export interface MatrixPixelsPhysical {
  dimensions?: [number, number, number];
  spacing?: [number, number, number];
}

export interface FixturePhysical {
  dimensions?: [number, number, number];
  weight?: number;
  power?: number;
  DMXconnector?: string;
  bulb?: FixtureBulb;
  lens?: FixtureLens;
  powerConnectors?: Record<string, string>;
  matrixPixels?: MatrixPixelsPhysical;
}

export interface GoboSlot {
  name?: string;
  resource?: string;
}

export interface WheelSlot {
  type: string;
  name?: string;
  colors?: string[];
  resource?: string;
  gobo?: string | GoboSlot;
}

export interface FixtureWheel {
  direction?: "CW" | "CCW" | string;
  slots?: WheelSlot[];
}

export interface FixtureMatrix {
  pixelKeys?: (string | null)[][][];
  pixelGroups?: Record<string, string[]>;
  pixelCount?: [number, number, number];
}

export interface FixtureCapability {
  dmxRange?: [number, number];
  type: string;
  comment?: string;
  helpWanted?: string;
  menuClick?: string;
  switchChannels?: Record<string, string>;
  effectPreset?: string;
  effectName?: string;
  soundControlled?: boolean;
  soundSensitivity?: number | string;
  speed?: number | string;
  speedStart?: number | string;
  speedEnd?: number | string;
  parameter?: number | string;
  parameterStart?: number | string;
  parameterEnd?: number | string;
  angle?: number | string;
  angleStart?: number | string;
  angleEnd?: number | string;
  horizontalAngle?: number | string;
  horizontalAngleStart?: number | string;
  horizontalAngleEnd?: number | string;
  verticalAngle?: number | string;
  verticalAngleStart?: number | string;
  verticalAngleEnd?: number | string;
  color?: string;
  colors?: string[];
  colorsStart?: string[];
  colorsEnd?: string[];
  colorTemperature?: number | string;
  colorTemperatureStart?: number | string;
  colorTemperatureEnd?: number | string;
  brightness?: number | string;
  brightnessStart?: number | string;
  brightnessEnd?: number | string;
  slotNumber?: number;
  slotNumberStart?: number;
  slotNumberEnd?: number;
  wheel?: string | string[];
  shutterEffect?: string;
  randomTiming?: boolean;
  fogType?: string;
  fogOutput?: number | string;
  fogOutputStart?: number | string;
  fogOutputEnd?: number | string;
  time?: number | string;
  timeStart?: number | string;
  timeEnd?: number | string;
  duration?: number | string;
  durationStart?: number | string;
  durationEnd?: number | string;
  hold?: number | string;
  distance?: number | string;
  distanceStart?: number | string;
  distanceEnd?: number | string;
  openPercent?: number;
  openPercentStart?: number;
  openPercentEnd?: number;
  frostIntensity?: number | string;
  frostIntensityStart?: number | string;
  frostIntensityEnd?: number | string;
  shakeSpeed?: number | string;
  shakeSpeedStart?: number | string;
  shakeSpeedEnd?: number | string;
  shakeAngle?: number | string;
  shakeAngleStart?: number | string;
  shakeAngleEnd?: number | string;
  blade?: number | string;
  insertionStart?: number | string;
  insertionEnd?: number | string;
  isShaking?: boolean | string;
}

export interface FixtureChannel {
  name?: string;
  fineChannelAliases?: string[];
  defaultValue?: number | string;
  highlightValue?: number | string;
  precedence?: "HTP" | "LTP" | string;
  constant?: boolean;
  dmxValueResolution?: "8bit" | "16bit" | "24bit" | string;
  capability?: FixtureCapability;
  capabilities?: FixtureCapability[];
}

export interface NamedChannel {
  name: string;
  fineChannelAliases?: string[];
  defaultValue?: number | string;
  highlightValue?: number | string;
  precedence?: "HTP" | "LTP" | string;
  constant?: boolean;
  dmxValueResolution?: "8bit" | "16bit" | "24bit" | string;
  capability?: FixtureCapability;
  capabilities?: FixtureCapability[];
  insert?: string;
  repeatFor?: string[];
  channel?: string;
}

export type DmxChannelRef = string | NamedChannel | null;

export interface FixtureMode {
  name: string;
  shortName?: string;
  channels: DmxChannelRef[];
  physical?: FixturePhysical;
  rdmPersonalityIndex?: number;
}

export interface FixtureRdm {
  modelId?: number;
  softwareVersion?: string;
}

export interface FixtureType {
  name: string;
  shortName?: string;
  categories: string[];
  meta?: FixtureMeta;
  links?: FixtureLinks;
  physical?: FixturePhysical;
  matrix?: FixtureMatrix;
  wheels?: Record<string, FixtureWheel>;
  availableChannels?: Record<string, FixtureChannel>;
  templateChannels?: Record<string, FixtureChannel>;
  modes: FixtureMode[];
  fixtureKey: string;
  manufacturerKey: string;
  oflURL?: string;
  comment?: string;
  helpWanted?: string | boolean;
  rdm?: FixtureRdm;
}

export interface Manufacturer {
  name: string;
  website?: string;
  comment?: string;
  rdmId?: number;
  fixtures: Record<string, FixtureType>;
}

export type Manufacturers = Record<string, Manufacturer>;
