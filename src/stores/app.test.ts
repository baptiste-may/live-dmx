import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useApp } from "./app";
import { useLiveButtons } from "./live/live-buttons";
import { ShowStatus, useShow } from "./show/show";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

const mockOpenDialog = vi.mocked(openDialog);
const mockInvoke = vi.mocked(invoke);

describe("app store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("should report isShowLoaded state based on show store", () => {
    const app = useApp();
    const show = useShow();

    expect(app.isShowLoaded).toBe(false);

    show.state = ShowStatus.LOADED;
    expect(app.isShowLoaded).toBe(true);
  });

  it("should create new show and set loaded state", async () => {
    const app = useApp();
    const show = useShow();
    const liveButtons = useLiveButtons();

    liveButtons.strobeRate = 75;
    liveButtons.effectBpm = 130;

    await app.createShow();

    expect(show.isLoaded).toBe(true);
    expect(liveButtons.strobeRate).toBe(50);
    expect(liveButtons.effectBpm).toBe(60);
    expect(mockInvoke).toHaveBeenCalledWith("live_reset_all_buttons");
  });

  it("should load show from path and set loaded state", async () => {
    const app = useApp();
    const show = useShow();
    const liveButtons = useLiveButtons();

    liveButtons.strobeRate = 85;
    liveButtons.effectBpm = 110;

    await app.loadShow("/path/to/show.json");

    expect(show.isLoaded).toBe(true);
    expect(liveButtons.strobeRate).toBe(50);
    expect(liveButtons.effectBpm).toBe(60);
    expect(mockInvoke).toHaveBeenCalledWith("live_reset_all_buttons");
  });

  it("should unload show when closeShow is called", async () => {
    const app = useApp();
    const show = useShow();
    const liveButtons = useLiveButtons();

    await app.createShow();
    expect(show.isLoaded).toBe(true);

    liveButtons.strobeRate = 95;
    liveButtons.effectBpm = 150;
    mockInvoke.mockClear();

    await app.closeShow();
    expect(show.isLoaded).toBe(false);
    expect(liveButtons.strobeRate).toBe(50);
    expect(liveButtons.effectBpm).toBe(60);
    expect(mockInvoke).toHaveBeenCalledWith("live_reset_all_buttons");
  });

  it("should open dialog and load show when file is selected in askLoadShow", async () => {
    const app = useApp();
    const show = useShow();

    mockOpenDialog.mockResolvedValueOnce("/path/to/selected.ldmx");

    await app.askLoadShow();

    expect(mockOpenDialog).toHaveBeenCalledTimes(1);
    expect(show.isLoaded).toBe(true);
    expect(show.file.path).toBe("/path/to/selected.ldmx");
  });

  it("should do nothing when dialog is cancelled in askLoadShow", async () => {
    const app = useApp();
    const show = useShow();

    mockOpenDialog.mockResolvedValueOnce(null);

    await app.askLoadShow();

    expect(mockOpenDialog).toHaveBeenCalledTimes(1);
    expect(show.isLoaded).toBe(false);
  });
});
