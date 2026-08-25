import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { EVERY_LIVE_PAGES, LivePageId, useLiveNavigation } from "./live-navigation";

describe("live-navigation store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with default intensity page", () => {
    const store = useLiveNavigation();
    expect(store.currentPageId).toBe(LivePageId.INTENSITY);
    expect(store.currentPage).toBe(EVERY_LIVE_PAGES[LivePageId.INTENSITY]);
  });

  it("should navigate to colors page", () => {
    const store = useLiveNavigation();
    store.navigate(LivePageId.COLORS);

    expect(store.currentPageId).toBe(LivePageId.COLORS);
    expect(store.currentPage).toBe(EVERY_LIVE_PAGES[LivePageId.COLORS]);
  });
});
