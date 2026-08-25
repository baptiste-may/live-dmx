import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";

import LiveColorsEffectsRow from "./LiveColorsEffectsRow.vue";

describe("LiveColorsEffectsRow.vue", () => {
  const renderRow = (group = { id: 1, name: "Back Wall" }) => {
    return render(LiveColorsEffectsRow, {
      props: { group },
      container: document.body.appendChild(document.createElement("tbody"))
        .parentElement as HTMLElement,
    });
  };

  it("renders group name and color effect buttons", () => {
    renderRow();

    expect(screen.getByText("Back Wall :")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Pulse/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Chase/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Ramp up/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Ramp down/i })).toBeInTheDocument();
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

    await user.click(chaseButton);
    expect(liveButtons.getGroupColorEffect(1).effect).toBeNull();
  });

  it("selects and deselects alignment", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();
    renderRow();

    const wingButton = screen.getByRole("button", { name: /Wing/i });
    await user.click(wingButton);
    expect(liveButtons.getGroupColorEffect(1).alignment).toBe("wing");

    await user.click(wingButton);
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
    });
  });
});
