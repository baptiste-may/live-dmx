import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import PatchVisualizer from "./PatchVisualizer.vue";

describe("PatchVisualizer.vue", () => {
  it("should render visualizer title and components", () => {
    render(PatchVisualizer);

    expect(screen.getByRole("heading", { name: "Visualizer" })).toBeInTheDocument();
  });
});
