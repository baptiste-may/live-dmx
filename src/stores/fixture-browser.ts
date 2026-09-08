import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { markRaw } from "vue";

import type {
  FixtureMode,
  FixtureType,
  GdtfCatalogItem,
  Manufacturers,
  ParsedGdtfMode,
} from "@/types/fixtures";

import { useSettings } from "./settings.ts";
import { ToastType, useToastStore } from "./toast.ts";

export function catalogItemsToManufacturers(items: GdtfCatalogItem[]): Manufacturers {
  const result: Manufacturers = {};

  for (const item of items) {
    const manKey = (item.manufacturer || "Generic").toLowerCase().replace(/[^a-z0-9_-]/g, "-");
    let fixKey = (item.fixture || "Fixture").toLowerCase().replace(/[^a-z0-9_-]/g, "-");

    if (!result[manKey]) {
      result[manKey] = {
        name: item.manufacturer,
        fixtures: {},
      };
    }

    const baseFixKey = fixKey;
    let collisionCounter = 1;
    while (result[manKey].fixtures[fixKey]) {
      const discriminator = item.rid ?? (item.uuid ? item.uuid.slice(0, 8) : collisionCounter);
      fixKey = `${baseFixKey}-${discriminator}`;
      collisionCounter++;
    }

    const modes: FixtureMode[] = (item.modes || []).map((m) => ({
      name: m.name,
      channels:
        m.channels && m.channels.length > 0
          ? m.channels
          : Array.from({ length: m.dmxfootprint }, (_, i) => `Channel ${i + 1}`),
      dmxfootprint: m.dmxfootprint,
    }));

    result[manKey].fixtures[fixKey] = {
      name: item.fixture,
      manufacturer: item.manufacturer,
      manufacturerKey: manKey,
      fixtureKey: fixKey,
      modes,
      rid: item.rid,
      isInstalled: item.isInstalled,
      isGeneric: item.isGeneric,
      uploader: item.uploader,
      rating: item.rating,
      revision: item.revision,
      categories: item.isGeneric ? ["Generic"] : ["GDTF"],
    };
  }

  return result;
}

export type FixtureBrowserState = {
  manufacturers: Manufacturers | undefined;
  searchQuery: string;
  filteredManufacturers: Manufacturers;
  isLoading: boolean;
  isInstalled: boolean | null;
  isFetchingMode: boolean;
  isAuthenticated: boolean;
  searchId: number;
};

