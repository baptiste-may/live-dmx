import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";
import { useShowGroups } from "@/stores/show/show-groups";
import { useShowPatch } from "@/stores/show/show-patch";
import { colors } from "@/types/live";

import LiveColorsTableHeader from "./LiveColorsTableHeader.vue";

describe("LiveColorsTableHeader.vue", () => {
  const renderHeader = () => {
    return render(LiveColorsTableHeader, {
      container: document.body.appendChild(document.createElement("thead"))
        .parentElement as HTMLElement,
    });
  };

  it("renders a button for every defined color", () => {
    renderHeader();

    const expectedCount = Object.keys(colors).length;
    const colorButtons = screen.getAllByRole("button");
    expect(colorButtons).toHaveLength(expectedCount);
  });

  it("applies color to color-capable groups when clicked", async () => {
    const user = userEvent.setup();
    const showGroups = useShowGroups();
    const showPatch = useShowPatch();
    const liveButtons = useLiveButtons();

    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
      { id: 2, name: "Wash" },
    ];

    showPatch.fixtures = [
      {
        id: 1,
        groups: [1],
        name: "Spot 1",
        universe: 1,
        address: 1,
        fixtureType: { name: "Dimmer" },
        mode: { name: "1ch", channels: ["Dimmer"] },
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

    renderHeader();

    const redButton = screen.getByTitle("Apply red to all groups");
    await user.click(redButton);

    expect(liveButtons.getGroupColor(0)).toBe("red");
    expect(liveButtons.getGroupColor(2)).toBe("red");
    // Group 1 does not have color capability, so its group color was not modified
    expect(liveButtons.groupColors[1]).toBeUndefined();
  });
});
