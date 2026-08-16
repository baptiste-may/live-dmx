import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import Configure from "./Configure.vue";

describe("Configure.vue", () => {
  it("should render configure heading", () => {
    render(Configure);
    expect(screen.getByRole("heading", { name: "Configure" })).toBeInTheDocument();
  });
});
