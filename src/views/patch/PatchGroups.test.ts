import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import PatchGroups from "./PatchGroups.vue";

describe("PatchGroups.vue", () => {
  it("should render groups heading and group components", () => {
    render(PatchGroups);

    expect(screen.getByRole("heading", { name: "Groups" })).toBeInTheDocument();
  });
});
