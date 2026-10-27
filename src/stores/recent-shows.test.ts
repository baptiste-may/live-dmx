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
        available: true,
      },
    ]);

    await store.load();

    expect(invoke).toHaveBeenCalledWith("get_recent_shows");
    expect(store.shows.length).toBe(1);
    expect(store.shows[0].displayName).toBe("Concert 2026");
  });

  it("should preserve the backend availability flag when loading", async () => {
    const store = useRecentShows();
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
        available: true,
      },
      {
        displayName: "Moved show",
        path: "/shows/moved.ldmx",
        updatedAt: "123457",
        available: false,
      },
    ]);

    await store.load();

    expect(store.shows.map((show) => show.available)).toEqual([true, false]);
  });

  it("should add and persist recent show via backend save_recent_show", async () => {
    const store = useRecentShows();
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
        available: true,
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

  it("should replace the list with the backend result after removal", async () => {
    const store = useRecentShows();
    store.shows = [
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
        available: true,
      },
      {
        displayName: "Moved show",
        path: "/shows/moved.ldmx",
        updatedAt: "123457",
        available: false,
      },
    ];
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        displayName: "Moved show",
        path: "/shows/moved.ldmx",
        updatedAt: "123457",
        available: false,
      },
    ]);

    await store.remove("/shows/concert.ldmx");

    expect(store.shows).toEqual([
      {
        displayName: "Moved show",
        path: "/shows/moved.ldmx",
        updatedAt: "123457",
        available: false,
      },
    ]);
  });

  it("should keep the list unchanged when removal fails", async () => {
    const store = useRecentShows();
    const shows = [
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
        available: true,
      },
    ];
    store.shows = shows;
    vi.mocked(invoke).mockRejectedValueOnce(new Error("disk full"));

    await store.remove("/shows/concert.ldmx");

    expect(store.shows).toEqual(shows);
  });

  it("should clean unavailable shows via backend clean_recent_shows", async () => {
    const store = useRecentShows();
    store.shows = [
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
        available: true,
      },
      {
        displayName: "Moved show",
        path: "/shows/moved.ldmx",
        updatedAt: "123457",
        available: false,
      },
    ];
    vi.mocked(invoke).mockResolvedValueOnce([
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
        available: true,
      },
    ]);

    await store.clean();

    expect(invoke).toHaveBeenCalledWith("clean_recent_shows");
    expect(store.shows).toEqual([
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
        available: true,
      },
    ]);
  });

  it("should keep the list unchanged when cleaning fails", async () => {
    const store = useRecentShows();
    const shows = [
      {
        displayName: "Concert 2026",
        path: "/shows/concert.ldmx",
        updatedAt: "123456",
        available: true,
      },
      {
        displayName: "Moved show",
        path: "/shows/moved.ldmx",
        updatedAt: "123457",
        available: false,
      },
    ];
    store.shows = shows;
    vi.mocked(invoke).mockRejectedValueOnce(new Error("disk full"));

    await store.clean();

    expect(store.shows).toEqual(shows);
  });
});
