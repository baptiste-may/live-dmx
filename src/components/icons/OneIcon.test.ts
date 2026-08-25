import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import OneIcon from "./OneIcon.vue";

describe("OneIcon.vue", () => {
  it("should render button with number 1", () => {
    render(OneIcon);
    expect(screen.getByRole("button", { name: "1" })).toBeInTheDocument();
  });
});
