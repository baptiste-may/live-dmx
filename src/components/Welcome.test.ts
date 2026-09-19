import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Update } from "@tauri-apps/plugin-updater";
import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useApp } from "@/stores/app";
import { useSettings } from "@/stores/settings";
import { useUpdates } from "@/stores/updates";

import Welcome from "./Welcome.vue";

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn().mockResolvedValue("0.2.0"),
  getTauriVersion: vi.fn().mockResolvedValue("2.1.0"),
}));

describe("Welcome.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render welcome screen title and action buttons", () => {
    render(Welcome);

    expect(screen.getByText("Live DMX")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "New show" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Load show" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import from MVR" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Settings" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Exit" })).toBeInTheDocument();
  });

  it("should display the current app version", async () => {
    render(Welcome);

    expect(await screen.findByText("v0.2.0")).toBeInTheDocument();
  });

  it("should call createShow when New show button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    const appStore = useApp();
    await user.click(screen.getByRole("button", { name: "New show" }));

    expect(appStore.createShow).toHaveBeenCalledTimes(1);
  });

  it("should call askLoadShow when Load show button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    const appStore = useApp();
    await user.click(screen.getByRole("button", { name: "Load show" }));

    expect(appStore.askLoadShow).toHaveBeenCalledTimes(1);
  });

  it("should call askImportMvr when Import from MVR button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    const appStore = useApp();
    await user.click(screen.getByRole("button", { name: "Import from MVR" }));

    expect(appStore.askImportMvr).toHaveBeenCalledTimes(1);
  });

  it("should close window when Exit button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    await user.click(screen.getByRole("button", { name: "Exit" }));

    expect(getCurrentWindow().close).toHaveBeenCalledTimes(1);
  });

  it("should open settings modal when Settings button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    const settingsStore = useSettings();
    await user.click(screen.getByRole("button", { name: "Settings" }));

    expect(settingsStore.isModalOpen).toBe(true);
  });

  it("should retry the update check when the update button is clicked", async () => {
    const user = userEvent.setup();
    render(Welcome);

    const store = useUpdates();
    store.requestUpdate = vi.fn();
    store.status = "error";
    store.error = "network";

    await user.click(await screen.findByRole("button", { name: "Retry update check" }));

    expect(store.requestUpdate).toHaveBeenCalledTimes(1);
  });

  it("should offer to install when an update is available", async () => {
    const store = useUpdates();
    store.appVersion = "0.2.0";
    store.status = "available";
    store.info = { version: "0.3.0" };
    store.pendingUpdate = { version: "0.3.0" } as unknown as Update;

    render(Welcome);

    expect(
      await screen.findByRole("button", { name: "Install update v0.3.0" }),
    ).toBeInTheDocument();
  });

  it("should disable the update button while busy", () => {
    const store = useUpdates();
    store.status = "checking";

    render(Welcome);

    expect(screen.getByRole("button", { name: "Checking for updates..." })).toBeDisabled();
  });

  it("should not show the update button when idle", async () => {
    render(Welcome);

    await screen.findByText("v0.2.0");
    expect(screen.queryByRole("button", { name: "Check for updates" })).not.toBeInTheDocument();
  });

  it("should not show the update button when up to date", () => {
    const store = useUpdates();
    store.status = "up-to-date";

    render(Welcome);

    expect(screen.queryByTitle(/check|update/i)).not.toBeInTheDocument();
  });
});
