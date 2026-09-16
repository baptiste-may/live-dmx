import { fireEvent, render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";
import { nextTick } from "vue";

import { defaultGroupSettings, useShowConfiguration } from "@/stores/show/show-configuration.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import type { ColorPreference, GroupConfigureInfo } from "@/views/configure/types";

import ConfigureSettings from "./ConfigureSettings.vue";

type InfoOverrides = Partial<GroupConfigureInfo>;

function seedOverview(
  configure: ReturnType<typeof useShowConfiguration>,
  overrides: InfoOverrides = {},
): void {
  const info = (id: number, name: string): GroupConfigureInfo => ({
    id,
    name,
    fixtureCount: 0,
    hasPan: false,
    hasTilt: false,
    hasColor: false,
    panBounds: [-180, 180],
    tiltBounds: [-110, 110],
    colorOptions: [] as ColorPreference[],
    settings: defaultGroupSettings(),
    ...overrides,
  });
  configure.overview = {
    rows: [{ id: 0, name: "All", fixtureCount: 0, children: [] }],
    groups: { 0: info(0, "All") },
  };
}

describe("ConfigureSettings.vue", () => {
  it("renders capability-specific settings", () => {
    const configure = useShowConfiguration();
    seedOverview(configure, {
      hasPan: true,
      hasTilt: false,
      hasColor: true,
      settings: defaultGroupSettings("Color Mix"),
    });
    configure.settingsByGroup[0] = {
      colorPreference: "Color Mix",
      panMin: -180,
      panMax: 180,
      tiltMin: -110,
      tiltMax: 110,
      inversePan: false,
      inverseTilt: false,
    };
    render(ConfigureSettings);

    expect(screen.getByText("All")).toBeInTheDocument();
    expect(screen.getByLabelText("Color preference")).toBeInTheDocument();
    expect(screen.getByLabelText("Pan range")).toBeInTheDocument();
    expect(screen.queryByLabelText("Tilt range")).not.toBeInTheDocument();
  });

  it("locks the color preference select to None without color capability", () => {
    const configure = useShowConfiguration();
    seedOverview(configure);
    render(ConfigureSettings);

    const select = screen.getByLabelText("Color preference");
    expect(select).toBeDisabled();
    expect(select).toHaveValue("None");
  });

  it("allows editing numeric pan/tilt bounds and toggles", async () => {
    const configure = useShowConfiguration();
    seedOverview(configure, {
      hasPan: true,
      hasTilt: true,
      hasColor: true,
      settings: defaultGroupSettings("Color Mix"),
    });
    configure.settingsByGroup[0] = {
      colorPreference: "Color Mix",
      panMin: -180,
      panMax: 180,
      tiltMin: -110,
      tiltMax: 110,
      inversePan: false,
      inverseTilt: false,
    };
    render(ConfigureSettings);

    const numberInputs = screen.getAllByRole("spinbutton");
    expect(numberInputs).toHaveLength(4);
    expect(screen.getByLabelText("Pan minimum")).toBeInTheDocument();
    expect(screen.getByLabelText("Pan maximum")).toBeInTheDocument();
    expect(screen.getByLabelText("Tilt minimum")).toBeInTheDocument();
    expect(screen.getByLabelText("Tilt maximum")).toBeInTheDocument();

    await fireEvent.update(screen.getByLabelText("Pan minimum"), "-90");
    expect(configure.selectedSettings.panMin).toBe(-90);

    await fireEvent.update(screen.getByLabelText("Pan maximum"), "90");
    expect(configure.selectedSettings.panMax).toBe(90);

    const inversePanToggle = screen.getByRole("checkbox", { name: "Inverse Pan" });
    await fireEvent.click(inversePanToggle);
    expect(configure.selectedSettings.inversePan).toBe(true);
  });

  it("binds dynamic min and max on inputs according to fixture bounds", () => {
    const configure = useShowConfiguration();
    seedOverview(configure, {
      hasPan: true,
      hasTilt: true,
      panBounds: [-270, 270],
      tiltBounds: [-135, 135],
      settings: defaultGroupSettings("Color Mix", [-270, 270], [-135, 135]),
    });
    render(ConfigureSettings);

    const numberInputs = screen.getAllByRole("spinbutton");
    expect(numberInputs[0]).toHaveAttribute("min", "-270");
    expect(numberInputs[0]).toHaveAttribute("max", "270");
    expect(numberInputs[1]).toHaveAttribute("min", "-270");
    expect(numberInputs[1]).toHaveAttribute("max", "270");
    expect(numberInputs[2]).toHaveAttribute("min", "-135");
    expect(numberInputs[2]).toHaveAttribute("max", "135");
    expect(numberInputs[3]).toHaveAttribute("min", "-135");
    expect(numberInputs[3]).toHaveAttribute("max", "135");
  });

  it("clamps inputs to group bounds and enforces min <= max ordering", async () => {
    const configure = useShowConfiguration();
    seedOverview(configure, {
      hasPan: true,
      hasTilt: true,
      settings: defaultGroupSettings("Color Mix"),
    });
    configure.settingsByGroup[0] = {
      colorPreference: "Color Mix",
      panMin: -50,
      panMax: 50,
      tiltMin: -30,
      tiltMax: 30,
      inversePan: false,
      inverseTilt: false,
    };
    render(ConfigureSettings);

    const numberInputs = screen.getAllByRole("spinbutton");

    // Try setting panMin below -180
    await fireEvent.update(numberInputs[0]!, "-300");
    expect(configure.selectedSettings.panMin).toBe(-180);

    // Try setting panMax above 180
    await fireEvent.update(numberInputs[1]!, "300");
    expect(configure.selectedSettings.panMax).toBe(180);

    // Setting panMin higher than panMax should push panMax
    await fireEvent.update(numberInputs[1]!, "50");
    await fireEvent.update(numberInputs[0]!, "100");
    expect(configure.selectedSettings.panMin).toBe(100);
    expect(configure.selectedSettings.panMax).toBe(100);

    await fireEvent.update(numberInputs[0]!, "150");
    await fireEvent.update(numberInputs[1]!, "50");
    expect(configure.selectedSettings.panMax).toBe(50);
    expect(configure.selectedSettings.panMin).toBe(50);
  });

  it("positions tilt range correctly centered without upward translate offset when tilt-only", () => {
    const configure = useShowConfiguration();
    seedOverview(configure, {
      hasTilt: true,
      tiltBounds: [-92, 92],
      settings: defaultGroupSettings("Color Mix", [-180, 180], [-92, 92]),
    });
    render(ConfigureSettings);

    expect(screen.queryByLabelText("Pan range")).not.toBeInTheDocument();
    const tiltRange = screen.getByLabelText("Tilt range");
    expect(tiltRange).toBeInTheDocument();
    const tiltWrapper = tiltRange.parentElement;
    expect(tiltWrapper?.className).toContain("absolute");
    expect(tiltWrapper?.className).toContain("-translate-y-1/2");
    expect(tiltWrapper?.className).not.toContain("-translate-y-[150%]");
  });

  it("centers both pan and tilt ranges independently in absolute containers when both are present", () => {
    const configure = useShowConfiguration();
    seedOverview(configure, {
      hasPan: true,
      hasTilt: true,
      panBounds: [-270, 270],
      tiltBounds: [-135, 135],
      settings: defaultGroupSettings("Color Mix", [-270, 270], [-135, 135]),
    });
    render(ConfigureSettings);

    const panRange = screen.getByLabelText("Pan range");
    const tiltRange = screen.getByLabelText("Tilt range");
    expect(panRange).toBeInTheDocument();
    expect(tiltRange).toBeInTheDocument();

    const panWrapper = panRange.parentElement;
    const tiltWrapper = tiltRange.parentElement;

    expect(panWrapper?.className).toContain("absolute");
    expect(panWrapper?.className).toContain("top-1/2");
    expect(panWrapper?.className).toContain("-translate-y-1/2");

    expect(tiltWrapper?.className).toContain("absolute");
    expect(tiltWrapper?.className).toContain("top-1/2");
    expect(tiltWrapper?.className).toContain("-translate-y-1/2");
    expect(tiltWrapper?.className).toContain("-rotate-90");
  });

  it("updates range values and input bounds when switching between groups with different limits", async () => {
    const configure = useShowConfiguration();
    const showGroups = useShowGroups();

    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "JDC2 Group" },
      { id: 2, name: "Spot Group" },
    ];
    configure.overview = {
      rows: [
        { id: 0, name: "All", fixtureCount: 2, children: [] },
        { id: 1, name: "JDC2 Group", fixtureCount: 1, children: [] },
        { id: 2, name: "Spot Group", fixtureCount: 1, children: [] },
      ],
      groups: {
        0: {
          id: 0,
          name: "All",
          fixtureCount: 2,
          hasPan: true,
          hasTilt: true,
          hasColor: true,
          panBounds: [-270, 270],
          tiltBounds: [-104, 104],
          colorOptions: [] as ColorPreference[],
          settings: defaultGroupSettings("Color Mix", [-270, 270], [-104, 104]),
        },
        1: {
          id: 1,
          name: "JDC2 Group",
          fixtureCount: 1,
          hasPan: false,
          hasTilt: true,
          hasColor: true,
          panBounds: [-180, 180],
          tiltBounds: [-92, 92],
          colorOptions: [] as ColorPreference[],
          settings: defaultGroupSettings("Color Mix", [-180, 180], [-92, 92]),
        },
        2: {
          id: 2,
          name: "Spot Group",
          fixtureCount: 1,
          hasPan: true,
          hasTilt: true,
          hasColor: true,
          panBounds: [-270, 270],
          tiltBounds: [-104, 104],
          colorOptions: [] as ColorPreference[],
          settings: defaultGroupSettings("Color Mix", [-270, 270], [-104, 104]),
        },
      },
    };

    configure.selectGroup(1);
    render(ConfigureSettings);

    // Group 1 has tilt only, so Pan inputs are disabled
    expect(screen.getByLabelText("Tilt minimum")).toHaveValue(-92);
    expect(screen.getByLabelText("Tilt maximum")).toHaveValue(92);

    configure.selectGroup(2);
    await nextTick();

    expect(screen.getByLabelText("Tilt minimum")).toHaveValue(-104);
    expect(screen.getByLabelText("Tilt maximum")).toHaveValue(104);
    expect(screen.getByLabelText("Pan minimum")).toHaveValue(-270);
    expect(screen.getByLabelText("Pan maximum")).toHaveValue(270);
  });
});
