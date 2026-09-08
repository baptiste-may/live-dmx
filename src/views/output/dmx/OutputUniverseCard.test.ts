import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import OutputUniverseCard from "./OutputUniverseCard.vue";

describe("OutputUniverseCard.vue", () => {
  it("should render universe number and enabled protocol label", () => {
    const channels = Array.from({ length: 512 }, () => 0);
    render(OutputUniverseCard, {
      props: {
        universe: 1,
        output: "artnet",
        enabled: true,
        channels,
      },
    });

    expect(screen.getByText("Universe 1")).toBeInTheDocument();
    expect(screen.getByText("Art-Net")).toBeInTheDocument();
    expect(screen.queryByText(/active/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/512 channels/i)).not.toBeInTheDocument();
  });

  it("should render Disabled when universe is disabled", () => {
    const channels = Array.from({ length: 512 }, () => 0);
    render(OutputUniverseCard, {
      props: {
        universe: 2,
        output: "artnet",
        enabled: false,
        channels,
      },
    });

    expect(screen.getByText("Universe 2")).toBeInTheDocument();
    expect(screen.getByText("Disabled")).toBeInTheDocument();
  });

  it("should render 512 channel dots with active styling", () => {
    const channels = Array.from({ length: 512 }, () => 0);
    channels[0] = 255;
    channels[9] = 128;

    const { container } = render(OutputUniverseCard, {
      props: {
        universe: 3,
        output: "sacn",
        enabled: true,
        channels,
      },
    });

    const dots = container.querySelectorAll(".h-full.w-full");
    expect(dots.length).toBe(512);

    expect(dots[0].getAttribute("title")).toBe("CH 1: 255");
    expect(dots[0].className).toContain("bg-primary");
    expect(dots[1].getAttribute("title")).toBe("CH 2: 0");
    expect(dots[1].className).toContain("bg-base-content/15");
  });
});
