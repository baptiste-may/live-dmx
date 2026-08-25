import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import FourIcon from "./FourIcon.vue";

describe("FourIcon.vue", () => {
  it("should render button with number 4", () => {
    render(FourIcon);
    expect(screen.getByRole("button", { name: "4" })).toBeInTheDocument();
  });
});
