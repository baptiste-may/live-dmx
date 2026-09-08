import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

import { ToastType, useToastStore } from "@/stores/toast.ts";
import type { UniverseOutputConfig } from "@/types/output.ts";

export const DEFAULT_UNIVERSES: UniverseOutputConfig[] = [
  { universe: 1, output: "none", enabled: true },
];

export const useOutputUniverses = defineStore("output-universes", {
  state: () => ({
    universes: DEFAULT_UNIVERSES.map((u) => ({ ...u })),
  }),
  getters: {
    assignedUniversesByInterface: (state) => {
      const mapping: Record<string, number[]> = {};
      for (const u of state.universes) {
        if (u.enabled && u.output.startsWith("physical")) {
          if (!mapping[u.output]) {
            mapping[u.output] = [];
          }
          mapping[u.output].push(u.universe);
        }
      }
      return mapping;
    },
  },
  actions: {
    async fetchUniverses() {
      try {
        const universes = await invoke<UniverseOutputConfig[]>("show_output_get_universes");
        if (Array.isArray(universes) && universes.length > 0) {
          this.universes = universes;
        }
      } catch (err: unknown) {
        if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
          useToastStore().addToast(
            `Failed to load universe configurations: ${String(err)}`,
            ToastType.ERROR,
          );
        }
      }
    },
    async setUniverseOutput(universe: number, output: string) {
      const target = this.universes.find((u) => u.universe === universe);
      if (!target) {
        return;
      }
      const previous = target.output;
      target.output = output;
      try {
        await invoke("show_output_set_target", { universe, output });
      } catch (err: unknown) {
        if (target.output === output) {
          target.output = previous;
        }
        useToastStore().addToast(
          `Failed to update universe output: ${String(err)}`,
          ToastType.ERROR,
        );
      }
    },
    async setUniverseEnabled(universe: number, enabled: boolean) {
      const target = this.universes.find((u) => u.universe === universe);
      if (!target) {
        return;
      }
      const previous = target.enabled;
      target.enabled = enabled;
      try {
        await invoke("show_output_set_enabled", { universe, enabled });
      } catch (err: unknown) {
        if (target.enabled === enabled) {
          target.enabled = previous;
        }
        useToastStore().addToast(
          `Failed to update universe state: ${String(err)}`,
          ToastType.ERROR,
        );
      }
    },
    async toggleUniverseEnabled(universe: number) {
      const target = this.universes.find((u) => u.universe === universe);
      if (target) {
        await this.setUniverseEnabled(universe, !target.enabled);
      }
    },
    async addUniverse() {
      const maxUniverse = this.universes.reduce((max, curr) => Math.max(max, curr.universe), 0);
      const newUniverse = maxUniverse + 1;
      const newConfig = {
        universe: newUniverse,
        output: "none",
        enabled: true,
      };
      this.universes.push(newConfig);
      try {
        const created = await invoke<number>("show_output_add_universe");
        if (created && created !== newUniverse) {
          newConfig.universe = created;
        }
      } catch (err: unknown) {
        this.universes = this.universes.filter((u) => u.universe !== newUniverse);
        useToastStore().addToast(`Failed to add universe: ${String(err)}`, ToastType.ERROR);
      }
    },
    async removeUniverse(universe: number) {
      if (this.universes.length <= 1) {
        return;
      }
      const previousUniverses = [...this.universes];
      this.universes = this.universes.filter((u) => u.universe !== universe);
      try {
        await invoke("show_output_remove_universe", { universe });
      } catch (err: unknown) {
        this.universes = previousUniverses;
        useToastStore().addToast(`Failed to remove universe: ${String(err)}`, ToastType.ERROR);
      }
    },
  },
});
