import { render, screen } from "@testing-library/vue";
import { describe, expect, it, vi } from "vitest";

import { useFixtureBrowser } from "@/stores/fixture-browser";
import { mockTooltipFixture } from "@/views/patch/__mocks__/fixtures";

import PatchFormTooltip from "./PatchFormTooltip.vue";

describe("PatchFormTooltip.vue", () => {
  it("should render fixture details, mode, and channel list", () => {
    const browser = useFixtureBrowser();
    browser.manufacturers = {
      cameo: {
        name: "Cameo",
        fixtures: {},
      },
    };

    render(PatchFormTooltip, {
      props: {
        fixture: mockTooltipFixture,
        mode: 0,
      },
    });

    expect(screen.getByText("Cameo")).toBeInTheDocument();
    expect(screen.getByText(/Mini Spot/)).toBeInTheDocument();
    expect(screen.getByText("Spot")).toBeInTheDocument();
    expect(screen.getByText(/7 channels/)).toBeInTheDocument();
    expect(screen.getByText("Dimmer")).toBeInTheDocument();
  });

  it("should render revision and uploader if provided", () => {
    render(PatchFormTooltip, {
      props: {
        fixture: {
          ...mockTooltipFixture,
          revision: "Official Fw 2.5",
          uploader: "Manuf.",
        },
        mode: 0,
      },
    });

    expect(screen.getByText("Revision: Official Fw 2.5")).toBeInTheDocument();
    expect(screen.getByText("Manuf.")).toBeInTheDocument();
  });

  it("should render placeholder message for uninstalled GDTF fixtures", () => {
    render(PatchFormTooltip, {
      props: {
        fixture: {
          ...mockTooltipFixture,
          rid: 12345,
          isInstalled: false,
          modes: [
            {
              name: "Standard",
              dmxfootprint: 16,
              channels: Array.from({ length: 16 }, (_, i) => `Channel ${i + 1}`),
            },
          ],
        },
        mode: 0,
      },
    });

    expect(screen.getByText("16 DMX Channels")).toBeInTheDocument();
    expect(
      screen.getByText(/Complete channel names and attributes will be downloaded/),
    ).toBeInTheDocument();
  });

  it("should treat empty channel strings and nameless channel objects as placeholder", () => {
    render(PatchFormTooltip, {
      props: {
        fixture: {
          ...mockTooltipFixture,
          rid: 54321,
          isInstalled: false,
          modes: [
            {
              name: "EmptyNames",
              dmxfootprint: 2,
              channels: ["", { name: "" } as unknown as string],
            },
          ],
        },
        mode: 0,
      },
    });

    expect(screen.getByText("2 DMX Channels")).toBeInTheDocument();
    expect(
      screen.getByText(/Complete channel names and attributes will be downloaded/),
    ).toBeInTheDocument();
  });

  it("should use dmxfootprint when channels array is empty", () => {
    render(PatchFormTooltip, {
      props: {
        fixture: {
          ...mockTooltipFixture,
          rid: 99999,
          isInstalled: false,
          modes: [
            {
              name: "EmptyArray",
              dmxfootprint: 24,
              channels: [],
            },
          ],
        },
        mode: 0,
      },
    });

    expect(screen.getByText("24 channels")).toBeInTheDocument();
    expect(screen.getByText("24 DMX Channels")).toBeInTheDocument();
  });

  it("should trigger preview fetch and render real channels after debounce", async () => {
    vi.useFakeTimers();
    const browser = useFixtureBrowser();
    const fetchSpy = vi.spyOn(browser, "fetchMode").mockImplementation(async (f, m) => {
      const mode = f.modes.find((x) => x.name === m);
      if (mode) {
        mode.channels = ["Pan", "Tilt"];
      }
      return {
        name: m,
        channelCount: 2,
        channels: ["Pan", "Tilt"],
      };
    });

    const fixture = {
      ...mockTooltipFixture,
      rid: 8888,
      isInstalled: false,
      modes: [
        {
          name: "Standard",
          dmxfootprint: 2,
          channels: ["Channel 1", "Channel 2"],
        },
      ],
    };

    render(PatchFormTooltip, {
      props: {
        fixture,
        mode: 0,
      },
    });

    await vi.advanceTimersByTimeAsync(300);

    expect(fetchSpy).toHaveBeenCalledWith(fixture, "Standard", { persist: false });
    expect(screen.getByText("Pan")).toBeInTheDocument();
    expect(screen.getByText("Tilt")).toBeInTheDocument();
    vi.useRealTimers();
  });

  it("should not retry preview fetch if it previously failed for fixture/mode", async () => {
    vi.useFakeTimers();
    const browser = useFixtureBrowser();
    const fetchSpy = vi.spyOn(browser, "fetchMode").mockRejectedValue(new Error("Network failure"));

    const fixture = {
      ...mockTooltipFixture,
      rid: 7777,
      fixtureKey: "failed-fix",
      isInstalled: false,
      modes: [
        {
          name: "Standard",
          dmxfootprint: 2,
          channels: ["Channel 1", "Channel 2"],
        },
      ],
    };

    const { unmount } = render(PatchFormTooltip, {
      props: {
        fixture,
        mode: 0,
      },
    });

    await vi.advanceTimersByTimeAsync(300);
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    unmount();

    render(PatchFormTooltip, {
      props: {
        fixture,
        mode: 0,
      },
    });

    await vi.advanceTimersByTimeAsync(300);
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    vi.useRealTimers();
  });
});
