import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { ToastType, useToastStore } from "./toast";

describe("toast store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("should initialize with empty toasts array", () => {
    const store = useToastStore();
    expect(store.toasts).toEqual([]);
  });

  it("should add a toast with default parameters", () => {
    const store = useToastStore();
    store.addToast("Test info message");

    expect(store.toasts).toHaveLength(1);
    expect(store.toasts[0].message).toBe("Test info message");
    expect(store.toasts[0].type).toBe(ToastType.INFO);
    expect(store.toasts[0].duration).toBe(10_000);
  });

  it("should add a toast with custom type and duration", () => {
    const store = useToastStore();
    store.addToast("Error occurred", ToastType.ERROR, 5000);

    expect(store.toasts).toHaveLength(1);
    expect(store.toasts[0].message).toBe("Error occurred");
    expect(store.toasts[0].type).toBe(ToastType.ERROR);
    expect(store.toasts[0].duration).toBe(5000);
  });

  it("should remove toast by id", () => {
    const store = useToastStore();
    store.addToast("Message 1");
    const toastId = store.toasts[0].id;

    store.removeToast(toastId);

    expect(store.toasts).toHaveLength(0);
  });

  it("should automatically remove toast after duration expires", () => {
    const store = useToastStore();
    store.addToast("Auto dismiss message", ToastType.SUCCESS, 3000);

    expect(store.toasts).toHaveLength(1);

    vi.advanceTimersByTime(3000);

    expect(store.toasts).toHaveLength(0);
  });
});
