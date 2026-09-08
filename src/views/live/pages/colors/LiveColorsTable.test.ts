import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useSettings } from "@/stores/settings.ts";
import { useShowGroups } from "@/stores/show/show-groups";
import { useShowPatch } from "@/stores/show/show-patch";

import LiveColorsTable from "./LiveColorsTable.vue";

describe("LiveColorsTable.vue", () => {
  it("renders only groups with color capability (excluding group 0 and non-color groups)", () => {
    const showGroups = useShowGroups();
    const showPatch = useShowPatch();

    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
      { id: 2, name: "Wash" },
      { id: 3, name: "Empty" },
    ];

    showPatch.fixtures = [
      {
        id: 1,
        groups: [1],
        name: "Spot 1",
        universe: 1,
        address: 1,
        fixtureType: { name: "Dimmer" },
        mode: { name: "1ch", channels: ["Dimmer", "Pan", "Tilt"] },
      },
      {
        id: 2,
        groups: [2],
        name: "Wash 1",
        universe: 1,
        address: 10,
        fixtureType: { name: "RGB" },
        mode: { name: "3ch", channels: ["Red", "Green", "Blue"] },
      },
    ];

    render(LiveColorsTable);

    expect(screen.getByRole("heading", { name: "Colors" })).toBeInTheDocument();
    expect(screen.queryByText("All :")).not.toBeInTheDocument();
    expect(screen.queryByText("Spots :")).not.toBeInTheDocument();
    expect(screen.queryByText("Empty :")).not.toBeInTheDocument();
    expect(screen.getByText("Wash :")).toBeInTheDocument();
  });

  it("hides section title when showSectionTitles is false in settings", () => {
    const settings = useSettings();
    settings.live.showSectionTitles = false;

    render(LiveColorsTable);

    expect(screen.queryByRole("heading", { name: "Colors" })).not.toBeInTheDocument();
  });
});
