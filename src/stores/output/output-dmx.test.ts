import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useOutputDmx } from "./output-dmx.ts";

const mockInvoke = vi.mocked(invoke);

describe("output-dmx store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it("should initialize with default state", () => {
    const store = useOutputDmx();
    expect(store.universeBuffers).toEqual({});
    expect(store.isPaused).toBe(false);
    expect(store.totalActiveCount).toBe(0);
    expect(store.getUniverseData(1)).toEqual(Array.from({ length: 512 }, () => 0));
    expect(store.getActiveCount(1)).toBe(0);
  });

  it("should fetch all DMX universe buffers", async () => {
    const mockData: Record<number, number[]> = {
      1: Array.from({ length: 512 }, () => 0),
      2: Array.from({ length: 512 }, () => 100),
    };
    mockInvoke.mockResolvedValue(mockData);

    const store = useOutputDmx();
    await store.fetchAllDmxData();

    expect(mockInvoke).toHaveBeenCalledWith("get_all_dmx_data");
    expect(store.universeBuffers).toEqual(mockData);
    expect(store.getUniverseData(2)[0]).toBe(100);
    expect(store.getActiveCount(2)).toBe(512);
    expect(store.totalActiveCount).toBe(512);
  });

  it("should toggle pause and not fetch when paused", async () => {
    const store = useOutputDmx();
    expect(store.isPaused).toBe(false);

    store.togglePause();
    expect(store.isPaused).toBe(true);

    await store.fetchAllDmxData();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("should discard stale out-of-order responses", async () => {
    const store = useOutputDmx();

    let resolveFirst!: (data: Record<number, number[]>) => void;
    let resolveSecond!: (data: Record<number, number[]>) => void;

    const firstPromise = new Promise<Record<number, number[]>>((res) => {
      resolveFirst = res;
    });
    const secondPromise = new Promise<Record<number, number[]>>((res) => {
      resolveSecond = res;
    });

    mockInvoke.mockReturnValueOnce(firstPromise as unknown as ReturnType<typeof invoke>);
    mockInvoke.mockReturnValueOnce(secondPromise as unknown as ReturnType<typeof invoke>);

    const call1 = store.fetchAllDmxData();
    const call2 = store.fetchAllDmxData();

    // Second response arrives first
    resolveSecond({ 1: Array.from({ length: 512 }, () => 200) });
    await call2;
    expect(store.getUniverseData(1)[0]).toBe(200);

    // Stale first response arrives later
    resolveFirst({ 1: Array.from({ length: 512 }, () => 100) });
    await call1;
    expect(store.getUniverseData(1)[0]).toBe(200);
  });

  it("should not update buffers if paused while request was in-flight", async () => {
    const store = useOutputDmx();
    let resolveRequest!: (data: Record<number, number[]>) => void;
    const requestPromise = new Promise<Record<number, number[]>>((res) => {
      resolveRequest = res;
    });

    mockInvoke.mockReturnValueOnce(requestPromise as unknown as ReturnType<typeof invoke>);

    const fetchPromise = store.fetchAllDmxData();
    store.togglePause();
    expect(store.isPaused).toBe(true);

    resolveRequest({ 1: Array.from({ length: 512 }, () => 150) });
    await fetchPromise;

    expect(store.universeBuffers).toEqual({});
  });
});
