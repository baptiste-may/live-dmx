import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import Output from "./Output.vue";

describe("Output.vue", () => {
  it("should render universes and physical interfaces panels", () => {
    render(Output);

    expect(screen.getByRole("heading", { name: "Universes" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Physical Interfaces" })).toBeInTheDocument();
  });
});
