import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import type { PhysicalInterface, UniverseOutputConfig } from "@/types/output.ts";

import OutputUniverseRow from "./OutputUniverseRow.vue";

describe("OutputUniverseRow.vue", () => {
  const mockInterfaces: PhysicalInterface[] = [
    {
      id: "physical-1",
      name: "Physical 1",
      device: "Enttec DMX USB Pro",
      port: "/dev/ttyUSB0",
      status: "connected",
      fps: 44,
    },
    {
      id: "physical-disconnected",
      name: "Physical Broken",
      device: "FTDI",
      port: "/dev/ttyUSB9",
      status: "disconnected",
      fps: 0,
    },
  ];

  it("should render universe number, status indicator and physical badge when output is physical and enabled", () => {
    const config: UniverseOutputConfig = {
      universe: 1,
      output: "physical-1",
      enabled: true,
    };

    render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    expect(screen.getByText("Universe 1")).toBeInTheDocument();
    expect(screen.getByText("Physical")).toBeInTheDocument();
    expect(
      screen.getByText(/Outputting via Physical 1 \(\/dev\/ttyUSB0 • Enttec DMX USB Pro\)/),
    ).toBeInTheDocument();

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 1: Sending data");
  });

  it("should render error status and badge when physical interface is disconnected", () => {
    const config: UniverseOutputConfig = {
      universe: 1,
      output: "physical-disconnected",
      enabled: true,
    };

    render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    expect(screen.getByText("Error")).toBeInTheDocument();
    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 1: Interface disconnected");
  });

  it("should render disabled badge and muted description when enabled is false", () => {
    const config: UniverseOutputConfig = {
      universe: 1,
      output: "physical-1",
      enabled: false,
    };

    render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    expect(screen.getByText("Universe 1")).toBeInTheDocument();
    expect(screen.getByText("Disabled")).toBeInTheDocument();
    expect(screen.getByText("Output disabled (muted)")).toBeInTheDocument();

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 1: Output disabled");
  });

  it("should render virtual badge and sending status when output is ArtNet", () => {
    const config: UniverseOutputConfig = {
      universe: 2,
      output: "artnet",
      enabled: true,
    };

    render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    expect(screen.getByText("Universe 2")).toBeInTheDocument();
    expect(screen.getByText("Virtual")).toBeInTheDocument();
    expect(screen.getByText("Broadcasting via Art-Net protocol (Port 6454)")).toBeInTheDocument();

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 2: Sending data");
  });

  it("should render virtual badge when output is KiNET", () => {
    const config: UniverseOutputConfig = {
      universe: 4,
      output: "kinet",
      enabled: true,
    };

    render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    expect(screen.getByText("Universe 4")).toBeInTheDocument();
    expect(screen.getByText("Virtual")).toBeInTheDocument();
    expect(screen.getByText("Philips Color Kinetics protocol")).toBeInTheDocument();
  });

  it("should render unassigned badge when output is none and enabled", () => {
    const config: UniverseOutputConfig = {
      universe: 3,
      output: "none",
      enabled: true,
    };

    render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    expect(screen.getByText("Universe 3")).toBeInTheDocument();
    expect(screen.getByText("Unassigned")).toBeInTheDocument();
    expect(screen.getByText("No output target routed")).toBeInTheDocument();

    const statusEl = screen.getByRole("status");
    expect(statusEl).toHaveAttribute("aria-label", "Universe 3: No route assigned");
  });

  it("should emit update:enabled when toggle is clicked", async () => {
    const config: UniverseOutputConfig = {
      universe: 1,
      output: "none",
      enabled: true,
    };

    const { emitted } = render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    const toggle = screen.getByRole("checkbox", { name: "Toggle universe 1" });
    await fireEvent.click(toggle);

    expect(emitted()["update:enabled"]).toBeTruthy();
    expect(emitted()["update:enabled"][0]).toEqual([false]);
  });

  it("should emit update:output when changing selection", async () => {
    const config: UniverseOutputConfig = {
      universe: 1,
      output: "none",
      enabled: true,
    };

    const { emitted } = render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    const select = screen.getByRole("combobox", { name: "Select output destination" });
    await fireEvent.update(select, "sacn");

    expect(emitted()["update:output"]).toBeTruthy();
    expect(emitted()["update:output"][0]).toEqual(["sacn"]);
  });

  it("should emit remove when remove button is clicked", async () => {
    const config: UniverseOutputConfig = {
      universe: 1,
      output: "none",
      enabled: true,
    };

    const { emitted } = render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: true,
      },
    });

    const removeBtn = screen.getByRole("button", { name: "Remove universe 1" });
    await fireEvent.click(removeBtn);

    expect(emitted()["remove"]).toBeTruthy();
  });

  it("should disable remove button when canRemove is false", () => {
    const config: UniverseOutputConfig = {
      universe: 1,
      output: "none",
      enabled: true,
    };

    render(OutputUniverseRow, {
      props: {
        config,
        physicalInterfaces: mockInterfaces,
        canRemove: false,
      },
    });

    const removeBtn = screen.getByRole("button", { name: "Remove universe 1" });
    expect(removeBtn).toBeDisabled();
  });
});
