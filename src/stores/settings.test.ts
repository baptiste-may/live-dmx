import { beforeEach, describe, expect, it, vi } from "vitest";

import { SETTINGS_STORAGE_KEY, useSettings } from "./settings.ts";

describe("settings store", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
    vi.clearAllMocks();
  });

  it("initializes with default values", () => {
    const settings = useSettings();

    expect(settings.isModalOpen).toBe(false);
    expect(settings.theme).toBe("system");
    expect(settings.scale).toBe(100);
    expect(settings.live.showButtonLabels).toBe(true);
    expect(settings.live.showSectionTitles).toBe(true);
  });

  it("updates scale, persists to localStorage, and sets documentElement zoom", () => {
    const settings = useSettings();

    settings.setScale(120);
    expect(settings.scale).toBe(120);
    expect(document.documentElement.style.zoom).toBe("1.2");

    const saved = JSON.parse(window.localStorage.getItem(SETTINGS_STORAGE_KEY) || "{}");
    expect(saved.scale).toBe(120);

    // Clamps between min (50) and max (150)
    settings.setScale(20);
    expect(settings.scale).toBe(50);
    expect(document.documentElement.style.zoom).toBe("0.5");

    settings.setScale(200);
    expect(settings.scale).toBe(150);
    expect(document.documentElement.style.zoom).toBe("1.5");
  });

  it("opens and closes the settings modal", () => {
    const settings = useSettings();

    settings.openModal();
    expect(settings.isModalOpen).toBe(true);

    settings.closeModal();
    expect(settings.isModalOpen).toBe(false);
  });

  it("updates theme and applies data-theme attribute on documentElement", () => {
    const settings = useSettings();

    settings.setTheme("dark");
    expect(settings.theme).toBe("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("forest");

    settings.setTheme("light");
    expect(settings.theme).toBe("light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("cupcake");

    const saved = JSON.parse(window.localStorage.getItem(SETTINGS_STORAGE_KEY) || "{}");
    expect(saved.theme).toBe("light");
  });

  it("updates live settings and persists to localStorage", () => {
    const settings = useSettings();

    settings.setShowButtonLabels(false);
    expect(settings.live.showButtonLabels).toBe(false);

    settings.setShowSectionTitles(false);
    expect(settings.live.showSectionTitles).toBe(false);

    const saved = JSON.parse(window.localStorage.getItem(SETTINGS_STORAGE_KEY) || "{}");
    expect(saved.live.showButtonLabels).toBe(false);
    expect(saved.live.showSectionTitles).toBe(false);
  });

  it("resolves system theme based on matchMedia", () => {
    const originalMatchMedia = window.matchMedia;

    window.matchMedia = vi.fn().mockImplementation((query: string) => ({
      matches: query.includes("dark"),
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));

    const settings = useSettings();
    settings.setTheme("system");

    expect(settings.resolvedTheme).toBe("forest");
    expect(document.documentElement.getAttribute("data-theme")).toBe("forest");

    window.matchMedia = originalMatchMedia;
  });

  it("initializes theme listener without errors", () => {
    const originalMatchMedia = window.matchMedia;
    const addEventListenerMock = vi.fn();

    window.matchMedia = vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: addEventListenerMock,
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }));

    const settings = useSettings();
    settings.setTheme("system");
    settings.initTheme();

    expect(document.documentElement.getAttribute("data-theme")).toBe("cupcake");

    window.matchMedia = originalMatchMedia;
  });
});
