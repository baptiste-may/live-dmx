import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import type { PhysicalInterface } from "@/types/output.ts";

import OutputTargetSelect from "./OutputTargetSelect.vue";

describe("OutputTargetSelect.vue", () => {
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
      id: "physical-2",
      name: "Physical 2",
      device: "FTDI Open DMX",
      port: "/dev/ttyUSB1",
      status: "connected",
      fps: 40,
    },
  ];

  it("should render select with options grouped by Physical and Virtual", () => {
    render(OutputTargetSelect, {
      props: {
        modelValue: "none",
        physicalInterfaces: mockInterfaces,
      },
    });

    const select = screen.getByRole("combobox", { name: "Select output destination" });
    expect(select).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "None" })).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "Physical 1 (Enttec DMX USB Pro)" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Physical 2 (FTDI Open DMX)" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "ArtNet" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "sCan (sACN)" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "KiNET" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "OLA" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "OPC" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "OSC" })).toBeInTheDocument();
  });

  it("should emit update:modelValue when selecting an option", async () => {
    const { emitted } = render(OutputTargetSelect, {
      props: {
        modelValue: "none",
        physicalInterfaces: mockInterfaces,
      },
    });

    const select = screen.getByRole("combobox", { name: "Select output destination" });
    await fireEvent.update(select, "physical-1");

    expect(emitted()["update:modelValue"]).toBeTruthy();
    expect(emitted()["update:modelValue"][0]).toEqual(["physical-1"]);
  });

  it("should emit virtual protocol selection correctly", async () => {
    const { emitted } = render(OutputTargetSelect, {
      props: {
        modelValue: "none",
        physicalInterfaces: mockInterfaces,
      },
    });

    const select = screen.getByRole("combobox", { name: "Select output destination" });
    await fireEvent.update(select, "kinet");

    expect(emitted()["update:modelValue"]).toBeTruthy();
    expect(emitted()["update:modelValue"][0]).toEqual(["kinet"]);
  });

  it("should disable select when disabled prop is true", () => {
    render(OutputTargetSelect, {
      props: {
        modelValue: "none",
        physicalInterfaces: mockInterfaces,
        disabled: true,
      },
    });

    const select = screen.getByRole("combobox", { name: "Select output destination" });
    expect(select).toBeDisabled();
  });
});
