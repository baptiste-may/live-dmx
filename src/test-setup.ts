import "@testing-library/jest-dom";

import { createTestingPinia } from "@pinia/testing";
import { cleanup } from "@testing-library/vue";
import { config } from "@vue/test-utils";
import { afterEach, beforeEach, vi } from "vitest";

const mockWindow = {
  close: vi.fn(),
  minimize: vi.fn(),
  maximize: vi.fn(),
  toggleMaximize: vi.fn(),
  hide: vi.fn(),
  show: vi.fn(),
};

// Global Tauri API mocks for Vitest
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({}),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => mockWindow,
}));

vi.mock("@tauri-apps/plugin-global-shortcut", () => ({
  register: vi.fn().mockResolvedValue(undefined),
  unregister: vi.fn().mockResolvedValue(undefined),
  unregisterAll: vi.fn().mockResolvedValue(undefined),
  isRegistered: vi.fn().mockResolvedValue(false),
}));

// Global Testing Pinia plugin for Vue Testing Library / Vue Test Utils
beforeEach(() => {
  config.global.plugins = [createTestingPinia({ stubActions: false })];
});

// Automatically cleanup DOM after each test
afterEach(() => {
  cleanup();
});
