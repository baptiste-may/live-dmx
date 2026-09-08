import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { useOutputUniverses } from "./output-universes.ts";

describe("output-universes store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("should initialize with default universe set to none and enabled", () => {
    const store = useOutputUniverses();
    expect(store.universes.length).toBe(1);
    expect(store.universes[0].universe).toBe(1);
    expect(store.universes[0].output).toBe("none");
    expect(store.universes[0].enabled).toBe(true);
  });

  it("should update universe output", () => {
    const store = useOutputUniverses();
    store.setUniverseOutput(1, "artnet");
    expect(store.universes[0].output).toBe("artnet");
  });

  it("should enable and disable a universe", () => {
    const store = useOutputUniverses();
    store.setUniverseEnabled(1, false);
    expect(store.universes[0].enabled).toBe(false);

    store.toggleUniverseEnabled(1);
    expect(store.universes[0].enabled).toBe(true);
  });

  it("should do nothing when updating non-existent universe", () => {
    const store = useOutputUniverses();
    store.setUniverseOutput(999, "artnet");
    store.setUniverseEnabled(999, true);
    store.toggleUniverseEnabled(999);
    expect(store.universes.find((u) => u.universe === 999)).toBeUndefined();
  });

  it("should add a new universe enabled by default", () => {
    const store = useOutputUniverses();
    const initialCount = store.universes.length;
    store.addUniverse();
    expect(store.universes.length).toBe(initialCount + 1);
    expect(store.universes[store.universes.length - 1].universe).toBe(2);
    expect(store.universes[store.universes.length - 1].output).toBe("none");
    expect(store.universes[store.universes.length - 1].enabled).toBe(true);
  });

  it("should remove a universe when more than 1 exist", () => {
    const store = useOutputUniverses();
    store.addUniverse();
    expect(store.universes.length).toBe(2);

    store.removeUniverse(2);
    expect(store.universes.find((u) => u.universe === 2)).toBeUndefined();
    expect(store.universes.length).toBe(1);
  });

  it("should not remove a universe if only 1 remains", () => {
    const store = useOutputUniverses();
    expect(store.universes.length).toBe(1);
    store.removeUniverse(1);
    expect(store.universes.length).toBe(1);
  });

  it("should map assigned universes by physical interface only when enabled", () => {
    const store = useOutputUniverses();
    store.setUniverseOutput(1, "physical-1");
    expect(store.assignedUniversesByInterface["physical-1"]).toEqual([1]);
    expect(store.assignedUniversesByInterface["physical-2"]).toBeUndefined();

    store.setUniverseEnabled(1, false);
    expect(store.assignedUniversesByInterface["physical-1"]).toBeUndefined();
  });
});
