import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import type { PhysicalInterface } from "@/types/output.ts";

import OutputInterfaceCard from "./OutputInterfaceCard.vue";

describe("OutputInterfaceCard.vue", () => {
  const baseItem: PhysicalInterface = {
    id: "physical-1",
    name: "Physical 1",
    device: "Enttec DMX USB Pro",
    port: "/dev/ttyUSB0",
    status: "connected",
    fps: 44,
  };

  it("should render interface name, device, port, and fps", () => {
    render(OutputInterfaceCard, {
      props: {
        item: baseItem,
        assignedUniverses: [1],
      },
    });

    expect(screen.getByText("Physical 1")).toBeInTheDocument();
    expect(screen.getByText("Enttec DMX USB Pro")).toBeInTheDocument();
    expect(screen.getByText("/dev/ttyUSB0")).toBeInTheDocument();
    expect(screen.getByText("44 FPS")).toBeInTheDocument();
    expect(screen.getByText("Routing Universe 1")).toBeInTheDocument();
    expect(screen.getByText("Connected")).toBeInTheDocument();
  });

  it("should show disconnected status badge when disconnected", () => {
    render(OutputInterfaceCard, {
      props: {
        item: { ...baseItem, status: "disconnected" },
      },
    });

    expect(screen.getByText("Disconnected")).toBeInTheDocument();
    expect(screen.getByText("Not assigned to any universe")).toBeInTheDocument();
  });

  it("should show busy status badge when busy", () => {
    render(OutputInterfaceCard, {
      props: {
        item: { ...baseItem, status: "busy" },
      },
    });

    expect(screen.getByText("Busy")).toBeInTheDocument();
  });

  it("should format multiple assigned universes properly", () => {
    render(OutputInterfaceCard, {
      props: {
        item: baseItem,
        assignedUniverses: [1, 2],
      },
    });

    expect(screen.getByText("Routing Universe 1, 2")).toBeInTheDocument();
  });
});
