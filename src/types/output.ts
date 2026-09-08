export type OutputTargetType = "none" | "physical" | "virtual";

export interface VirtualProtocol {
  id: string;
  name: string;
  description: string;
}

export const VIRTUAL_PROTOCOLS: VirtualProtocol[] = [
  {
    id: "artnet",
    name: "ArtNet",
    description: "Broadcasting via Art-Net protocol (Port 6454)",
  },
  {
    id: "sacn",
    name: "sCan (sACN)",
    description: "Streaming ACN protocol (E1.31 multicast)",
  },
  {
    id: "kinet",
    name: "KiNET",
    description: "Philips Color Kinetics protocol",
  },
  {
    id: "ola",
    name: "OLA",
    description: "Open Lighting Architecture daemon",
  },
  {
    id: "opc",
    name: "OPC",
    description: "Open Pixel Control protocol (Port 7890)",
  },
  {
    id: "osc",
    name: "OSC",
    description: "Open Sound Control messages",
  },
];

export interface PhysicalInterface {
  id: string;
  name: string;
  device: string;
  port: string;
  status: "connected" | "disconnected" | "busy";
  fps: number;
}

export interface UniverseOutputConfig {
  universe: number;
  output: string;
  enabled: boolean;
}
