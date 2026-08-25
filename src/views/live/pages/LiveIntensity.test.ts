import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import LiveIntensity from "./LiveIntensity.vue";

describe("LiveIntensity.vue", () => {
  it("renders all intensity sections", () => {
    render(LiveIntensity);

    expect(screen.getByRole("heading", { name: "Effects" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Strobes" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Masters" })).toBeInTheDocument();
  });
});
