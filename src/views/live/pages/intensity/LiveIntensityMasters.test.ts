import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";
import { useSettings } from "@/stores/settings.ts";
import { useShowGroups } from "@/stores/show/show-groups";

import LiveIntensityMasters from "./LiveIntensityMasters.vue";

describe("LiveIntensityMasters.vue", () => {
  it("renders masters controls and badges for fixture groups", () => {
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
    ];

    render(LiveIntensityMasters);

    expect(screen.getByRole("heading", { name: "Masters" })).toBeInTheDocument();
    expect(screen.getByText("Grand Master")).toBeInTheDocument();
    expect(screen.getByText("Spots")).toBeInTheDocument();
  });

  it("updates group master when fader value changes", async () => {
    const showGroups = useShowGroups();
    const liveButtons = useLiveButtons();
    showGroups.groups = [{ id: 1, name: "Spots" }];

    render(LiveIntensityMasters);

    const fader = screen.getByRole("slider");
    await fireEvent.update(fader, "65");

    expect(liveButtons.getGroupMaster(1)).toBe(65);
  });

  it("hides section title when showSectionTitles is false in settings", () => {
    const settings = useSettings();
    settings.live.showSectionTitles = false;

    render(LiveIntensityMasters);

    expect(screen.queryByRole("heading", { name: "Masters" })).not.toBeInTheDocument();
  });
});
