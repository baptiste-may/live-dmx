import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useToastStore } from "@/stores/toast.ts";

import { useOutputInterfaces } from "./output-interfaces.ts";

describe("output-interfaces store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with empty physical interfaces", () => {
    const store = useOutputInterfaces();
    expect(store.interfaces.length).toBe(0);
    expect(store.isScanning).toBe(false);
  });

  it("should scan interfaces and notify toast", async () => {
    const store = useOutputInterfaces();
    const toastStore = useToastStore();
    const addToastSpy = vi.spyOn(toastStore, "addToast");

    await store.scanInterfaces();

    expect(store.isScanning).toBe(false);
    expect(addToastSpy).toHaveBeenCalledWith(
      expect.stringContaining("Scanned interfaces: 0 found"),
      expect.anything(),
    );
  });

  it("should clear interfaces when fetch fails", async () => {
    const store = useOutputInterfaces();
    store.interfaces = [
      {
        id: "physical-1",
        name: "Physical 1",
        device: "Test",
        port: "/dev/test",
        status: "connected",
        fps: 44,
      },
    ];

    await store.fetchInterfaces();
    // Non-Tauri environment rejects/fails invoke, clearing interfaces to []
    expect(store.interfaces).toEqual([]);
  });
});
