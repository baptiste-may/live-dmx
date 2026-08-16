import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useConfirmation } from "./confirmation";

describe("confirmation store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with default values", () => {
    const store = useConfirmation();

    expect(store.isModalOpen).toBe(false);
    expect(store.reason).toBe("");
    expect(store.callback).toBeUndefined();
  });

  it("should set reason, callback, and open modal on ask", () => {
    const store = useConfirmation();
    const callback = vi.fn();

    store.ask("Are you sure?", callback);

    expect(store.reason).toBe("Are you sure?");
    expect(store.callback).toBe(callback);
    expect(store.isModalOpen).toBe(true);
  });

  it("should open modal with optional callback undefined if omitted", () => {
    const store = useConfirmation();

    store.ask("Delete item?");

    expect(store.reason).toBe("Delete item?");
    expect(store.callback).toBeUndefined();
    expect(store.isModalOpen).toBe(true);
  });
});
