import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useShowGroups } from "@/stores/show/show-groups";

import LiveColorsTable from "./LiveColorsTable.vue";

describe("LiveColorsTable.vue", () => {
  it("renders colors card, header, and rows for non-zero groups", () => {
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
      { id: 2, name: "Wash" },
    ];

    render(LiveColorsTable);

    expect(screen.getByRole("heading", { name: "Colors" })).toBeInTheDocument();
    expect(screen.queryByText("All :")).not.toBeInTheDocument();
    expect(screen.getByText("Spots :")).toBeInTheDocument();
    expect(screen.getByText("Wash :")).toBeInTheDocument();
  });
});
