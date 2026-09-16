import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useShowConfiguration } from "@/stores/show/show-configuration.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import type { ConfigureOverview, GroupConfigureInfo } from "@/views/configure/types";

import ConfigureSelection from "./ConfigureSelection.vue";

function buildOverview(): ConfigureOverview {
  const groupInfo = (id: number, name: string): GroupConfigureInfo => ({
    id,
    name,
    fixtureCount: id === 0 ? 1 : 1,
    hasPan: true,
    hasTilt: false,
    hasColor: false,
    panBounds: [-180, 180] as [number, number],
    tiltBounds: [-110, 110] as [number, number],
    colorOptions: [],
    settings: {
      colorPreference: "Color Mix",
      panMin: -180,
      panMax: 180,
      tiltMin: -110,
      tiltMax: 110,
      inversePan: false,
      inverseTilt: false,
    },
  });
  return {
    rows: [
      { id: 0, name: "All", fixtureCount: 1, children: [] },
      { id: 1, name: "Moving Heads", fixtureCount: 1, children: [] },
    ],
    groups: {
      0: groupInfo(0, "All"),
      1: groupInfo(1, "Moving Heads"),
    },
  };
}

describe("ConfigureSelection.vue", () => {
  it("renders groups and emits the selected group", async () => {
    const user = userEvent.setup();
    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Moving Heads" },
    ];
    configure.overview = buildOverview();
    render(ConfigureSelection);

    expect(screen.getByRole("button", { name: /All1/ })).toHaveAttribute("aria-pressed", "true");
    await user.click(screen.getByRole("button", { name: /Moving Heads1/ }));

    expect(screen.getByRole("button", { name: /Moving Heads1/ })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });
});
