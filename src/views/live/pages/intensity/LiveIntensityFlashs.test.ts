import userEvent from "@testing-library/user-event";
import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useLiveButtons } from "@/stores/live/live-buttons";
import { useSettings } from "@/stores/settings.ts";
import { useShowGroups } from "@/stores/show/show-groups";

import LiveIntensityFlashs from "./LiveIntensityFlashs.vue";

describe("LiveIntensityFlashs.vue", () => {
  it("renders flash controls and groups", () => {
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Front" },
    ];

    render(LiveIntensityFlashs);

    expect(screen.getByRole("heading", { name: "Flashs" })).toBeInTheDocument();
    expect(screen.getByText("All :")).toBeInTheDocument();
    expect(screen.getByText("Front :")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Simple" })).toHaveLength(2);
    expect(screen.getAllByRole("button", { name: "White" })).toHaveLength(2);
    expect(screen.getAllByRole("button", { name: "Strobe" })).toHaveLength(2);
    expect(screen.getAllByRole("button", { name: "Random" })).toHaveLength(2);
  });

  it("updates flash fade and strobe rates via knobs", async () => {
    const user = userEvent.setup();
    const liveButtons = useLiveButtons();

    render(LiveIntensityFlashs);

    const knobs = screen.getAllByRole("slider");
    expect(knobs).toHaveLength(4);

    knobs[0].focus();
    await user.keyboard("{ArrowUp}");
    expect(liveButtons.simpleFlashFade).toBe(0.1);

    knobs[1].focus();
    await user.keyboard("{ArrowUp}");
    expect(liveButtons.whiteFlashFade).toBe(0.1);

    knobs[2].focus();
    await user.keyboard("{ArrowUp}");
    expect(liveButtons.strobeRate).toBe(51);

    knobs[3].focus();
    await user.keyboard("{ArrowUp}");
    expect(liveButtons.randomStrobeRate).toBe(51);
  });

  it("triggers flash on button press for simple, white, strobe, and random", async () => {
    const showGroups = useShowGroups();
    const liveButtons = useLiveButtons();
    showGroups.groups = [{ id: 1, name: "Front" }];

    render(LiveIntensityFlashs);

    const simpleBtn = screen.getByRole("button", { name: "Simple" });
    const whiteBtn = screen.getByRole("button", { name: "White" });
    const strobeBtn = screen.getByRole("button", { name: "Strobe" });
    const randomBtn = screen.getByRole("button", { name: "Random" });

    await fireEvent.mouseDown(simpleBtn);
    expect(liveButtons.isFlashActive(1, "simple")).toBe(true);
    await fireEvent.mouseUp(simpleBtn);
    expect(liveButtons.isFlashActive(1, "simple")).toBe(false);

    await fireEvent.mouseDown(whiteBtn);
    expect(liveButtons.isFlashActive(1, "white")).toBe(true);
    await fireEvent.mouseUp(whiteBtn);
    expect(liveButtons.isFlashActive(1, "white")).toBe(false);

    await fireEvent.mouseDown(strobeBtn);
    expect(liveButtons.isFlashActive(1, "strobe")).toBe(true);
    await fireEvent.mouseUp(strobeBtn);
    expect(liveButtons.isFlashActive(1, "strobe")).toBe(false);

    await fireEvent.mouseDown(randomBtn);
    expect(liveButtons.isFlashActive(1, "randomStrobe")).toBe(true);
    await fireEvent.mouseUp(randomBtn);
    expect(liveButtons.isFlashActive(1, "randomStrobe")).toBe(false);
  });

  it("hides section title when showSectionTitles is false in settings", () => {
    const settings = useSettings();
    settings.live.showSectionTitles = false;

    render(LiveIntensityFlashs);

    expect(screen.queryByRole("heading", { name: "Flashs" })).not.toBeInTheDocument();
  });
});
