import userEvent from "@testing-library/user-event";
import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";
import { useShowGroups } from "@/stores/show/show-groups";

import LiveIntensityStrobes from "./LiveIntensityStrobes.vue";

describe("LiveIntensityStrobes.vue", () => {
  it("renders strobes controls and groups", () => {
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Front" },
    ];

    render(LiveIntensityStrobes);

    expect(screen.getByRole("heading", { name: "Strobes" })).toBeInTheDocument();
    expect(screen.getByText("All :")).toBeInTheDocument();
    expect(screen.getByText("Front :")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Strobe" })).toHaveLength(2);
    expect(screen.getAllByRole("button", { name: "Random" })).toHaveLength(2);
  });

  it("updates strobe rates via knobs", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();

    render(LiveIntensityStrobes);

    const knobs = screen.getAllByRole("slider");
    expect(knobs).toHaveLength(2);

    knobs[0].focus();
    await user.keyboard("{ArrowUp}");
    expect(liveButtons.strobeRate).toBe(51);

    knobs[1].focus();
    await user.keyboard("{ArrowUp}");
    expect(liveButtons.randomStrobeRate).toBe(51);
  });

  it("triggers strobe flash on button press", async () => {
    const showGroups = useShowGroups();
    const liveButtons = useLiveButtons();
    showGroups.groups = [{ id: 1, name: "Front" }];

    render(LiveIntensityStrobes);

    const strobeBtn = screen.getByRole("button", { name: "Strobe" });
    const randomBtn = screen.getByRole("button", { name: "Random" });

    await fireEvent.mouseDown(strobeBtn);
    expect(liveButtons.isFlashActive(1, "strobe")).toBe(true);
    await fireEvent.mouseUp(strobeBtn);
    expect(liveButtons.isFlashActive(1, "strobe")).toBe(false);

    await fireEvent.mouseDown(randomBtn);
    expect(liveButtons.isFlashActive(1, "randomStrobe")).toBe(true);
    await fireEvent.mouseUp(randomBtn);
    expect(liveButtons.isFlashActive(1, "randomStrobe")).toBe(false);
  });
});
