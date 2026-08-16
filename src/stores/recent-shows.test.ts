import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useRecentShows } from "./recent-shows";

describe("recent-shows store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("should initialize with empty shows", () => {
    const store = useRecentShows();
    expect(store.shows).toEqual([]);
  });

  it("should load shows from backend get_recent_shows command", async () => {
    const store = useRecentShows();
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
      },
    ]);

    await store.load();

    expect(invoke).toHaveBeenCalledWith("get_recent_shows");
    expect(store.shows.length).toBe(1);
    expect(store.shows[0].displayName).toBe("Concert 2026");
  });

  it("should add and persist recent show via backend save_recent_show", async () => {
    const store = useRecentShows();
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
      },
    ]);

    await store.addOrUpdate("Concert 2026", "/shows/concert.ldmx");

    expect(invoke).toHaveBeenCalledWith("save_recent_show", {
      displayName: "Concert 2026",
      path: "/shows/concert.ldmx",
    });
    expect(store.shows.length).toBe(1);
    expect(store.shows[0].displayName).toBe("Concert 2026");
  });

  it("should remove show by path via backend remove_recent_show", async () => {
    const store = useRecentShows();
    vi.mocked(invoke).mockResolvedValueOnce([]);

    await store.remove("/shows/show1.ldmx");

    expect(invoke).toHaveBeenCalledWith("remove_recent_show", {
      path: "/shows/show1.ldmx",
    });
    expect(store.shows.length).toBe(0);
  });
});
