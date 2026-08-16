import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useShowPatch } from "@/stores/show/show-patch";

import Patch from "./Patch.vue";
import { mockPatchedFixture } from "./patch/__mocks__/fixtures";

describe("Patch.vue", () => {
  it("should render patch view with form, groups, and visualizer headings", () => {
    const { container } = render(Patch);

    expect(container.querySelector(".grid")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Fixtures" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Groups" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Visualizer" })).toBeInTheDocument();
  });

  it("should display patched fixtures within the integrated patch table", () => {
    const showPatch = useShowPatch();
    showPatch.fixtures = [mockPatchedFixture];

    render(Patch);

    expect(screen.getAllByText("Spot 1").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("1.001")).toBeInTheDocument();
  });
});
