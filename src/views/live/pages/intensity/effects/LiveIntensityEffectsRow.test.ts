import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";

import LiveIntensityEffectsRow from "./LiveIntensityEffectsRow.vue";

describe("LiveIntensityEffectsRow.vue", () => {
  const renderRow = (group = { id: 1, name: "Front Truss" }) => {
    const table = document.createElement("table");
    const tbody = document.createElement("tbody");
    table.appendChild(tbody);
    document.body.appendChild(table);
    return render(LiveIntensityEffectsRow, {
      props: { group },
      container: tbody,
    });
  };

  it("renders group name and all intensity effect buttons", () => {
    renderRow();

    expect(screen.getByText("Front Truss :")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Full/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Pulse/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Chase/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Ramp up/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Ramp down/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Sine/i })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Triangle/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Dip/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Center/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Bounce/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Reset/i })).toBeInTheDocument();
  });

  it("selects and deselects an effect on click", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const pulseButton = screen.getByRole("button", { name: /Pulse/i });
    await user.click(pulseButton);
    expect(liveButtons.getGroupEffect(1).effect).toBe("pulse");

    const sineButton = screen.getByRole("button", { name: /Sine/i });
    await user.click(sineButton);
    expect(liveButtons.getGroupEffect(1).effect).toBe("sine");

    await user.click(sineButton);
    expect(liveButtons.getGroupEffect(1).effect).toBeNull();
  });

  it("selects and deselects alignment", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const leftButton = screen.getByRole("button", { name: /Left/i });
    await user.click(leftButton);
    expect(liveButtons.getGroupEffect(1).alignment).toBe("left");

    const centerButton = screen.getByRole("button", { name: /Center/i });
    await user.click(centerButton);
    expect(liveButtons.getGroupEffect(1).alignment).toBe("center");

    await user.click(centerButton);
    expect(liveButtons.getGroupEffect(1).alignment).toBeNull();
  });

  it("toggles bounce and resets effect", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const bounceButton = screen.getByRole("button", { name: /Bounce/i });
    await user.click(bounceButton);
    expect(liveButtons.getGroupEffect(1).bounce).toBe(true);

    const resetButton = screen.getByRole("button", { name: /Reset/i });
    await user.click(resetButton);
    expect(liveButtons.getGroupEffect(1)).toEqual({
      effect: null,
      alignment: null,
      bounce: false,
      width: 100,
    });
  });

  it("updates group effect width on knob keyboard navigation", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const knob = screen.getByRole("slider", { name: "Width" });
    knob.focus();
    await user.keyboard("{ArrowDown}");

    expect(liveButtons.getGroupEffect(1).width).toBe(99);
  });
});
