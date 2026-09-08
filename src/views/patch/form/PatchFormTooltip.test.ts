import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

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
});
