import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useApp } from "./app";
import { ShowStatus, useShow } from "./show/show";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

const mockOpenDialog = vi.mocked(openDialog);

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

    await app.createShow();

    expect(show.isLoaded).toBe(true);
  });

  it("should load show from path and set loaded state", async () => {
    const app = useApp();
    const show = useShow();

    await app.loadShow("/path/to/show.json");

    expect(show.isLoaded).toBe(true);
  });

  it("should unload show when closeShow is called", async () => {
    const app = useApp();
    const show = useShow();

    await app.createShow();
    expect(show.isLoaded).toBe(true);

    app.closeShow();
    expect(show.isLoaded).toBe(false);
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
