import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import Input from "./Input.vue";

describe("Input.vue", () => {
  it("should render input heading", () => {
    render(Input);
    expect(screen.getByRole("heading", { name: "Input" })).toBeInTheDocument();
  });
});
