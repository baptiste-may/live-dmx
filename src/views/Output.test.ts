import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import Output from "./Output.vue";

describe("Output.vue", () => {
  it("should render output heading", () => {
    render(Output);
    expect(screen.getByRole("heading", { name: "Output" })).toBeInTheDocument();
  });
});
