import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import TwoIcon from "./TwoIcon.vue";

describe("TwoIcon.vue", () => {
  it("should render button with number 2", () => {
    render(TwoIcon);
    expect(screen.getByRole("button", { name: "2" })).toBeInTheDocument();
  });
});
