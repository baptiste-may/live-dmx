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

  it("should fetch manufacturers and perform search filtering via backend", async () => {
    mockInvoke.mockResolvedValue({
      chauvet: {
        name: "Chauvet DJ",
        fixtures: {
          "slimpar-64": mockFixturesData.chauvet.fixtures["slimpar-64"],
        },
      },
    });

    const store = useFixtureBrowser();
    await store.search("SlimPAR");

    expect(store.isLoading).toBe(false);
    expect(store.searchQuery).toBe("SlimPAR");
    expect(mockInvoke).toHaveBeenCalledWith("get_fixtures", { query: "SlimPAR" });
    expect(Object.keys(store.filteredManufacturers)).toEqual(["chauvet"]);
    expect(store.filteredManufacturers.chauvet.fixtures).toHaveProperty("slimpar-64");
  });

  it("should clear search query when clearSearch is called", async () => {
    mockInvoke.mockResolvedValue(mockFixturesData);

    const store = useFixtureBrowser();
    await store.search("gig");
    expect(store.searchQuery).toBe("gig");
    expect(mockInvoke).toHaveBeenCalledWith("get_fixtures", { query: "gig" });

    await store.clearSearch();
    expect(store.searchQuery).toBe("");
    expect(mockInvoke).toHaveBeenCalledWith("get_fixtures", { query: undefined });
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
