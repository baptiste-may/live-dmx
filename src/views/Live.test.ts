import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import Live from "./Live.vue";

describe("Live.vue", () => {
  it("should render live navigation buttons", () => {
    render(Live);
    expect(screen.getByRole("button", { name: "Intensity" })).toBeInTheDocument();
  });
});
