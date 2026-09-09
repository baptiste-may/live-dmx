import {
  ActivityIcon,
  ArrowBigLeftDashIcon,
  ArrowBigRightDashIcon,
  ChartNoAxesColumnDecreasingIcon,
  ChartNoAxesColumnIncreasingIcon,
  DicesIcon,
  FlipHorizontal2Icon,
  FoldHorizontalIcon,
  RainbowIcon,
  SquareSplitHorizontalIcon,
  SunIcon,
  WavesIcon,
} from "lucide-vue-next";
import type { Component } from "vue";

import { FourIcon, ThreeIcon, TwoIcon } from "@/components/icons";

export type FlashType = "simple" | "white" | "strobe" | "randomStrobe" | "master";

export type EffectType = "pulse" | "chase" | "ramp_up" | "ramp_down" | "sine";

export type IntensityEffectType = "full" | EffectType;

export type ColorEffectType = EffectType | "rainbow";

export type AlignmentType = "rng" | "left" | "right" | "wing" | "center" | "two" | "three" | "four";

export type ColorFadeOrder = "left" | "right" | "wing" | "center";

export type GroupEffectState = {
  effect: IntensityEffectType | null;
  alignment: AlignmentType | null;
  bounce: boolean;
};

export type GroupColorEffectState = {
  effect: ColorEffectType | null;
  alignment: AlignmentType | null;
  bounce: boolean;
};

export type LiveButtonsState = {
  flashStates: Record<number, Record<FlashType, boolean>>;
  buttonStates: Record<string, boolean>;
  groupEffects: Record<number, GroupEffectState>;
  groupColorEffects: Record<number, GroupColorEffectState>;
  groupMasters: Record<number, number>;
  groupColors: Record<number, string | null>;
  groupSecondaryColors: Record<number, string | null>;
  strobeRate: number;
  randomStrobeRate: number;
  simpleFlashFade: number;
  whiteFlashFade: number;
  effectBpm: number;
  colorEffectBpm: number;
  effectWidth: number;
  colorEffectWidth: number;
  colorFadeSpeed: number;
  colorFadeOrder: ColorFadeOrder | null;
};

export const effects: Record<
  EffectType,
  {
    label: string;
    icon: Component;
  }
> = {
  pulse: {
    label: "Pulse",
    icon: ActivityIcon,
  },
  chase: {
    label: "Chase",
    icon: SquareSplitHorizontalIcon,
  },
  ramp_up: {
    label: "Ramp up",
    icon: ChartNoAxesColumnIncreasingIcon,
  },
  ramp_down: {
    label: "Ramp down",
    icon: ChartNoAxesColumnDecreasingIcon,
  },
  sine: {
    label: "Sine",
    icon: WavesIcon,
  },
};

export const intensityEffects: Record<IntensityEffectType, { label: string; icon: Component }> = {
  full: {
    label: "Full",
    icon: SunIcon,
  },
  ...effects,
};

export const colorEffects: Record<
  ColorEffectType,
  {
    label: string;
    icon: Component;
  }
> = {
  ...effects,
  rainbow: {
    label: "Rainbow",
    icon: RainbowIcon,
  },
};

export const alignments: Record<
  AlignmentType,
  {
    label: string;
    icon: Component;
  }
> = {
  rng: {
    label: "Random",
    icon: DicesIcon,
  },
  left: {
    label: "Left",
    icon: ArrowBigLeftDashIcon,
  },
  right: {
    label: "Right",
    icon: ArrowBigRightDashIcon,
  },
  wing: {
    label: "Wing",
    icon: FlipHorizontal2Icon,
  },
  center: {
    label: "Center",
    icon: FoldHorizontalIcon,
  },
  two: {
    label: "groups",
    icon: TwoIcon,
  },
  three: {
    label: "groups",
    icon: ThreeIcon,
  },
  four: {
    label: "groups",
    icon: FourIcon,
  },
};

export const colors: Record<string, string> = {
  white: "#FFFFFF",
  ctb: "#C6E6FF",
  cto: "#FFC78A",
  red: "#FF0000",
  orange: "#FF7F00",
  yellow: "#FFFF00",
  lime: "#7FFF00",
  green: "#00FF00",
  mint: "#00FF7F",
  cyan: "#00FFFF",
  azure: "#007FFF",
  blue: "#0000FF",
  violet: "#7F00FF",
  magenta: "#FF00FF",
  pink: "#FF007F",
};
