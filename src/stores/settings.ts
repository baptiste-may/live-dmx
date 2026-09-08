import { defineStore } from "pinia";

export type ThemeMode = "dark" | "light" | "system";

export interface LiveSettings {
  showButtonLabels: boolean;
  showSectionTitles: boolean;
}

export interface SettingsState {
  isModalOpen: boolean;
  theme: ThemeMode;
  live: LiveSettings;
}

export const SETTINGS_STORAGE_KEY = "live-dmx-settings";

function getStoredSettings(): {
  theme: ThemeMode;
  live: LiveSettings;
} {
  const defaults: { theme: ThemeMode; live: LiveSettings } = {
    theme: "system",
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

        return { theme, live };
      }
    }
  } catch {
    // Ignore JSON parse or storage read errors
  }

  return defaults;
}

function persistSettings(theme: ThemeMode, live: LiveSettings) {
  try {
    if (typeof window !== "undefined" && window.localStorage) {
      window.localStorage.setItem(
        SETTINGS_STORAGE_KEY,
        JSON.stringify({
          theme,
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
      persistSettings(this.theme, this.live);
      this.applyTheme();
    },
    setShowButtonLabels(show: boolean) {
      this.live.showButtonLabels = Boolean(show);
      persistSettings(this.theme, this.live);
    },
    setShowSectionTitles(show: boolean) {
      this.live.showSectionTitles = Boolean(show);
      persistSettings(this.theme, this.live);
    },
    applyTheme() {
      if (typeof document !== "undefined" && document.documentElement) {
        document.documentElement.setAttribute("data-theme", this.resolvedTheme);
      }
    },
    initTheme() {
      this.applyTheme();

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
