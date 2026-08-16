import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { usePatchVisualizer } from "./patch-visualizer";

describe("patch-visualizer store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with default universe 1", () => {
    const store = usePatchVisualizer();

    expect(store.currentUniverse).toBe(1);
  });

  it("should allow updating current universe", () => {
    const store = usePatchVisualizer();

    store.currentUniverse = 2;
    expect(store.currentUniverse).toBe(2);
  });
});
