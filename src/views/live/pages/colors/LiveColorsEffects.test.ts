import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";
import { useSettings } from "@/stores/settings.ts";
import { useShowGroups } from "@/stores/show/show-groups";
import { useShowPatch } from "@/stores/show/show-patch";

import LiveColorsEffects from "./LiveColorsEffects.vue";

describe("LiveColorsEffects.vue", () => {
  it("renders card, BPM knob, and rows for color-capable groups only", () => {
    const showGroups = useShowGroups();
    const showPatch = useShowPatch();

    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
      { id: 2, name: "Beams" },
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
        name: "Beam 1",
        universe: 1,
        address: 10,
        fixtureType: { name: "RGB" },
        mode: { name: "3ch", channels: ["Red", "Green", "Blue"] },
      },
    ];

    render(LiveColorsEffects);

    expect(screen.getByRole("heading", { name: "Effects" })).toBeInTheDocument();
    expect(screen.getByText(/BPM/i)).toBeInTheDocument();
    expect(screen.queryByText("All :")).not.toBeInTheDocument();
    expect(screen.queryByText("Spots :")).not.toBeInTheDocument();
    expect(screen.getByText("Beams :")).toBeInTheDocument();
  });

  it("updates color BPM on knob keyboard navigation", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();

    render(LiveColorsEffects);

    const knob = screen.getByRole("slider");
    knob.focus();
    await user.keyboard("{ArrowUp}");

    expect(liveButtons.colorEffectBpm).toBe(61);
  });

  it("hides section title when showSectionTitles is false in settings", () => {
    const settings = useSettings();
    settings.live.showSectionTitles = false;

    render(LiveColorsEffects);

    expect(screen.queryByRole("heading", { name: "Effects" })).not.toBeInTheDocument();
    expect(screen.getByText(/BPM/i)).toBeInTheDocument();
  });
});
