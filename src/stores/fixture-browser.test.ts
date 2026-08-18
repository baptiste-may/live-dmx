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
});
