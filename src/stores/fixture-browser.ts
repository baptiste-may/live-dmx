import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { markRaw } from "vue";

import type { FixtureType, Manufacturers } from "@/types/fixtures";

import { ToastType, useToastStore } from "./toast.ts";

export type FixtureDownloadProgressPayload = {
  percentage: number;
  status: string;
};

export type FixtureLoadProgressPayload = {
  current: number;
  total: number;
  percentage: number;
  manufacturer: string;
};

export async function checkFixturesInstalled(): Promise<boolean> {
  try {
    return await invoke<boolean>("check_fixtures_installed");
  } catch {
    return false;
  }
}

export function filterManufacturersLocally(
  manufacturers: Manufacturers,
  query?: string,
): Manufacturers {
  const trimmed = (query ?? "").trim().toLowerCase();
  if (!trimmed) {
    return manufacturers;
  }
  const tokens = trimmed.split(/\s+/).filter(Boolean);
  if (tokens.length === 0) {
    return manufacturers;
  }

  const result: Manufacturers = {};

  for (const [manId, manufacturer] of Object.entries(manufacturers)) {
    const manNameLower = manufacturer.name.toLowerCase();
    const manIdLower = manId.toLowerCase();

    const matchingFixtures: Record<string, FixtureType> = {};

    for (const [fixId, fixture] of Object.entries(manufacturer.fixtures)) {
      const fixNameLower = fixture.name.toLowerCase();
      const fixIdLower = fixId.toLowerCase();
      const shortNameLower = (fixture.shortName || "").toLowerCase();
      const categories = (fixture.categories || []).map((c) => c.toLowerCase());
      const modeNames = (fixture.modes || []).map((m) => m.name.toLowerCase());

      const allTokensMatch = tokens.every(
        (token) =>
          manNameLower.includes(token) ||
          manIdLower.includes(token) ||
          fixNameLower.includes(token) ||
          fixIdLower.includes(token) ||
          shortNameLower.includes(token) ||
          categories.some((cat) => cat.includes(token)) ||
          modeNames.some((mode) => mode.includes(token)),
      );

      if (allTokensMatch) {
        matchingFixtures[fixId] = fixture;
      }
    }

    if (Object.keys(matchingFixtures).length > 0) {
      result[manId] = {
        ...manufacturer,
        fixtures: matchingFixtures,
      };
    }
  }

  return result;
}

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
  isInstalled: boolean | null;
  isDownloading: boolean;
  progress: number;
  loadingStatus: string;
  searchId: number;
};

let unlistenProgress: (() => void) | null = null;
let unlistenDownloadProgress: (() => void) | null = null;

export const useFixtureBrowser = defineStore("fixture-browser", {
  state: (): FixtureBrowser => ({
    manufacturers: undefined,
    searchQuery: "",
    filteredManufacturers: markRaw({}),
    isLoading: true,
    isInstalled: null,
    isDownloading: false,
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
    async initDownloadEventListener() {
      if (unlistenDownloadProgress) {
        unlistenDownloadProgress();
        unlistenDownloadProgress = null;
      }
      try {
        unlistenDownloadProgress = await listen<FixtureDownloadProgressPayload>(
          "fixture-download-progress",
          (event) => {
            this.progress = event.payload.percentage;
            this.loadingStatus = event.payload.status;
          },
        );
      } catch {
        // Ignored in non-Tauri / test environments
      }
    },
    async checkInstalled() {
      const installed = await checkFixturesInstalled();
      this.isInstalled = installed;
      return installed;
    },
    async init() {
      const installed = await this.checkInstalled();
      if (installed) {
        if (this.manufacturers === undefined) {
          await this.loadAll();
        }
      } else {
        this.isLoading = false;
      }
    },
    async loadAll(forceReload = false): Promise<boolean> {
      const currentSearchId = ++this.searchId;
      this.isLoading = true;
      this.progress = 0;
      this.loadingStatus = forceReload ? "Reloading fixtures..." : "Loading fixtures...";

      await this.initEventListener();

      try {
        const results = forceReload
          ? markRaw(await invoke<Manufacturers>("reload_fixtures"))
          : await loadManufacturers();
        if (currentSearchId !== this.searchId) {
          return false;
        }
        this.manufacturers = markRaw(results);
        this.filteredManufacturers = markRaw(filterManufacturersLocally(results, this.searchQuery));
        this.progress = 100;
        this.loadingStatus = "Loaded";
        return true;
      } catch (error: unknown) {
        useToastStore().addToast(`Failed to load fixtures: ${String(error)}`, ToastType.ERROR);
        return false;
      } finally {
        if (currentSearchId === this.searchId) {
          this.isLoading = false;
        }
      }
    },
    async refresh() {
      const success = await this.loadAll(true);
      if (success) {
        useToastStore().addToast("Fixture library refreshed.", ToastType.SUCCESS);
      }
    },
    async downloadFixtures() {
      this.isDownloading = true;
      this.progress = 0;
      this.loadingStatus = "Starting download...";

      await this.initDownloadEventListener();

      try {
        await invoke("download_fixtures");
        this.isInstalled = true;
        this.isDownloading = false;
        await this.loadAll();
      } catch (error: unknown) {
        this.isDownloading = false;
        useToastStore().addToast(`Failed to download fixtures: ${String(error)}`, ToastType.ERROR);
      }
    },
    async search(query?: string) {
      if (query !== undefined) {
        this.searchQuery = query;
      }
      if (this.manufacturers === undefined) {
        await this.loadAll();
        return;
      }
      this.filteredManufacturers = markRaw(
        filterManufacturersLocally(this.manufacturers, this.searchQuery),
      );
    },
    clearSearch() {
      this.searchQuery = "";
      if (this.manufacturers !== undefined) {
        this.filteredManufacturers = this.manufacturers;
      }
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