export const useFixtureBrowser = defineStore("fixture-browser", {
  state: (): FixtureBrowserState => ({
    manufacturers: undefined,
    searchQuery: "",
    filteredManufacturers: markRaw({}),
    isLoading: false,
    isInstalled: true,
    isFetchingMode: false,
    isAuthenticated: false,
    searchId: 0,
  }),
  actions: {
    async checkAuthStatus() {
      try {
        this.isAuthenticated = await invoke<boolean>("gdtf_get_credentials_status");
      } catch {
        this.isAuthenticated = false;
      }
    },
    async setCredentials(user: string, pass: string): Promise<boolean> {
      try {
        await invoke<boolean>("gdtf_set_credentials", { user, pass });
        this.isAuthenticated = true;
        useToastStore().addToast("Connected to GDTF-Share successfully!", ToastType.SUCCESS);
        return true;
      } catch (err) {
        useToastStore().addToast(`GDTF-Share login failed: ${String(err)}`, ToastType.ERROR);
        return false;
      }
    },
    async logout(): Promise<void> {
      try {
        await invoke("gdtf_logout");
        this.isAuthenticated = false;
        useToastStore().addToast("Disconnected from GDTF-Share", ToastType.INFO);
      } catch {
        this.isAuthenticated = false;
      }
    },
    async checkInstalled(): Promise<boolean> {
      try {
        this.isInstalled = await invoke<boolean>("check_fixtures_installed");
        return this.isInstalled;
      } catch {
        this.isInstalled = true;
        return true;
      }
    },
    async init() {
      await this.checkAuthStatus();
      await this.checkInstalled();
      if (this.manufacturers === undefined) {
        await this.loadAll();
      }
    },
    async loadAll(): Promise<boolean> {
      const currentSearchId = ++this.searchId;
      this.isLoading = true;

      try {
        const raw = await invoke<GdtfCatalogItem[]>("gdtf_search", {
          query: this.searchQuery.trim() ? this.searchQuery.trim() : undefined,
        });

        if (currentSearchId !== this.searchId) {
          return false;
        }

        const grouped = catalogItemsToManufacturers(raw);
        this.manufacturers = markRaw(grouped);
        this.filteredManufacturers = markRaw(grouped);
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
    async search(query = "") {
      this.searchQuery = query;
      await this.loadAll();
    },
    clearSearch() {
      this.searchQuery = "";
      this.loadAll();
    },
    async refresh(): Promise<boolean> {
      try {
        await invoke("reload_fixtures");
      } catch {
        // Tolerated in test / non-Tauri environments
      }
      return this.loadAll();
    },
    async syncCatalog(): Promise<boolean> {
      if (!this.isAuthenticated) {
        useSettings().openModal();
        useToastStore().addToast(
          "Please connect your free GDTF-Share account in Settings to sync the online catalog.",
          ToastType.INFO,
        );
        return false;
      }

      this.isLoading = true;
      try {
        const count = await invoke<number>("gdtf_sync_catalog");
        useToastStore().addToast(`Synced ${count} fixtures from GDTF-Share!`, ToastType.SUCCESS);
        await this.loadAll();
        return true;
      } catch (err) {
        const errorMsg = String(err);
        if (errorMsg.includes("authentication required") || errorMsg.includes("401")) {
          this.isAuthenticated = false;
          useSettings().openModal();
        }
        useToastStore().addToast(`Failed to sync GDTF catalog: ${errorMsg}`, ToastType.ERROR);
        return false;
      } finally {
        this.isLoading = false;
      }
    },
    getFixture(manufacturerKey: string, fixtureKey: string): FixtureType | null {
      const source = this.filteredManufacturers?.[manufacturerKey]
        ? this.filteredManufacturers
        : this.manufacturers;
      if (!source) return null;
      const manufacturer = source[manufacturerKey];
      if (!manufacturer) return null;
      return manufacturer.fixtures[fixtureKey] || null;
    },
    async fetchMode(
      fixture: FixtureType,
      modeName: string,
      options: { persist?: boolean } = {},
    ): Promise<ParsedGdtfMode> {
      const shouldPersist = options.persist ?? true;
      this.isFetchingMode = true;
      try {
        const result = await invoke<ParsedGdtfMode>("gdtf_fetch_mode", {
          rid: fixture.rid,
          manufacturer: fixture.manufacturer || fixture.name,
          fixture: fixture.name,
          modeName,
          persist: shouldPersist,
        });
        if (shouldPersist) {
          fixture.isInstalled = true;
        }
        const targetMode = fixture.modes.find(
          (m) => m.name.trim().toLowerCase() === modeName.trim().toLowerCase(),
        );
        if (targetMode) {
          targetMode.channels = result.channels;
          targetMode.dmxfootprint = result.channelCount;
        }
        if (this.manufacturers) {
          this.manufacturers = { ...this.manufacturers };
        }
        if (this.filteredManufacturers) {
          this.filteredManufacturers = { ...this.filteredManufacturers };
        }
        return result;
      } catch (err) {
        const errorMsg = String(err);
        if (
          shouldPersist &&
          (errorMsg.includes("authentication required") || errorMsg.includes("401"))
        ) {
          this.isAuthenticated = false;
          useSettings().openModal();
        }
        throw err;
      } finally {
        this.isFetchingMode = false;
      }
    },
    async importGdtf(filePath: string): Promise<boolean> {
      try {
        const item = await invoke<GdtfCatalogItem>("gdtf_import_file", { path: filePath });
        useToastStore().addToast(
          `Imported GDTF fixture: ${item.fixture} (${item.manufacturer})`,
          ToastType.SUCCESS,
        );
        await this.loadAll();
        return true;
      } catch (err) {
        useToastStore().addToast(`Failed to import GDTF: ${String(err)}`, ToastType.ERROR);
        return false;
      }
    },
  },
});
