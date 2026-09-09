import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { GdtfCatalogItem } from "@/types/fixtures";

import { catalogItemsToManufacturers, useFixtureBrowser } from "./fixture-browser";
import { useSettings } from "./settings.ts";

const mockInvoke = vi.mocked(invoke);

const mockCatalogItems: GdtfCatalogItem[] = [
  {
    fixture: "SlimPAR 64",
    manufacturer: "Chauvet DJ",
    isInstalled: true,
    isGeneric: false,
    modes: [{ name: "Standard", dmxfootprint: 4, channels: ["Dimmer", "Red", "Green", "Blue"] }],
  },
  {
    fixture: "GigBAR 2",
    manufacturer: "Chauvet DJ",
    isInstalled: true,
    isGeneric: false,
    modes: [{ name: "Standard", dmxfootprint: 4, channels: ["Dimmer", "Red", "Green", "Blue"] }],
  },
  {
    fixture: "Mega Par Profile",
    manufacturer: "American DJ",
    isInstalled: false,
    isGeneric: false,
    rid: 1234,
    modes: [{ name: "Standard", dmxfootprint: 4, channels: ["Dimmer", "Red", "Green", "Blue"] }],
  },
];

describe("fixture-browser store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("should initialize with default state", () => {
    const store = useFixtureBrowser();

    expect(store.manufacturers).toBeUndefined();
    expect(store.searchQuery).toBe("");
    expect(store.filteredManufacturers).toEqual({});
    expect(store.isLoading).toBe(false);
    expect(store.isInstalled).toBe(true);
    expect(store.isAuthenticated).toBe(false);
  });

  it("should convert catalog items to manufacturers correctly", () => {
    const manufacturers = catalogItemsToManufacturers(mockCatalogItems);

    expect(Object.keys(manufacturers)).toContain("chauvet-dj");
    expect(Object.keys(manufacturers)).toContain("american-dj");
    expect(manufacturers["chauvet-dj"].fixtures["slimpar-64"].name).toBe("SlimPAR 64");
    expect(manufacturers["american-dj"].fixtures["mega-par-profile"].rid).toBe(1234);
  });

  it("should propagate real categories when present and fall back otherwise", () => {
    const items: GdtfCatalogItem[] = [
      {
        fixture: "Spider",
        manufacturer: "Robe",
        isInstalled: true,
        isGeneric: false,
        categories: ["Moving Head", "Spot"],
        modes: [{ name: "Standard", dmxfootprint: 24 }],
      },
      {
        fixture: "Dimmer",
        manufacturer: "Generic",
        isInstalled: true,
        isGeneric: true,
        modes: [{ name: "1ch", dmxfootprint: 1 }],
      },
      {
        fixture: "Unknown Online",
        manufacturer: "Vendor",
        isInstalled: false,
        isGeneric: false,
        modes: [{ name: "Basic", dmxfootprint: 2 }],
      },
    ];

    const manufacturers = catalogItemsToManufacturers(items);

    expect(manufacturers["robe"].fixtures["spider"].categories).toEqual(["Moving Head", "Spot"]);
    expect(manufacturers["generic"].fixtures["dimmer"].categories).toEqual(["Generic"]);
    expect(manufacturers["vendor"].fixtures["unknown-online"].categories).toEqual(["GDTF"]);
  });

  it("should fetch fixtures and perform search via gdtf_search", async () => {
    mockInvoke.mockResolvedValue(mockCatalogItems);

    const store = useFixtureBrowser();
    await store.loadAll();

    expect(mockInvoke).toHaveBeenCalledWith("gdtf_search", { query: undefined });
    expect(store.isLoading).toBe(false);
    expect(store.manufacturers).toBeDefined();
    expect(store.manufacturers?.["chauvet-dj"].fixtures).toHaveProperty("slimpar-64");

    await store.search("SlimPAR");
    expect(store.searchQuery).toBe("SlimPAR");
    expect(mockInvoke).toHaveBeenCalledWith("gdtf_search", { query: "SlimPAR" });
  });

  it("should clear search query when clearSearch is called", async () => {
    mockInvoke.mockResolvedValue(mockCatalogItems);

    const store = useFixtureBrowser();
    await store.search("gig");
    expect(store.searchQuery).toBe("gig");

    store.clearSearch();
    expect(store.searchQuery).toBe("");
    expect(mockInvoke).toHaveBeenCalledWith("gdtf_search", { query: undefined });
  });

  it("should reload fixtures when refresh is called", async () => {
    mockInvoke.mockResolvedValue(mockCatalogItems);

    const store = useFixtureBrowser();
    await store.refresh();

    expect(mockInvoke).toHaveBeenCalledWith("gdtf_search", { query: undefined });
    expect(store.isLoading).toBe(false);
  });

  it("should handle error when loadAll fails", async () => {
    mockInvoke.mockRejectedValue(new Error("Search failure"));

    const store = useFixtureBrowser();
    const res = await store.loadAll();

    expect(res).toBe(false);
    expect(store.isLoading).toBe(false);
  });

  it("should get fixture by manufacturer key and fixture key", async () => {
    mockInvoke.mockResolvedValue(mockCatalogItems);

    const store = useFixtureBrowser();
    await store.loadAll();

    const fixture = store.getFixture("chauvet-dj", "slimpar-64");
    expect(fixture).not.toBeNull();
    expect(fixture?.name).toBe("SlimPAR 64");
  });

  it("should check auth status from backend", async () => {
    mockInvoke.mockResolvedValue(true);

    const store = useFixtureBrowser();
    await store.checkAuthStatus();

    expect(mockInvoke).toHaveBeenCalledWith("gdtf_get_credentials_status");
    expect(store.isAuthenticated).toBe(true);
  });

  it("should login with credentials via gdtf_set_credentials", async () => {
    mockInvoke.mockResolvedValue(true);

    const store = useFixtureBrowser();
    const success = await store.setCredentials("user@example.com", "secret");

    expect(mockInvoke).toHaveBeenCalledWith("gdtf_set_credentials", {
      user: "user@example.com",
      pass: "secret",
    });
    expect(success).toBe(true);
    expect(store.isAuthenticated).toBe(true);
  });

  it("should fetch mode on demand via gdtf_fetch_mode", async () => {
    const mockParsedMode = {
      name: "Standard",
      shortName: "Std",
      channelCount: 4,
      channels: ["Dimmer", "Red", "Green", "Blue"],
    };
    mockInvoke.mockResolvedValue(mockParsedMode);

    const store = useFixtureBrowser();
    const fixture = {
      name: "SlimPAR 64",
      manufacturer: "Chauvet DJ",
      manufacturerKey: "chauvet-dj",
      fixtureKey: "slimpar-64",
      rid: 100,
      isInstalled: false,
      modes: [],
    };

    const mode = await store.fetchMode(fixture, "Standard");

    expect(mockInvoke).toHaveBeenCalledWith("gdtf_fetch_mode", {
      rid: 100,
      manufacturer: "Chauvet DJ",
      fixture: "SlimPAR 64",
      modeName: "Standard",
      persist: true,
    });
    expect(mode).toEqual(mockParsedMode);
    expect(fixture.isInstalled).toBe(true);
  });

  it("should fetch mode for preview without persisting or setting isInstalled", async () => {
    const mockParsedMode = {
      name: "Standard",
      shortName: "Std",
      channelCount: 4,
      channels: ["Dimmer", "Red", "Green", "Blue"],
    };
    mockInvoke.mockResolvedValue(mockParsedMode);

    const store = useFixtureBrowser();
    const fixture = {
      name: "SlimPAR 64",
      manufacturer: "Chauvet DJ",
      manufacturerKey: "chauvet-dj",
      fixtureKey: "slimpar-64",
      rid: 100,
      isInstalled: false,
      modes: [{ name: "Standard", channels: [] }],
    };

    const mode = await store.fetchMode(fixture, "Standard", { persist: false });

    expect(mockInvoke).toHaveBeenCalledWith("gdtf_fetch_mode", {
      rid: 100,
      manufacturer: "Chauvet DJ",
      fixture: "SlimPAR 64",
      modeName: "Standard",
      persist: false,
    });
    expect(mode).toEqual(mockParsedMode);
    expect(fixture.isInstalled).toBe(false);
    expect(fixture.modes[0].channels).toEqual(["Dimmer", "Red", "Green", "Blue"]);
  });

  it("should import GDTF file via gdtf_import_file", async () => {
    const importedItem: GdtfCatalogItem = {
      fixture: "Imported Spot",
      manufacturer: "Custom",
      isInstalled: true,
      isGeneric: false,
      modes: [{ name: "Default", dmxfootprint: 5 }],
    };
    mockInvoke.mockImplementation(async (cmd) => {
      if (cmd === "gdtf_import_file") return importedItem;
      if (cmd === "gdtf_search") return [importedItem];
      return {};
    });

    const store = useFixtureBrowser();
    const success = await store.importGdtf("/path/to/fixture.gdtf");

    expect(mockInvoke).toHaveBeenCalledWith("gdtf_import_file", { path: "/path/to/fixture.gdtf" });
    expect(success).toBe(true);
  });

  it("should open settings modal if unauthenticated when syncing catalog", async () => {
    const store = useFixtureBrowser();
    const settings = useSettings();
    store.isAuthenticated = false;
    settings.isModalOpen = false;
    const success = await store.syncCatalog();

    expect(settings.isModalOpen).toBe(true);
    expect(success).toBe(false);
  });

  it("should sync online catalog via gdtf_sync_catalog when authenticated", async () => {
    mockInvoke.mockImplementation(async (cmd) => {
      if (cmd === "gdtf_sync_catalog") return 42;
      if (cmd === "gdtf_search") return mockCatalogItems;
      return {};
    });

    const store = useFixtureBrowser();
    store.isAuthenticated = true;
    const success = await store.syncCatalog();

    expect(mockInvoke).toHaveBeenCalledWith("gdtf_sync_catalog");
    expect(success).toBe(true);
  });
});
