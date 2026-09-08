import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import Output from "./Output.vue";

describe("Output.vue", () => {
  it("should render universes, physical interfaces, and dmx output panels", () => {
    render(Output);

    expect(screen.getByRole("heading", { name: "Universes" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Physical Interfaces" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "DMX Output" })).toBeInTheDocument();
  });
});
