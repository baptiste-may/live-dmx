import { defineStore } from "pinia";

export type ThemeMode = "dark" | "light" | "system";

export interface LiveSettings {
  showButtonLabels: boolean;
  showSectionTitles: boolean;
}

export interface SettingsState {
  isModalOpen: boolean;
  theme: ThemeMode;
  scale: number;
  live: LiveSettings;
}

export const SETTINGS_STORAGE_KEY = "live-dmx-settings";
export const DEFAULT_UI_SCALE = 100;
export const MIN_UI_SCALE = 50;
export const MAX_UI_SCALE = 150;

function getStoredSettings(): {
  theme: ThemeMode;
  scale: number;
  live: LiveSettings;
} {
  const defaults: { theme: ThemeMode; scale: number; live: LiveSettings } = {
    theme: "system",
    scale: DEFAULT_UI_SCALE,
    live: {
      showButtonLabels: true,
      showSectionTitles: true,
    },
  };

  try {
    if (typeof window !== "undefined" && window.localStorage) {
      const stored = window.localStorage.getItem(SETTINGS_STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored);
        const theme =
          parsed.theme === "dark" || parsed.theme === "light" || parsed.theme === "system"
            ? parsed.theme
            : defaults.theme;

        const scale =
          typeof parsed.scale === "number" &&
          parsed.scale >= MIN_UI_SCALE &&
          parsed.scale <= MAX_UI_SCALE
            ? parsed.scale
            : defaults.scale;

        const live: LiveSettings = {
          showButtonLabels:
            typeof parsed.live?.showButtonLabels === "boolean"
              ? parsed.live.showButtonLabels
              : defaults.live.showButtonLabels,
          showSectionTitles:
            typeof parsed.live?.showSectionTitles === "boolean"
              ? parsed.live.showSectionTitles
              : defaults.live.showSectionTitles,
        };

        return { theme, scale, live };
      }
    }
  } catch {
    // Ignore JSON parse or storage read errors
  }

  return defaults;
}

function persistSettings(theme: ThemeMode, scale: number, live: LiveSettings) {
  try {
    if (typeof window !== "undefined" && window.localStorage) {
      window.localStorage.setItem(
        SETTINGS_STORAGE_KEY,
        JSON.stringify({
          theme,
          scale,
          live,
        }),
      );
    }
  } catch {
    // Ignore storage write errors
  }
}

function resolveSystemTheme(): "forest" | "cupcake" {
  if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "forest" : "cupcake";
  }
  return "forest";
}

let mediaQueryListenerAttached = false;

export const useSettings = defineStore("settings", {
  state: (): SettingsState => {
    const stored = getStoredSettings();
    return {
      isModalOpen: false,
      theme: stored.theme,
      scale: stored.scale,
      live: stored.live,
    };
  },
  getters: {
    resolvedTheme(state): "forest" | "cupcake" {
      if (state.theme === "system") {
        return resolveSystemTheme();
      }
      return state.theme === "dark" ? "forest" : "cupcake";
    },
  },
  actions: {
    openModal() {
      this.isModalOpen = true;
    },
    closeModal() {
      this.isModalOpen = false;
    },
    setTheme(theme: ThemeMode) {
      this.theme = theme;
      persistSettings(this.theme, this.scale, this.live);
      this.applyTheme();
    },
    setScale(scale: number) {
      const num = Number(scale);
      this.scale = Math.min(
        Math.max(Number.isFinite(num) ? num : DEFAULT_UI_SCALE, MIN_UI_SCALE),
        MAX_UI_SCALE,
      );
      persistSettings(this.theme, this.scale, this.live);
      this.applyScale();
    },
    setShowButtonLabels(show: boolean) {
      this.live.showButtonLabels = Boolean(show);
      persistSettings(this.theme, this.scale, this.live);
    },
    setShowSectionTitles(show: boolean) {
      this.live.showSectionTitles = Boolean(show);
      persistSettings(this.theme, this.scale, this.live);
    },
    applyTheme() {
      if (typeof document !== "undefined" && document.documentElement) {
        document.documentElement.setAttribute("data-theme", this.resolvedTheme);
      }
    },
    applyScale() {
      if (typeof document !== "undefined" && document.documentElement) {
        document.documentElement.style.zoom = `${this.scale / 100}`;
      }
    },
    initTheme() {
      this.applyTheme();
      this.applyScale();

      if (
        typeof window !== "undefined" &&
        typeof window.matchMedia === "function" &&
        !mediaQueryListenerAttached
      ) {
        const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
        const handleChange = () => {
          if (this.theme === "system") {
            this.applyTheme();
          }
        };

        if (typeof mediaQuery.addEventListener === "function") {
          mediaQuery.addEventListener("change", handleChange);
        } else if (
          "addListener" in mediaQuery &&
          typeof (mediaQuery as { addListener: (cb: () => void) => void }).addListener ===
            "function"
        ) {
          (mediaQuery as { addListener: (cb: () => void) => void }).addListener(handleChange);
        }
        mediaQueryListenerAttached = true;
      }
    },
  },
});
