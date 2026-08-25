import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";
import { colors } from "@/types/live";

import LiveColorsTableRow from "./LiveColorsTableRow.vue";

describe("LiveColorsTableRow.vue", () => {
  const renderRow = (group = { id: 1, name: "Spots" }) => {
    const table = document.createElement("table");
    const tbody = document.createElement("tbody");
    table.appendChild(tbody);
    document.body.appendChild(table);
    return render(LiveColorsTableRow, {
      props: { group },
      container: tbody,
    });
  };

  it("renders group label and primary/secondary color buttons", () => {
    renderRow();

    expect(screen.getByText("Spots :")).toBeInTheDocument();
    const totalColors = Object.keys(colors).length;
    const buttons = screen.getAllByRole("button");
    // 2 buttons per color (primary & secondary)
    expect(buttons).toHaveLength(totalColors * 2);
  });

  it("selects primary and secondary colors on click", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const buttons = screen.getAllByRole("button");
    const primaryBtn = buttons[0]; // first primary button
    const secondaryBtn = buttons[1]; // first secondary button

    const firstColorKey = Object.keys(colors)[0];

    await user.click(primaryBtn);
    expect(liveButtons.getGroupColor(1)).toBe(firstColorKey);

    await user.click(secondaryBtn);
    expect(liveButtons.getGroupSecondaryColor(1)).toBe(firstColorKey);
  });
});
