import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { EVERY_PAGES, NAVIGATION_SECTIONS, PageId, useNavigation } from "./navigation";

describe("navigation store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should default to patch page", () => {
    const store = useNavigation();

    expect(store.currentPage).toBe(PageId.PATCH);
    expect(store.getCurrentPage()?.name).toBe("Patch");
  });

  it("should navigate to target page id", () => {
    const store = useNavigation();

    store.navigate(PageId.INTENSITY);
    expect(store.currentPage).toBe(PageId.INTENSITY);
    expect(store.getCurrentPage()?.name).toBe("Intensity");

    store.navigate(PageId.COLORS);
    expect(store.currentPage).toBe(PageId.COLORS);
    expect(store.getCurrentPage()?.name).toBe("Colors");
  });

  it("should define Setup and Live navigation sections", () => {
    expect(NAVIGATION_SECTIONS).toHaveLength(2);
    expect(NAVIGATION_SECTIONS[0]?.title).toBe("Setup");
    expect(NAVIGATION_SECTIONS[0]?.pages.map((p) => p.id)).toEqual([
      PageId.INPUT,
      PageId.OUTPUT,
      PageId.PATCH,
      PageId.CONFIG,
    ]);
    expect(NAVIGATION_SECTIONS[1]?.title).toBe("Live");
    expect(NAVIGATION_SECTIONS[1]?.pages.map((p) => p.id)).toEqual([
      PageId.INTENSITY,
      PageId.COLORS,
    ]);
  });

  it("should contain all defined views across sections", () => {
    expect(EVERY_PAGES).toHaveLength(6);
    expect(EVERY_PAGES.map((p) => p.id)).toEqual([
      PageId.INPUT,
      PageId.OUTPUT,
      PageId.PATCH,
      PageId.CONFIG,
      PageId.INTENSITY,
      PageId.COLORS,
    ]);
  });
});
