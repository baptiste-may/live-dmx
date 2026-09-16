import { invoke } from "@tauri-apps/api/core";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import { useShowConfigurationSync } from "@/libs/show-configuration-sync.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import { useShowPatch } from "@/stores/show/show-patch.ts";
import type { PatchedFixture } from "@/types/patch";
import type {
  ColorPreference,
  ConfigureOverview,
  GroupConfigureInfo,
  GroupRow,
} from "@/views/configure/types";

import { defaultGroupSettings, useShowConfiguration } from "./show-configuration";

const mockInvoke = vi.mocked(invoke);

function makeGroupInfo(
  id: number,
  name: string,
  overrides: Partial<GroupConfigureInfo> = {},
): GroupConfigureInfo {
  return {
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
  };
}

function makeOverview(
  rows: GroupRow[],
  groups: Record<number, GroupConfigureInfo>,
): ConfigureOverview {
  return { rows, groups };
}

describe("show-configuration store", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
    setActivePinia(createPinia());
  });

  it("debounces scheduleSave and only persists the latest settings", async () => {
    vi.useFakeTimers();
    mockInvoke.mockImplementation(async () => ({}));

    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
    ];
    configure.selectGroup(1);

    configure.scheduleSave(1);
    configure.selectedSettings.panMin = -90;
    vi.advanceTimersByTime(100);
    configure.scheduleSave(1);
    configure.selectedSettings.panMax = 90;

    await vi.advanceTimersByTimeAsync(250);

    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("show_group_update_settings", {
      groupId: 1,
      settings: expect.objectContaining({ panMin: -90, panMax: 90 }),
    });
    expect(showGroups.settings[1]?.panMin).toBe(-90);
    expect(showGroups.settings[1]?.panMax).toBe(90);
  });

  it("flushPendingSave persists pending settings immediately and cancels the timer", async () => {
    vi.useFakeTimers();
    mockInvoke.mockImplementation(async () => ({}));

    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
    ];
    configure.selectGroup(1);
    configure.selectedSettings.inversePan = true;

    configure.scheduleSave(1);
    configure.flushPendingSave();

    expect(configure.saveTimeout).toBeNull();
    expect(configure.pendingGroupId).toBeNull();
    expect(mockInvoke).toHaveBeenCalledTimes(1);

    await Promise.resolve();
    await Promise.resolve();
    expect(showGroups.settings[1]?.inversePan).toBe(true);

    vi.advanceTimersByTime(1000);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });

  it("selectGroup flushes the pending save before switching groups", async () => {
    vi.useFakeTimers();
    mockInvoke.mockImplementation(async () => ({}));

    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
      { id: 2, name: "Washers" },
    ];
    configure.selectGroup(1);
    configure.selectedSettings.tiltMin = -45;
    configure.scheduleSave(1);

    configure.selectGroup(2);

    expect(configure.selectedGroupId).toBe(2);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("show_group_update_settings", {
      groupId: 1,
      settings: expect.objectContaining({ tiltMin: -45 }),
    });

    await Promise.resolve();
    await Promise.resolve();
    expect(showGroups.settings[1]?.tiltMin).toBe(-45);

    vi.advanceTimersByTime(1000);
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });

  it("reads group capabilities from the backend overview", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "show_configure_get_overview") {
        return Promise.resolve(
          makeOverview([{ id: 0, name: "All", fixtureCount: 1, children: [] }], {
            0: makeGroupInfo(0, "All", {
              fixtureCount: 1,
              hasPan: true,
              hasTilt: false,
              hasColor: true,
              panBounds: [-270, 270],
              tiltBounds: [-110, 110],
              colorOptions: ["Color Mix", "Color Wheel"],
              settings: defaultGroupSettings("Color Mix", [-270, 270], [-110, 110]),
            }),
          }),
        );
      }
      return Promise.resolve({});
    });

    const configure = useShowConfiguration();
    await configure.fetchOverview();

    expect(configure.hasPan).toBe(true);
    expect(configure.hasTilt).toBe(false);
    expect(configure.hasColor).toBe(true);
    expect(configure.colorOptions).toEqual(["Color Mix", "Color Wheel"]);
    expect(configure.groupPanBounds).toEqual([-270, 270]);
    expect(configure.groupTiltBounds).toEqual([-110, 110]);
    expect(configure.groupRows).toEqual([{ id: 0, name: "All", fixtureCount: 1, children: [] }]);
  });

  it("keeps settings per group from the backend overview and allows editing", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "show_configure_get_overview") {
        return Promise.resolve(
          makeOverview(
            [
              { id: 0, name: "All", fixtureCount: 1, children: [] },
              { id: 1, name: "Wheel", fixtureCount: 1, children: [] },
              { id: 2, name: "CCT", fixtureCount: 1, children: [] },
            ],
            {
              0: makeGroupInfo(0, "All"),
              1: makeGroupInfo(1, "Wheel", {
                hasColor: true,
                colorOptions: ["Color Wheel"],
                settings: defaultGroupSettings("Color Wheel", [-180, 180], [-110, 110]),
              }),
              2: makeGroupInfo(2, "CCT", {
                hasColor: true,
                colorOptions: ["CCT"],
                settings: defaultGroupSettings("CCT", [-180, 180], [-110, 110]),
              }),
            },
          ),
        );
      }
      return Promise.resolve({});
    });

    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Wheel" },
      { id: 2, name: "CCT" },
    ];
    await configure.fetchOverview();

    configure.selectGroup(1);
    configure.selectedSettings.panMin = 25;
    configure.selectedSettings.colorPreference = "Color Wheel";
    configure.selectGroup(2);

    expect(configure.selectedGroupId).toBe(2);
    expect(configure.selectedSettings.colorPreference).toBe("CCT");
    expect(configure.settingsByGroup[1]?.panMin).toBe(25);

    configure.selectGroup(1);
    expect(configure.selectedSettings.colorPreference).toBe("Color Wheel");
    expect(configure.selectedSettings.panMin).toBe(25);
  });

  it("persists group settings to backend via saveGroupSettings", async () => {
    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Spots" },
    ];

    configure.selectGroup(1);
    configure.selectedSettings.panMin = -90;
    configure.selectedSettings.panMax = 90;
    configure.selectedSettings.inversePan = true;

    await configure.saveGroupSettings(1);

    expect(mockInvoke).toHaveBeenCalledWith("show_group_update_settings", {
      groupId: 1,
      settings: expect.objectContaining({
        panMin: -90,
        panMax: 90,
        inversePan: true,
      }),
    });
    expect(showGroups.settings[1]?.panMin).toBe(-90);
  });

  it("syncs external updates from showGroups.settings", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "show_configure_get_overview") {
        return Promise.resolve(
          makeOverview(
            [
              { id: 0, name: "All", fixtureCount: 1, children: [] },
              { id: 1, name: "Group 1", fixtureCount: 1, children: [] },
            ],
            {
              0: makeGroupInfo(0, "All"),
              1: makeGroupInfo(1, "Group 1"),
            },
          ),
        );
      }
      return Promise.resolve({});
    });

    useShowConfigurationSync();
    const configure = useShowConfiguration();
    const showGroups = useShowGroups();

    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Group 1" },
    ];

    const initial = configure.getSettings(1);
    expect(initial.panMin).toBe(-180);

    showGroups.settings = {
      1: {
        ...defaultGroupSettings(),
        panMin: -45,
        panMax: 45,
      },
    };

    await nextTick();
    await nextTick();

    expect(configure.getSettings(1).panMin).toBe(-45);
    expect(configure.getSettings(1).panMax).toBe(45);
  });

  it("resets selectedGroupId to 0 when selected group is removed", async () => {
    useShowConfigurationSync();
    const configure = useShowConfiguration();
    const showGroups = useShowGroups();

    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "To Delete" },
    ];

    configure.selectGroup(1);
    expect(configure.selectedGroupId).toBe(1);

    showGroups.groups = [{ id: 0, name: "All" }];

    await nextTick();
    await nextTick();

    expect(configure.selectedGroupId).toBe(0);
  });

  it("uses backend-provided settings and bounds for the selected group", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "show_configure_get_overview") {
        return Promise.resolve(
          makeOverview(
            [
              { id: 0, name: "All", fixtureCount: 1, children: [] },
              { id: 1, name: "Heads A", fixtureCount: 1, children: [] },
              { id: 2, name: "Heads B", fixtureCount: 1, children: [] },
            ],
            {
              0: makeGroupInfo(0, "All", {
                fixtureCount: 1,
                hasPan: true,
                hasTilt: true,
                panBounds: [-360, 360],
                tiltBounds: [-135, 135],
              }),
              1: makeGroupInfo(1, "Heads A", {
                fixtureCount: 1,
                hasPan: true,
                hasTilt: true,
                panBounds: [-270, 270],
                tiltBounds: [-135, 135],
                settings: defaultGroupSettings("Color Mix", [-270, 270], [-135, 135]),
              }),
              2: makeGroupInfo(2, "Heads B", {
                fixtureCount: 1,
                hasPan: true,
                hasTilt: true,
                panBounds: [-360, 360],
                tiltBounds: [-100, 100],
                settings: defaultGroupSettings("Color Mix", [-360, 360], [-100, 100]),
              }),
            },
          ),
        );
      }
      return Promise.resolve({});
    });

    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Heads A" },
      { id: 2, name: "Heads B" },
    ];
    await configure.fetchOverview();

    // Group 1: overview-provided bounds
    configure.selectGroup(1);
    expect(configure.groupPanBounds).toEqual([-270, 270]);
    expect(configure.groupTiltBounds).toEqual([-135, 135]);
    expect(configure.selectedSettings.panMin).toBe(-270);
    expect(configure.selectedSettings.tiltMax).toBe(135);

    // Group 2: overview-provided bounds
    configure.selectGroup(2);
    expect(configure.groupPanBounds).toEqual([-360, 360]);
    expect(configure.groupTiltBounds).toEqual([-100, 100]);
    expect(configure.selectedSettings.panMin).toBe(-360);
    expect(configure.selectedSettings.tiltMax).toBe(100);

    // Group 0 ("All"): envelope from backend overview
    configure.selectGroup(0);
    expect(configure.groupPanBounds).toEqual([-360, 360]);
    expect(configure.groupTiltBounds).toEqual([-135, 135]);
  });

  it("falls back to sensible defaults without an overview", () => {
    const configure = useShowConfiguration();
    const showGroups = useShowGroups();
    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Group" },
    ];

    expect(configure.groupRows).toEqual([]);
    expect(configure.hasPan).toBe(false);
    expect(configure.hasTilt).toBe(false);
    expect(configure.hasColor).toBe(false);
    expect(configure.colorOptions).toEqual([]);
    expect(configure.groupPanBounds).toEqual([-180, 180]);
    expect(configure.groupTiltBounds).toEqual([-110, 110]);
    expect(configure.selectedSettings).toEqual(defaultGroupSettings());
  });

  it("reconciles settingsByGroup when incoming settings remove an entry", async () => {
    useShowConfigurationSync();
    const configure = useShowConfiguration();
    const showGroups = useShowGroups();

    showGroups.groups = [
      { id: 0, name: "All" },
      { id: 1, name: "Group 1" },
    ];
    showGroups.settings = {
      1: {
        colorPreference: "Color Mix",
        panMin: -50,
        panMax: 50,
        tiltMin: -30,
        tiltMax: 30,
        inversePan: true,
        inverseTilt: true,
      },
    };
    await nextTick();
    await nextTick();

    expect(configure.getSettings(1).inversePan).toBe(true);

    // Simulate external removal (e.g. undo)
    showGroups.settings = {};
    await nextTick();
    await nextTick();

    expect(configure.settingsByGroup[1]).toBeUndefined();
    // Getting settings again recreates fresh defaults
    expect(configure.getSettings(1).inversePan).toBe(false);
  });

  it("debounces overview refetch when fixtures or groups change", async () => {
    vi.useFakeTimers();
    mockInvoke.mockImplementation(async () => ({}));

    useShowConfigurationSync();
    const showGroups = useShowGroups();
    const showPatch = useShowPatch();

    const overviewCalls = () =>
      mockInvoke.mock.calls.filter(([cmd]) => cmd === "show_configure_get_overview");
    expect(overviewCalls().length).toBe(1); // initial fetch

    showGroups.groups = [{ id: 0, name: "All" }];
    await nextTick();
    vi.advanceTimersByTime(100);

    showPatch.fixtures = [{} as PatchedFixture];
    await nextTick();
    vi.advanceTimersByTime(100);
    expect(overviewCalls().length).toBe(1); // still debounced across both changes

    await vi.advanceTimersByTimeAsync(50);
    expect(overviewCalls().length).toBe(2); // single debounced refetch fired
  });
});
