import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";

import LiveColorsEffectsRow from "./LiveColorsEffectsRow.vue";

describe("LiveColorsEffectsRow.vue", () => {
  const renderRow = (group = { id: 1, name: "Back Wall" }) => {
    const table = document.createElement("table");
    const tbody = document.createElement("tbody");
    table.appendChild(tbody);
    document.body.appendChild(table);
    return render(LiveColorsEffectsRow, {
      props: { group },
      container: tbody,
    });
  };

  it("renders group name and color effect buttons", () => {
    renderRow();

    expect(screen.getByText("Back Wall :")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Pulse/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Chase/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Ramp up/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Ramp down/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Sine/i })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Triangle/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Dip/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Rainbow/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Center/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Bounce/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Reset/i })).toBeInTheDocument();
  });

  it("selects and deselects a color effect", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const chaseButton = screen.getByRole("button", { name: /Chase/i });
    await user.click(chaseButton);
    expect(liveButtons.getGroupColorEffect(1).effect).toBe("chase");

    const rainbowButton = screen.getByRole("button", { name: /Rainbow/i });
    await user.click(rainbowButton);
    expect(liveButtons.getGroupColorEffect(1).effect).toBe("rainbow");

    await user.click(rainbowButton);
    expect(liveButtons.getGroupColorEffect(1).effect).toBeNull();
  });

  it("selects and deselects alignment", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const wingButton = screen.getByRole("button", { name: /Wing/i });
    await user.click(wingButton);
    expect(liveButtons.getGroupColorEffect(1).alignment).toBe("wing");

    const centerButton = screen.getByRole("button", { name: /Center/i });
    await user.click(centerButton);
    expect(liveButtons.getGroupColorEffect(1).alignment).toBe("center");

    await user.click(centerButton);
    expect(liveButtons.getGroupColorEffect(1).alignment).toBeNull();
  });

  it("toggles bounce and resets effect", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const bounceButton = screen.getByRole("button", { name: /Bounce/i });
    await user.click(bounceButton);
    expect(liveButtons.getGroupColorEffect(1).bounce).toBe(true);

    const resetButton = screen.getByRole("button", { name: /Reset/i });
    await user.click(resetButton);
    expect(liveButtons.getGroupColorEffect(1)).toEqual({
      effect: null,
      alignment: null,
      bounce: false,
      width: 100,
    });
  });

  it("updates group color effect width on knob keyboard navigation", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const knob = screen.getByRole("slider", { name: "Width" });
    knob.focus();
    await user.keyboard("{ArrowDown}");

    expect(liveButtons.getGroupColorEffect(1).width).toBe(99);
  });
});
