import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

export interface OutputDmxState {
  universeBuffers: Record<number, number[]>;
  isPaused: boolean;
  lastRequestId: number;
}

export const useOutputDmx = defineStore("output-dmx", {
  state: (): OutputDmxState => ({
    universeBuffers: {},
    isPaused: false,
    lastRequestId: 0,
  }),
  getters: {
    getUniverseData:
      (state) =>
      (universe: number): number[] => {
        const buf =
          state.universeBuffers[universe] ??
          state.universeBuffers[String(universe) as unknown as number];
        return buf ?? Array.from({ length: 512 }, () => 0);
      },
    getActiveCount:
      (state) =>
      (universe: number): number => {
        const buf =
          state.universeBuffers[universe] ??
          state.universeBuffers[String(universe) as unknown as number];
        if (!buf) return 0;
        return buf.filter((val) => val > 0).length;
      },
    totalActiveCount: (state): number => {
      let count = 0;
      for (const buf of Object.values(state.universeBuffers)) {
        count += buf.filter((val) => val > 0).length;
      }
      return count;
    },
  },
  actions: {
    async fetchAllDmxData() {
      if (this.isPaused) {
        return;
      }
      const requestId = ++this.lastRequestId;
      try {
        const buffers = await invoke<Record<number, number[]>>("get_all_dmx_data");
        if (!this.isPaused && requestId === this.lastRequestId && buffers) {
          this.universeBuffers = buffers;
        }
      } catch {
        // Silently ignore during tests or if tauri is unavailable
      }
    },
    togglePause() {
      this.isPaused = !this.isPaused;
    },
    setUniverseData(universe: number, data: number[]) {
      this.universeBuffers[universe] = data;
    },
  },
});
