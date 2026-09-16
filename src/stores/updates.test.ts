import { getTauriVersion, getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ToastType, useToastStore } from "./toast";
import { useUpdates } from "./updates";

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn().mockResolvedValue("0.2.0"),
  getTauriVersion: vi.fn().mockResolvedValue("2.1.0"),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn().mockResolvedValue(undefined),
}));

const mockGetVersion = vi.mocked(getVersion);
const mockGetTauriVersion = vi.mocked(getTauriVersion);
const mockCheck = vi.mocked(check);
const mockRelaunch = vi.mocked(relaunch);

function mockUpdate(version = "0.3.0") {
  return {
    version,
    body: "release notes",
    date: "2026-09-01",
    downloadAndInstall: vi.fn().mockResolvedValue(undefined),
  } as unknown as Update;
}

function emitStarted(update: Update, contentLength: number) {
  const event: DownloadEvent = { event: "Started", data: { contentLength } };
  (update.downloadAndInstall as ReturnType<typeof vi.fn>).mock.calls[0][0](event);
}

function emitProgress(update: Update, chunkLength: number) {
  const event: DownloadEvent = { event: "Progress", data: { chunkLength } };
  (update.downloadAndInstall as ReturnType<typeof vi.fn>).mock.calls[0][0](event);
}

describe("updates store", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads app and tauri versions on init", async () => {
    const updates = useUpdates();
    await updates.init();

    expect(updates.appVersion).toBe("0.2.0");
    expect(updates.tauriVersion).toBe("2.1.0");
  });

  it("does not reload versions once initialized", async () => {
    const updates = useUpdates();
    await updates.init();
    await updates.init();

    expect(mockGetVersion).toHaveBeenCalledTimes(1);
  });

  it("falls back to 0.0.0 when version retrieval fails", async () => {
    mockGetVersion.mockRejectedValueOnce(new Error("nope"));
    mockGetTauriVersion.mockRejectedValueOnce(new Error("nope"));

    const updates = useUpdates();
    await updates.init();

    expect(updates.appVersion).toBe("0.0.0");
    expect(updates.tauriVersion).toBe("");
  });

  it("reports an available update", async () => {
    mockCheck.mockResolvedValue(mockUpdate("0.3.0"));

    const updates = useUpdates();
    await updates.checkForUpdates();

    expect(updates.status).toBe("available");
    expect(updates.hasUpdate).toBe(true);
    expect(updates.info?.version).toBe("0.3.0");
  });

  it("reports when no update is available", async () => {
    mockCheck.mockResolvedValue(null);

    const updates = useUpdates();
    updates.appVersion = "0.2.0";
    const toast = useToastStore();
    const addToast = vi.spyOn(toast, "addToast");

    await updates.checkForUpdates();

    expect(updates.status).toBe("up-to-date");
    expect(addToast).toHaveBeenCalledWith("You're up to date (v0.2.0)", ToastType.SUCCESS);
  });

  it("handles a failed update check", async () => {
    mockCheck.mockRejectedValue(new Error("network down"));

    const updates = useUpdates();
    const toast = useToastStore();
    const addToast = vi.spyOn(toast, "addToast");

    await updates.checkForUpdates();

    expect(updates.status).toBe("error");
    expect(updates.error).toBe("network down");
    expect(addToast).toHaveBeenCalledWith("Update check failed: network down", ToastType.ERROR);
  });

  it("downloads and installs a pending update, then relaunches", async () => {
    const update = mockUpdate();
    mockCheck.mockResolvedValue(update);

    const updates = useUpdates();
    await updates.checkForUpdates();
    await updates.downloadAndInstall();

    expect(update.downloadAndInstall).toHaveBeenCalled();
    expect(updates.status).toBe("installing");
    expect(mockRelaunch).toHaveBeenCalledTimes(1);
  });

  it("tracks download progress from events", async () => {
    const update = mockUpdate();
    mockCheck.mockResolvedValue(update);

    const updates = useUpdates();
    await updates.checkForUpdates();
    const promise = updates.downloadAndInstall();

    emitStarted(update, 100);
    emitProgress(update, 25);
    emitProgress(update, 75);

    expect(updates.contentLength).toBe(100);
    expect(updates.downloaded).toBe(100);
    expect(updates.progress).toBe(1);

    await promise;
  });

  it("does nothing when installing without a pending update", async () => {
    const updates = useUpdates();
    await updates.downloadAndInstall();

    expect(mockRelaunch).not.toHaveBeenCalled();
    expect(updates.status).toBe("idle");
  });

  it("handles a failed install", async () => {
    const update = mockUpdate();
    (update.downloadAndInstall as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
      new Error("install failed"),
    );
    mockCheck.mockResolvedValue(update);

    const updates = useUpdates();
    await updates.checkForUpdates();
    await updates.downloadAndInstall();

    expect(updates.status).toBe("error");
    expect(updates.error).toBe("install failed");
    expect(mockRelaunch).not.toHaveBeenCalled();
  });
});
