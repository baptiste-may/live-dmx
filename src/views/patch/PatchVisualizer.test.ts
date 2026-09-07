import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import PatchVisualizer from "./PatchVisualizer.vue";

describe("PatchVisualizer.vue", () => {
  it("should render visualizer title and universe selector", () => {
    render(PatchVisualizer);

    expect(screen.getByRole("heading", { name: "Visualizer" })).toBeInTheDocument();
    expect(screen.getByText("Universe :")).toBeInTheDocument();
  });
});
