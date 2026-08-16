import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { usePatchVisualizer } from "@/stores/patch/patch-visualizer.ts";
import { useShowPatch } from "@/stores/show/show-patch.ts";
import type { PatchedFixture } from "@/types/patch.ts";

import PatchVisualizerGrid from "./PatchVisualizerGrid.vue";

describe("PatchVisualizerGrid.vue", () => {
  it("should render grid slots and display universe fixtures", () => {
    const showPatch = useShowPatch();
    const visualizer = usePatchVisualizer();
    visualizer.currentUniverse = 1;

    const fixture: PatchedFixture = {
      id: 1,
      name: "Front Par 1",
      universe: 1,
      address: 1,
      groups: [0],
      fixtureType: {
        name: "LED Par",
        categories: ["Color Changer"],
        modes: [],
        fixtureKey: "led-par",
        manufacturerKey: "generic",
      },
      mode: {
        name: "3-Channel",
        shortName: "3ch",
        channels: ["Red", "Green", "Blue"],
      },
    };

    showPatch.fixtures = [fixture];

    render(PatchVisualizerGrid);

    expect(screen.getByText("Front Par 1")).toBeInTheDocument();
  });
});
