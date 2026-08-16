import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { markRaw } from "vue";

import type { Manufacturers } from "@/types/fixtures";

import { ToastType, useToastStore } from "./toast.ts";

export type FixtureLoadProgressPayload = {
  current: number;
  total: number;
  percentage: number;
  manufacturer: string;
};

export async function loadManufacturers(query?: string): Promise<Manufacturers> {
  try {
    const raw = await invoke<Manufacturers>("get_fixtures", {
      query: query && query.trim() ? query.trim() : undefined,
    });
    return markRaw(raw);
  } catch (error: unknown) {
    useToastStore().addToast(`Failed to load fixtures: ${String(error)}`, ToastType.ERROR);
    return {};
  }
}

export type FixtureBrowser = {
  manufacturers: undefined | Manufacturers;
  searchQuery: string;
  filteredManufacturers: Manufacturers;
  isLoading: boolean;
  progress: number;
  loadingStatus: string;
  searchId: number;
};

let unlistenProgress: (() => void) | null = null;

export const useFixtureBrowser = defineStore("fixture-browser", {
  state: (): FixtureBrowser => ({
    manufacturers: undefined,
    searchQuery: "",
    filteredManufacturers: markRaw({}),
    isLoading: true,
    progress: 0,
    loadingStatus: "Loading fixtures...",
    searchId: 0,
  }),
  actions: {
    async initEventListener() {
      if (unlistenProgress) {
        unlistenProgress();
        unlistenProgress = null;
      }
      try {
        unlistenProgress = await listen<FixtureLoadProgressPayload>(
          "fixture-load-progress",
          (event) => {
            this.progress = event.payload.percentage;
            this.loadingStatus = `Loading ${event.payload.manufacturer}...`;
          },
        );
      } catch {
        // Ignored in non-Tauri / test environments
      }
    },
    async search(query?: string) {
      if (query !== undefined) {
        this.searchQuery = query;
      }
      const currentSearchId = ++this.searchId;
      this.isLoading = true;
      this.progress = 0;
      this.loadingStatus = "Loading fixtures...";

      await this.initEventListener();

      try {
        const q = this.searchQuery.trim();
        const results = await loadManufacturers(q);
        if (currentSearchId !== this.searchId) {
          return;
        }
        this.filteredManufacturers = markRaw(results);

        if (this.manufacturers === undefined || !q) {
          this.manufacturers = markRaw(results);
        }
        this.progress = 100;
        this.loadingStatus = "Loaded";
      } finally {
        if (currentSearchId === this.searchId) {
          this.isLoading = false;
        }
      }
    },
    clearSearch() {
      this.searchQuery = "";
      return this.search();
    },
    getFixture(manufacturerKey: string, fixtureKey: string) {
      if (this.manufacturers && this.manufacturers[manufacturerKey]?.fixtures[fixtureKey]) {
        return this.manufacturers[manufacturerKey].fixtures[fixtureKey];
      }
      if (
        this.filteredManufacturers &&
        this.filteredManufacturers[manufacturerKey]?.fixtures[fixtureKey]
      ) {
        return this.filteredManufacturers[manufacturerKey].fixtures[fixtureKey];
      }
      return null;
    },
  },
});
