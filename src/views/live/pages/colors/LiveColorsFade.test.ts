import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";

import LiveColorsFade from "./LiveColorsFade.vue";

describe("LiveColorsFade.vue", () => {
  it("renders fade speed knob and order toggle buttons", () => {
    render(LiveColorsFade);

    expect(screen.getByRole("heading", { name: "Fade" })).toBeInTheDocument();
    expect(screen.getByText("Speed (s)")).toBeInTheDocument();
    expect(screen.getByText("Order")).toBeInTheDocument();
    expect(screen.getByRole("slider")).toBeInTheDocument();
    expect(screen.getAllByRole("button")).toHaveLength(3);
  });

  it("updates fade speed via knob", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();

    render(LiveColorsFade);

    const knob = screen.getByRole("slider");
    knob.focus();
    await user.keyboard("{ArrowUp}");

    expect(liveButtons.colorFadeSpeed).toBe(1.25);
  });

  it("selects and deselects fade order", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();

    render(LiveColorsFade);

    const buttons = screen.getAllByRole("button");
    const [leftBtn, rightBtn, wingBtn] = buttons;

    await user.click(leftBtn);
    expect(liveButtons.colorFadeOrder).toBe("left");

    await user.click(rightBtn);
    expect(liveButtons.colorFadeOrder).toBe("right");

    await user.click(wingBtn);
    expect(liveButtons.colorFadeOrder).toBe("wing");

    await user.click(wingBtn);
    expect(liveButtons.colorFadeOrder).toBeNull();
  });
});
