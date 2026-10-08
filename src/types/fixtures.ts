export interface GdtfCatalogMode {
  name: string;
  dmxfootprint: number;
  channels?: string[];
  panBounds?: [number, number];
  tiltBounds?: [number, number];
}

export interface GdtfCatalogItem {
  rid?: number;
  fixture: string;
  manufacturer: string;
  revision?: string;
  uploader?: string;
  rating?: number;
  modes: GdtfCatalogMode[];
  uuid?: string;
  filesize?: number;
  isInstalled: boolean;
  isGeneric: boolean;
  categories?: string[];
}

export interface ParsedGdtfMode {
  name: string;
  shortName?: string;
  channelCount: number;
  channels: string[];
  channelDefaults?: number[];
  shutterOpenValues?: Record<number, number>;
  panBounds?: [number, number];
  tiltBounds?: [number, number];
}

export interface EmbeddedGdtfMode {
  name: string;
  shortName?: string;
  channelCount: number;
  channels: string[];
  channelDefaults?: number[];
  shutterOpenValues?: Record<number, number>;
  panBounds?: [number, number];
  tiltBounds?: [number, number];
}

export interface EmbeddedGdtfDefinition {
  name: string;
  shortName?: string;
  manufacturer: string;
  description?: string;
  fixtureTypeId?: string;
  rid?: number;
  modes: EmbeddedGdtfMode[];
}

export interface NamedChannel {
  name: string;
  channel?: string;
  capability?: Record<string, unknown>;
  capabilities?: Record<string, unknown>[];
}

export type DmxChannelRef = string | NamedChannel | null;

export interface FixturePhysical {
  power?: number;
  lens?: {
    degreesMinMax?: [number, number];
  };
  weight?: number;
  DMXconnector?: string;
  bulb?: {
    type?: string;
  };
  dimensions?: [number, number, number];
}

export interface FixtureMode {
  name: string;
  shortName?: string;
  channels: DmxChannelRef[];
  dmxfootprint?: number;
  physical?: FixturePhysical;
  panBounds?: [number, number];
  tiltBounds?: [number, number];
}

export interface FixtureType {
  name: string;
  shortName?: string;
  manufacturer?: string;
  categories?: string[];
  description?: string;
  comment?: string;
  physical?: FixturePhysical;
  modes: FixtureMode[];
  fixtureKey: string;
  manufacturerKey: string;
  rid?: number;
  isInstalled?: boolean;
  isGeneric?: boolean;
  uploader?: string;
  rating?: number;
  revision?: string;
}

export interface Manufacturer {
  name: string;
  website?: string;
  comment?: string;
  fixtures: Record<string, FixtureType>;
}

export type Manufacturers = Record<string, Manufacturer>;
