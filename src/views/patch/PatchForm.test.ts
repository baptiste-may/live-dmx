import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import PatchForm from "./PatchForm.vue";

describe("PatchForm.vue", () => {
  it("should render fixtures title and child components", () => {
    render(PatchForm);

    expect(screen.getByRole("heading", { name: "Fixtures" })).toBeInTheDocument();
  });
});
