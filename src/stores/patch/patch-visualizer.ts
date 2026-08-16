import { defineStore } from "pinia";

export const usePatchVisualizer = defineStore("patch-visualizer", {
  state: () => ({
    currentUniverse: 1,
  }),
});
