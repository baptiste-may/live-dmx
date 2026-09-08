import { fireEvent, render, screen } from "@testing-library/vue";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useOutputDmx } from "@/stores/output/output-dmx.ts";
import { useOutputUniverses } from "@/stores/output/output-universes.ts";

import OutputDmxMonitor from "./OutputDmxMonitor.vue";

describe("OutputDmxMonitor.vue", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("should render heading without active channels badge", () => {
    render(OutputDmxMonitor);

    expect(screen.getByRole("heading", { name: "DMX Output" })).toBeInTheDocument();
    expect(screen.queryByText(/active/i)).not.toBeInTheDocument();
  });

  it("should render universe cards for each configured universe", () => {
    const universesStore = useOutputUniverses();
    universesStore.universes = [
      { universe: 1, output: "artnet", enabled: true },
      { universe: 2, output: "sacn", enabled: true },
    ];

    render(OutputDmxMonitor);

    expect(screen.getByText("Universe 1")).toBeInTheDocument();
    expect(screen.getByText("Universe 2")).toBeInTheDocument();
  });

  it("should render empty state when no universes exist", () => {
    const universesStore = useOutputUniverses();
    universesStore.universes = [];

    render(OutputDmxMonitor);

    expect(screen.getByText("No universes configured")).toBeInTheDocument();
  });

  it("should toggle pause state when pause button is clicked", async () => {
    const dmxStore = useOutputDmx();
    render(OutputDmxMonitor);

    const pauseBtn = screen.getByRole("button", { name: "Pause live update" });
    await fireEvent.click(pauseBtn);
    expect(dmxStore.isPaused).toBe(true);

    const resumeBtn = screen.getByRole("button", { name: "Resume live update" });
    await fireEvent.click(resumeBtn);
    expect(dmxStore.isPaused).toBe(false);
  });

  it("should poll fetchAllDmxData periodically", () => {
    const dmxStore = useOutputDmx();
    const fetchSpy = vi.spyOn(dmxStore, "fetchAllDmxData");

    const { unmount } = render(OutputDmxMonitor);
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(250);
    expect(fetchSpy).toHaveBeenCalledTimes(3);

    unmount();
    vi.advanceTimersByTime(200);
    expect(fetchSpy).toHaveBeenCalledTimes(3);
  });
});
