import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import OutputUniverseStatus from "./OutputUniverseStatus.vue";

describe("OutputUniverseStatus.vue", () => {
  it("should display sending status when enabled and output is configured", () => {
    render(OutputUniverseStatus, {
      props: {
        universe: 1,
        enabled: true,
        output: "artnet",
      },
    });

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 1: Sending data");
    expect(statusEl).toHaveClass("text-success");
  });

  it("should display disabled status when enabled is false", () => {
    render(OutputUniverseStatus, {
      props: {
        universe: 2,
        enabled: false,
        output: "artnet",
      },
    });

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 2: Output disabled");
    expect(statusEl).toHaveClass("text-base-content/40");
  });

  it("should display idle status when enabled but output is none", () => {
    render(OutputUniverseStatus, {
      props: {
        universe: 3,
        enabled: true,
        output: "none",
      },
    });

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 3: No route assigned");
    expect(statusEl).toHaveClass("text-warning");
  });

  it("should display error status when physical device is disconnected", () => {
    render(OutputUniverseStatus, {
      props: {
        universe: 4,
        enabled: true,
        output: "physical-1",
        physicalStatus: "disconnected",
      },
    });

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 4: Interface disconnected");
    expect(statusEl).toHaveClass("text-error");
  });

  it("should display error status when physical device is missing", () => {
    render(OutputUniverseStatus, {
      props: {
        universe: 5,
        enabled: true,
        output: "physical-99",
        isMissingPhysical: true,
      },
    });

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 5: Interface not found");
    expect(statusEl).toHaveClass("text-error");
  });
});
