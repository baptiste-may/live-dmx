import { fireEvent, render, screen } from "@testing-library/vue";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useUpdates } from "@/stores/updates";

import SettingsAbout from "./SettingsAbout.vue";

describe("SettingsAbout.vue", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders app and tauri versions", () => {
    const store = useUpdates();
    store.appVersion = "0.2.0";
    store.tauriVersion = "2.1.0";

    render(SettingsAbout);

    expect(screen.getByText("About & Updates")).toBeInTheDocument();
    expect(screen.getByText("v0.2.0")).toBeInTheDocument();
    expect(screen.getByText("2.1.0")).toBeInTheDocument();
  });

  it("checks for updates when the button is clicked", async () => {
    const store = useUpdates();
    store.appVersion = "0.2.0";
    store.checkForUpdates = vi.fn();

    render(SettingsAbout);

    await fireEvent.click(screen.getByTestId("check-updates"));
    expect(store.checkForUpdates).toHaveBeenCalledTimes(1);
  });

  it("offers to install when an update is available", () => {
    const store = useUpdates();
    store.appVersion = "0.2.0";
    store.status = "available";
    store.info = { version: "0.3.0", body: "notes", date: "2026-09-01" };

    render(SettingsAbout);

    expect(screen.getByText("Update available")).toBeInTheDocument();
    expect(screen.getByTestId("install-update")).toHaveTextContent("Install update v0.3.0");
  });

  it("shows download progress while downloading", () => {
    const store = useUpdates();
    store.status = "downloading";
    store.downloaded = 50;
    store.contentLength = 100;

    render(SettingsAbout);

    expect(screen.getByText(/Downloading\.\.\. 50%/)).toBeInTheDocument();
    const progress = document.querySelector("progress");
    expect(progress).toHaveAttribute("value", "50");
    expect(progress).toHaveAttribute("max", "100");
  });

  it("shows an error message when the check fails", () => {
    const store = useUpdates();
    store.status = "error";
    store.error = "network down";

    render(SettingsAbout);

    expect(screen.getByText("network down")).toBeInTheDocument();
    expect(screen.getByTestId("check-updates")).toBeInTheDocument();
  });
});
