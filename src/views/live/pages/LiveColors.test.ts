import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import LiveColors from "./LiveColors.vue";

describe("LiveColors.vue", () => {
  it("renders all color sections", () => {
    render(LiveColors);

    expect(screen.getByRole("heading", { name: "Colors" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Fade" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Effects" })).toBeInTheDocument();
  });
});
