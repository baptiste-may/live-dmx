import {
  CommandIcon,
  EthernetPortIcon,
  HammerIcon,
  PaletteIcon,
  SlidersVerticalIcon,
  SunIcon,
} from "lucide-vue-next";
import { defineStore } from "pinia";
import type { Component } from "vue";

import Configure from "@/views/Configure.vue";
import Input from "@/views/Input.vue";
import LiveColors from "@/views/live/pages/LiveColors.vue";
import LiveIntensity from "@/views/live/pages/LiveIntensity.vue";
import Output from "@/views/Output.vue";
import Patch from "@/views/Patch.vue";

export enum PageId {
  INPUT = "input",
  OUTPUT = "output",
  PATCH = "patch",
  CONFIG = "config",
  INTENSITY = "intensity",
  COLORS = "colors",
}

export interface NavigationPage {
  id: PageId;
  name: string;
  icon: Component;
  component: Component;
}

export interface NavigationSection {
  title: string;
  pages: NavigationPage[];
}

export const NAVIGATION_SECTIONS: NavigationSection[] = [
  {
    title: "Setup",
    pages: [
      {
        id: PageId.INPUT,
        name: "Input",
        icon: CommandIcon,
        component: Input,
      },
      {
        id: PageId.OUTPUT,
        name: "Output",
        icon: EthernetPortIcon,
        component: Output,
      },
      {
        id: PageId.PATCH,
        name: "Patch",
        icon: HammerIcon,
        component: Patch,
      },
      {
        id: PageId.CONFIG,
        name: "Configure",
        icon: SlidersVerticalIcon,
        component: Configure,
      },
    ],
  },
  {
    title: "Live",
    pages: [
      {
        id: PageId.INTENSITY,
        name: "Intensity",
        icon: SunIcon,
        component: LiveIntensity,
      },
      {
        id: PageId.COLORS,
        name: "Colors",
        icon: PaletteIcon,
        component: LiveColors,
      },
    ],
  },
];

export const EVERY_PAGES: NavigationPage[] = NAVIGATION_SECTIONS.flatMap(
  (section) => section.pages,
);

export const useNavigation = defineStore("navigation", {
  state: (): { currentPage: PageId } => ({
    currentPage: PageId.PATCH,
  }),
  actions: {
    navigate(target: PageId) {
      this.currentPage = target;
    },
    getCurrentPage() {
      return EVERY_PAGES.find((page) => page.id === this.currentPage);
    },
  },
});
