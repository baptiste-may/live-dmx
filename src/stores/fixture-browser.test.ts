import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { mockFixturesData } from "./__mocks__/fixture-browser";
import { useFixtureBrowser } from "./fixture-browser";

const mockInvoke = vi.mocked(invoke);

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
    expect(store.isLoading).toBe(true);
  });

  it("should fetch manufacturers and perform search filtering", async () => {
    mockInvoke.mockResolvedValue(mockFixturesData);

    const store = useFixtureBrowser();
    await store.loadAll();

    expect(store.isLoading).toBe(false);
    expect(store.manufacturers).toEqual(mockFixturesData);

    await store.search("SlimPAR");
    expect(store.searchQuery).toBe("SlimPAR");
    expect(Object.keys(store.filteredManufacturers)).toEqual(["chauvet"]);
    expect(store.filteredManufacturers.chauvet.fixtures).toHaveProperty("slimpar-64");

    // Multi-token search
    await store.search("American mega");
    expect(Object.keys(store.filteredManufacturers)).toEqual(["adj"]);
    expect(store.filteredManufacturers.adj.fixtures).toHaveProperty("mega-par-profile");
  });

  it("should clear search query when clearSearch is called", async () => {
    mockInvoke.mockResolvedValue(mockFixturesData);

    const store = useFixtureBrowser();
    await store.loadAll();

    await store.search("gig");
    expect(store.searchQuery).toBe("gig");
    expect(Object.keys(store.filteredManufacturers.chauvet.fixtures)).toEqual(["gig-bar-2"]);

    store.clearSearch();
    expect(store.searchQuery).toBe("");
    expect(store.filteredManufacturers).toEqual(mockFixturesData);
  });

  it("should reload fixtures when refresh is called", async () => {
    mockInvoke.mockImplementation(async (cmd) => {
      if (cmd === "reload_fixtures") return mockFixturesData;
      return {};
    });

    const store = useFixtureBrowser();
    await store.refresh();

    expect(mockInvoke).toHaveBeenCalledWith("reload_fixtures");
    expect(store.manufacturers).toEqual(mockFixturesData);
    expect(store.isLoading).toBe(false);
  });

  it("should handle error when refresh fails", async () => {
    mockInvoke.mockRejectedValue(new Error("Reload failure"));

    const store = useFixtureBrowser();
    await store.refresh();

    expect(store.isLoading).toBe(false);
  });

  it("should get fixture by manufacturer key and fixture key", async () => {
    mockInvoke.mockResolvedValue(mockFixturesData);

    const store = useFixtureBrowser();
    await store.search();

    const fixture = store.getFixture("chauvet", "slimpar-64");
    expect(fixture).not.toBeNull();
    expect(fixture?.name).toBe("SlimPAR 64");
  });

  it("should check if fixtures are installed", async () => {
    mockInvoke.mockResolvedValue(true);
    const store = useFixtureBrowser();

    const result = await store.checkInstalled();
    expect(result).toBe(true);
    expect(store.isInstalled).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("check_fixtures_installed");
  });

  it("should initialize by searching if fixtures are installed", async () => {
    mockInvoke.mockImplementation(async (cmd) => {
      if (cmd === "check_fixtures_installed") return true;
      if (cmd === "get_fixtures") return mockFixturesData;
      return {};
    });

    const store = useFixtureBrowser();
    await store.init();

    expect(store.isInstalled).toBe(true);
    expect(store.isLoading).toBe(false);
    expect(store.manufacturers).toEqual(mockFixturesData);
  });

  it("should initialize by setting isLoading false if fixtures are not installed", async () => {
    mockInvoke.mockImplementation(async (cmd) => {
      if (cmd === "check_fixtures_installed") return false;
      return {};
    });

    const store = useFixtureBrowser();
    await store.init();

    expect(store.isInstalled).toBe(false);
    expect(store.isLoading).toBe(false);
    expect(store.manufacturers).toBeUndefined();
  });

  it("should download fixtures and then search", async () => {
    mockInvoke.mockImplementation(async (cmd) => {
      if (cmd === "download_fixtures") return undefined;
      if (cmd === "get_fixtures") return mockFixturesData;
      return {};
    });

    const store = useFixtureBrowser();
    await store.downloadFixtures();

    expect(mockInvoke).toHaveBeenCalledWith("download_fixtures");
    expect(store.isInstalled).toBe(true);
    expect(store.isDownloading).toBe(false);
    expect(store.manufacturers).toEqual(mockFixturesData);
  });

  it("should handle error when downloadFixtures fails", async () => {
    mockInvoke.mockRejectedValue(new Error("Network connection error"));

    const store = useFixtureBrowser();
    await store.downloadFixtures();

    expect(store.isDownloading).toBe(false);
  });
});
