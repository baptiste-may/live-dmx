import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useApp } from "@/stores/app";
import { ShowStatus } from "@/stores/show/show";

import { OPEN_SHOW_EVENT, useShowOpener } from "./show-opener";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

describe("show-opener", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should load the pending open path on start", async () => {
    mockInvoke.mockResolvedValue("/shows/show.ldmx");
    mockListen.mockResolvedValue(vi.fn());

    const loadSpy = vi.spyOn(useApp(), "loadShow");

    await useShowOpener().start();

    expect(mockInvoke).toHaveBeenCalledWith("get_pending_open_path");
    expect(loadSpy).toHaveBeenCalledWith("/shows/show.ldmx");
    expect(mockListen).toHaveBeenCalledWith(OPEN_SHOW_EVENT, expect.any(Function));
  });

  it("should load the path received via the open-show-request event", async () => {
    mockInvoke.mockResolvedValue(null);
    let handler: ((event: { payload: string }) => void) | undefined;
    mockListen.mockImplementation(async (_event, cb) => {
      handler = cb as (event: { payload: string }) => void;
      return () => {};
    });

    const loadSpy = vi.spyOn(useApp(), "loadShow");

    await useShowOpener().start();
    handler?.({ payload: "/shows/other.ldmx" });

    await vi.waitFor(() => {
      expect(loadSpy).toHaveBeenCalledWith("/shows/other.ldmx");
    });
  });

  it("should not load the same path twice", async () => {
    mockInvoke.mockResolvedValue("/shows/show.ldmx");
    mockListen.mockResolvedValue(vi.fn());

    const loadSpy = vi.spyOn(useApp(), "loadShow");
    const opener = useShowOpener();

    await opener.start();
    await opener.openPath("/shows/show.ldmx");

    expect(loadSpy).toHaveBeenCalledTimes(1);
  });

  it("should allow retrying the same path after a failed load", async () => {
    mockInvoke.mockResolvedValue("/shows/show.ldmx");
    mockListen.mockResolvedValue(vi.fn());

    const loadSpy = vi.spyOn(useApp(), "loadShow").mockRejectedValueOnce(new Error("boom"));
    const opener = useShowOpener();

    await opener.start();
    await opener.openPath("/shows/show.ldmx");

    expect(loadSpy).toHaveBeenCalledTimes(2);
  });

  it("should ignore empty pending paths", async () => {
    mockInvoke.mockResolvedValue(null);
    mockListen.mockResolvedValue(vi.fn());

    const loadSpy = vi.spyOn(useApp(), "loadShow");

    await useShowOpener().start();

    expect(loadSpy).not.toHaveBeenCalled();
  });

  it("should allow re-opening a path after the show changed", async () => {
    mockInvoke.mockResolvedValue("/shows/show.ldmx");
    mockListen.mockResolvedValue(vi.fn());

    const app = useApp();
    const loadSpy = vi.spyOn(app, "loadShow");
    const opener = useShowOpener();

    await opener.start();

    app.show.state = ShowStatus.NOT_LOADED;
    app.show.file.path = undefined;
    await opener.openPath("/shows/show.ldmx");

    expect(loadSpy).toHaveBeenCalledTimes(2);
  });

  it("should register the listener before reading the pending path", async () => {
    const calls: string[] = [];
    mockInvoke.mockImplementation(async () => {
      calls.push("get_pending_open_path");
      return null;
    });
    mockListen.mockImplementation(async () => {
      calls.push("listen");
      return () => {};
    });

    const loadSpy = vi.spyOn(useApp(), "loadShow");

    await useShowOpener().start();

    expect(calls).toEqual(["listen", "get_pending_open_path"]);
    expect(loadSpy).not.toHaveBeenCalled();
  });
});
