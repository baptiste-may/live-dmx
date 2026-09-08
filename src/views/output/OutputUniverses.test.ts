import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useOutputInterfaces } from "@/stores/output/output-interfaces.ts";
import { useOutputUniverses } from "@/stores/output/output-universes.ts";

import OutputUniverses from "./OutputUniverses.vue";

describe("OutputUniverses.vue", () => {
  it("should render universes title and error/warning badges", () => {
    const universesStore = useOutputUniverses();
    universesStore.universes = [{ universe: 1, output: "artnet", enabled: true }];
    render(OutputUniverses);

    expect(screen.getByRole("heading", { name: "Universes" })).toBeInTheDocument();
    expect(screen.getByText("0 errors")).toBeInTheDocument();
    expect(screen.getByText("0 warnings")).toBeInTheDocument();
  });

  it("should display warning badge when an enabled universe has output none", () => {
    const store = useOutputUniverses();
    store.universes = [{ universe: 1, output: "none", enabled: true }];

    render(OutputUniverses);

    expect(screen.getByText("1 warning")).toBeInTheDocument();
  });

  it("should display error badge when an enabled universe uses a disconnected interface", () => {
    const universesStore = useOutputUniverses();
    const interfacesStore = useOutputInterfaces();

    universesStore.universes = [{ universe: 1, output: "physical-1", enabled: true }];
    interfacesStore.interfaces = [
      {
        id: "physical-1",
        name: "Physical 1",
        device: "Broken Device",
        port: "/dev/ttyUSB0",
        status: "disconnected",
        fps: 0,
      },
    ];

    render(OutputUniverses);

    expect(screen.getByText("1 error")).toBeInTheDocument();
  });

  it("should display error badge when an enabled universe uses a non-existent physical interface", () => {
    const universesStore = useOutputUniverses();
    const interfacesStore = useOutputInterfaces();

    universesStore.universes = [{ universe: 1, output: "physical-non-existent", enabled: true }];
    interfacesStore.interfaces = [];

    render(OutputUniverses);

    expect(screen.getByText("1 error")).toBeInTheDocument();
  });

  it("should render all universes rows", () => {
    const store = useOutputUniverses();
    render(OutputUniverses);

    for (const u of store.universes) {
      expect(screen.getByText(`Universe ${u.universe}`)).toBeInTheDocument();
    }
  });

  it("should call store.addUniverse when clicking Add Universe button at the bottom", async () => {
    const store = useOutputUniverses();
    const initialCount = store.universes.length;

    render(OutputUniverses);

    const addBtn = screen.getByRole("button", { name: "Add Universe" });
    await fireEvent.click(addBtn);

    expect(store.universes.length).toBe(initialCount + 1);
  });

  it("should toggle universe enabled when toggle is clicked", async () => {
    const store = useOutputUniverses();
    render(OutputUniverses);

    const toggle = screen.getByRole("checkbox", { name: "Toggle universe 1" });
    expect(store.universes[0].enabled).toBe(true);

    await fireEvent.click(toggle);
    expect(store.universes[0].enabled).toBe(false);
  });

  it("should render empty state when universes array is empty", () => {
    const store = useOutputUniverses();
    store.universes = [];

    render(OutputUniverses);

    expect(screen.getByText("No universes configured")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add First Universe" })).toBeInTheDocument();
  });
});
