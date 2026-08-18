import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import PatchVisualizerForm from "./PatchVisualizerForm.vue";

describe("PatchVisualizerForm.vue", () => {
  it("should render universe label and stepper", () => {
    render(PatchVisualizerForm);

    expect(screen.getByText("Universe :")).toBeInTheDocument();
  });
});
