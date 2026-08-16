import {
  CommandIcon,
  EthernetPortIcon,
  HammerIcon,
  RadioIcon,
  SlidersVerticalIcon,
} from "lucide-vue-next";
import { defineStore } from "pinia";
import type { Component } from "vue";

import Configure from "@/views/Configure.vue";
import Input from "@/views/Input.vue";
import Live from "@/views/Live.vue";
import Output from "@/views/Output.vue";
import Patch from "@/views/Patch.vue";

export enum PageId {
  INPUT = "input",
  OUTPUT = "output",
  PATCH = "patch",
  CONFIG = "config",
  LIVE = "live",
}

export interface NavigationPage {
  id: PageId;
  name: string;
  icon: Component;
  component: Component;
}

export const EVERY_PAGES: NavigationPage[] = [
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
  {
    id: PageId.LIVE,
    name: "Live",
    icon: RadioIcon,
    component: Live,
  },
];

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
