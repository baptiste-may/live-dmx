import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import ThreeIcon from "./ThreeIcon.vue";

describe("ThreeIcon.vue", () => {
  it("should render button with number 3", () => {
    render(ThreeIcon);
    expect(screen.getByRole("button", { name: "3" })).toBeInTheDocument();
  });
});
