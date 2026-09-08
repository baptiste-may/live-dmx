import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it, vi } from "vitest";

import {
  DEFAULT_PHYSICAL_INTERFACES,
  useOutputInterfaces,
} from "@/stores/output/output-interfaces.ts";

import OutputInterfaces from "./OutputInterfaces.vue";

describe("OutputInterfaces.vue", () => {
  it("should render heading and count badge", () => {
    const store = useOutputInterfaces();
    store.interfaces = DEFAULT_PHYSICAL_INTERFACES;
    render(OutputInterfaces);

    expect(screen.getByRole("heading", { name: "Physical Interfaces" })).toBeInTheDocument();
    expect(screen.getByText(`${store.interfaces.length} detected`)).toBeInTheDocument();
  });

  it("should render all physical interface items", () => {
    const store = useOutputInterfaces();
    store.interfaces = DEFAULT_PHYSICAL_INTERFACES;
    render(OutputInterfaces);

    for (const item of store.interfaces) {
      expect(screen.getByText(item.name)).toBeInTheDocument();
      expect(screen.getByText(item.device)).toBeInTheDocument();
    }
  });

  it("should call store.scanInterfaces when clicking rescan button", async () => {
    const store = useOutputInterfaces();
    const scanSpy = vi.spyOn(store, "scanInterfaces");

    render(OutputInterfaces);

    const rescanBtn = screen.getByRole("button", { name: "Rescan physical interfaces" });
    await fireEvent.click(rescanBtn);

    expect(scanSpy).toHaveBeenCalled();
  });

  it("should display empty state when no interfaces are present", () => {
    const store = useOutputInterfaces();
    store.interfaces = [];

    render(OutputInterfaces);

    expect(screen.getByText("No physical interfaces detected")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Rescan Devices" })).toBeInTheDocument();
  });
});
