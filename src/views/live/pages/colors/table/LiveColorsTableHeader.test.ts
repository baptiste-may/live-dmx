import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";
import { useShowGroups } from "@/stores/show/show-groups";
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

  it("applies color to all groups when clicked", async () => {
    const user = userEvent.setup();
    const showGroups = useShowGroups();
    const liveButtons = useLiveButtons();

    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
      { id: 2, name: "Wash" },
    ];

    renderHeader();

    const redButton = screen.getByTitle("Apply red to all groups");
    await user.click(redButton);

    expect(liveButtons.getGroupColor(0)).toBe("red");
    expect(liveButtons.getGroupColor(1)).toBe("red");
    expect(liveButtons.getGroupColor(2)).toBe("red");
  });
});
