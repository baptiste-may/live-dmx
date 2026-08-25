import { PaletteIcon, SlidersVerticalIcon } from "lucide-vue-next";
import { defineStore } from "pinia";
import type { Component } from "vue";

import LiveColors from "@/views/live/pages/LiveColors.vue";
import LiveIntensity from "@/views/live/pages/LiveIntensity.vue";

export enum LivePageId {
  INTENSITY = "intensity",
  COLORS = "colors",
}

export type LivePage = {
  name: string;
  icon: Component;
  component: Component;
};

export const EVERY_LIVE_PAGES: Record<LivePageId, LivePage> = {
  [LivePageId.INTENSITY]: {
    name: "Intensity",
    icon: SlidersVerticalIcon,
    component: LiveIntensity,
  },
  [LivePageId.COLORS]: {
    name: "Colors",
    icon: PaletteIcon,
    component: LiveColors,
  },
};

export const useLiveNavigation = defineStore("live-navigation", {
  state: () => ({
    currentPageId: LivePageId.INTENSITY,
  }),
  getters: {
    currentPage: (state) => EVERY_LIVE_PAGES[state.currentPageId],
  },
  actions: {
    navigate(pageId: LivePageId) {
      this.currentPageId = pageId;
    },
  },
});
