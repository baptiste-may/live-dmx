import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { EVERY_PAGES, PageId, useNavigation } from "./navigation";

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

    store.navigate(PageId.LIVE);
    expect(store.currentPage).toBe(PageId.LIVE);
    expect(store.getCurrentPage()?.name).toBe("Live");
  });

  it("should contain all defined views", () => {
    expect(EVERY_PAGES).toHaveLength(5);
    expect(EVERY_PAGES.map((p) => p.id)).toEqual([
      PageId.INPUT,
      PageId.OUTPUT,
      PageId.PATCH,
      PageId.CONFIG,
      PageId.LIVE,
    ]);
  });
});
