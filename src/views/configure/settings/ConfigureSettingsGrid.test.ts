import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { defaultGroupSettings, useShowConfiguration } from "@/stores/show/show-configuration.ts";
import type { ColorPreference, GroupConfigureInfo } from "@/views/configure/types";

import ConfigureSettingsGrid from "./ConfigureSettingsGrid.vue";

function seedOverview(
  hasPan: boolean,
  hasTilt: boolean,
  panBounds: [number, number],
  tiltBounds: [number, number],
): void {
  const configure = useShowConfiguration();
  const info: GroupConfigureInfo = {
    id: 0,
    name: "All",
    fixtureCount: 0,
    hasPan,
    hasTilt,
    hasColor: false,
    panBounds,
    tiltBounds,
    colorOptions: [] as ColorPreference[],
    settings: defaultGroupSettings("Color Mix", panBounds, tiltBounds),
  };
  configure.overview = {
    rows: [{ id: 0, name: "All", fixtureCount: 0, children: [] }],
    groups: { 0: info },
  };
}

describe("ConfigureSettingsGrid.vue", () => {
  it("does not render angle markers when group has no pan or tilt", () => {
    seedOverview(false, false, [-180, 180], [-110, 110]);
    render(ConfigureSettingsGrid);

    expect(screen.queryByText("-180°")).not.toBeInTheDocument();
    expect(screen.queryByText("180°")).not.toBeInTheDocument();
    expect(screen.queryByText("-110°")).not.toBeInTheDocument();
    expect(screen.queryByText("110°")).not.toBeInTheDocument();
  });

  it("renders only pan angle markers when group has only pan", () => {
    seedOverview(true, false, [-180, 180], [-110, 110]);
    render(ConfigureSettingsGrid);

    expect(screen.getByText("-180°")).toBeInTheDocument();
    expect(screen.getByText("180°")).toBeInTheDocument();
    expect(screen.queryByText("-110°")).not.toBeInTheDocument();
    expect(screen.queryByText("110°")).not.toBeInTheDocument();
  });

  it("renders dynamic pan and tilt angle markers according to fixture bounds", () => {
    seedOverview(true, true, [-270, 270], [-135, 135]);
    render(ConfigureSettingsGrid);

    expect(screen.getByText("-270°")).toBeInTheDocument();
    expect(screen.getByText("270°")).toBeInTheDocument();
    expect(screen.getByText("-135°")).toBeInTheDocument();
    expect(screen.getByText("135°")).toBeInTheDocument();
  });
});
