import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";
import { useShowGroups } from "@/stores/show/show-groups";

import LiveIntensityEffects from "./LiveIntensityEffects.vue";

describe("LiveIntensityEffects.vue", () => {
  it("renders effects card, BPM knob, and rows for non-zero groups", () => {
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
      { id: 2, name: "Beams" },
    ];

    render(LiveIntensityEffects);

    expect(screen.getByRole("heading", { name: "Effects" })).toBeInTheDocument();
    expect(screen.getByText(/BPM/i)).toBeInTheDocument();
    expect(screen.queryByText("All :")).not.toBeInTheDocument();
    expect(screen.getByText("Spots :")).toBeInTheDocument();
    expect(screen.getByText("Beams :")).toBeInTheDocument();
  });

  it("updates BPM on knob keyboard navigation", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();

    render(LiveIntensityEffects);

    const knob = screen.getByRole("slider");
    knob.focus();
    await user.keyboard("{ArrowUp}");

    expect(liveButtons.effectBpm).toBe(61);
  });
});
