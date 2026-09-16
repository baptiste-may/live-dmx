import userEvent from "@testing-library/user-event";
import { render, screen } from "@testing-library/vue";
import { describe, expect, it } from "vitest";

import { useShowConfiguration } from "@/stores/show/show-configuration.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import type { ConfigureOverview, GroupConfigureInfo } from "@/views/configure/types";

import ConfigureSelectionMenuItem from "./ConfigureSelectionMenuItem.vue";

function buildOverview(): ConfigureOverview {
  const groupInfo = (id: number, name: string): GroupConfigureInfo => ({
    id,
    name,
    fixtureCount: 1,
    hasPan: false,
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
      {
        id: 0,
        name: "All",
        fixtureCount: 2,
        children: [
          {
            id: 1,
            name: "Group 1",
            fixtureCount: 2,
            children: [{ id: 2, name: "Subgroup 1", fixtureCount: 1, children: [] }],
          },
        ],
      },
    ],
    groups: {
      0: groupInfo(0, "All"),
      1: groupInfo(1, "Group 1"),
      2: groupInfo(2, "Subgroup 1"),
    },
  };
}

describe("ConfigureSelectionMenuItem.vue", () => {
  it("renders nested DaisyUI menu items and emits selection", async () => {
    const user = userEvent.setup();
    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Group 1" },
      { id: 2, name: "Subgroup 1" },
    ];
    configure.overview = buildOverview();

    render(ConfigureSelectionMenuItem, {
      props: {
        groupId: 1,
      },
    });

    expect(screen.getByRole("list")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Subgroup 1/ }));

    expect(screen.getByRole("button", { name: /Subgroup 1/ })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });
});
