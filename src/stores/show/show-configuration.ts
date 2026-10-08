import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

import type { FixtureGroup } from "@/stores/show/show-groups.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import type {
  ColorPreference,
  ConfigureOverview,
  GroupRow,
  GroupSettings,
  PixelPreference,
} from "@/views/configure/types";

export type ShowConfigurationState = {
  selectedGroupId: number;
  overview: ConfigureOverview | null;
  settingsByGroup: Record<number, GroupSettings>;
  saveTimeout: ReturnType<typeof setTimeout> | null;
  pendingGroupId: number | null;
};

export const defaultShowConfigurationState = (): ShowConfigurationState => ({
  selectedGroupId: 0,
  overview: null,
  settingsByGroup: {},
  saveTimeout: null,
  pendingGroupId: null,
});

export const defaultGroupSettings = (
  colorPreference: ColorPreference = "Color Mix",
  panBounds: [number, number] = [-180, 180],
  tiltBounds: [number, number] = [-110, 110],
  pixelPreference: PixelPreference = "Single",
  pixelGroupCount = 2,
): GroupSettings => ({
  colorPreference,
  pixelPreference,
  pixelGroupCount,
  panMin: panBounds[0],
  panMax: panBounds[1],
  tiltMin: tiltBounds[0],
  tiltMax: tiltBounds[1],
  inversePan: false,
  inverseTilt: false,
});

function ensureSettings(state: ShowConfigurationState, groupId: number): GroupSettings {
  if (state.settingsByGroup[groupId]) return state.settingsByGroup[groupId];

  const info = state.overview?.groups[groupId];
  const existing = useShowGroups().settings[groupId];
  state.settingsByGroup[groupId] = info
    ? { ...info.settings }
    : existing
      ? { ...existing }
      : defaultGroupSettings();
  return state.settingsByGroup[groupId];
}

export const useShowConfiguration = defineStore("show-configuration", {
  state: (): ShowConfigurationState => defaultShowConfigurationState(),
  getters: {
    selectedGroup: (state): FixtureGroup | undefined => {
      const showGroups = useShowGroups();
      return (
        showGroups.groups.find((group) => group.id === state.selectedGroupId) ??
        showGroups.groups[0]
      );
    },
    selectedSettings: (state): GroupSettings => ensureSettings(state, state.selectedGroupId),
    groupRows: (state): GroupRow[] => state.overview?.rows ?? [],
    hasPan: (state): boolean => state.overview?.groups[state.selectedGroupId]?.hasPan ?? false,
    hasTilt: (state): boolean => state.overview?.groups[state.selectedGroupId]?.hasTilt ?? false,
    hasColor: (state): boolean => state.overview?.groups[state.selectedGroupId]?.hasColor ?? false,
    hasPixels: (state): boolean =>
      state.overview?.groups[state.selectedGroupId]?.hasPixels ?? false,
    colorOptions: (state): ColorPreference[] =>
      state.overview?.groups[state.selectedGroupId]?.colorOptions ?? [],
    pixelOptions: (state): PixelPreference[] =>
      state.overview?.groups[state.selectedGroupId]?.pixelOptions ?? [],
    maxPixels: (state): number => state.overview?.groups[state.selectedGroupId]?.maxPixels ?? 1,
    groupPanBounds: (state): [number, number] =>
      state.overview?.groups[state.selectedGroupId]?.panBounds ?? [-180, 180],
    groupTiltBounds: (state): [number, number] =>
      state.overview?.groups[state.selectedGroupId]?.tiltBounds ?? [-110, 110],
  },
  actions: {
    async fetchOverview(): Promise<ConfigureOverview | null> {
      try {
        const data = await invoke<ConfigureOverview>("show_configure_get_overview");
        if (data && Array.isArray(data.rows)) {
          this.overview = data;
          for (const [idStr, info] of Object.entries(data.groups)) {
            const gid = Number(idStr);
            if (!this.settingsByGroup[gid]) {
              this.settingsByGroup[gid] = { ...info.settings };
            }
          }
          return data;
        }
      } catch {
        // Ignored in test / non-Tauri environment
      }
      return null;
    },
    reconcileSettings(incoming: Record<number, GroupSettings>): void {
      const showGroups = useShowGroups();
      for (const idStr of Object.keys(this.settingsByGroup)) {
        const gid = Number(idStr);
        if ((gid !== 0 && !showGroups.groups.some((g) => g.id === gid)) || !(idStr in incoming)) {
          delete this.settingsByGroup[gid];
        }
      }
      for (const [idStr, settings] of Object.entries(incoming)) {
        const gid = Number(idStr);
        if (JSON.stringify(this.settingsByGroup[gid]) !== JSON.stringify(settings)) {
          this.settingsByGroup[gid] = { ...settings };
        }
      }
    },
    flushPendingSave(): void {
      if (this.saveTimeout !== null && this.pendingGroupId !== null) {
        clearTimeout(this.saveTimeout);
        this.saveTimeout = null;
        const gid = this.pendingGroupId;
        this.pendingGroupId = null;
        const settings = this.settingsByGroup[gid];
        if (settings) {
          useShowGroups().updateSettings(gid, { ...settings });
        }
      }
    },
    scheduleSave(groupId: number): void {
      if (this.saveTimeout !== null) {
        clearTimeout(this.saveTimeout);
      }
      this.pendingGroupId = groupId;
      this.saveTimeout = setTimeout(() => {
        this.saveTimeout = null;
        const gid = this.pendingGroupId;
        this.pendingGroupId = null;
        if (gid !== null) {
          const settings = this.settingsByGroup[gid];
          if (settings) {
            useShowGroups().updateSettings(gid, { ...settings });
          }
        }
      }, 250);
    },
    getSettings(groupId: number): GroupSettings {
      return ensureSettings(this.$state, groupId);
    },
    getGroupRow(id: number, rows?: GroupRow[]): GroupRow | undefined {
      for (const row of rows ?? this.groupRows) {
        if (row.id === id) return row;
        const child = this.getGroupRow(id, row.children);
        if (child) return child;
      }
      return undefined;
    },
    selectGroup(id: number): void {
      const showGroups = useShowGroups();
      if (!showGroups.groups.some((group) => group.id === id)) return;
      this.flushPendingSave();
      this.selectedGroupId = id;
    },
    async saveGroupSettings(groupId?: number): Promise<void> {
      if (this.saveTimeout !== null) {
        clearTimeout(this.saveTimeout);
        this.saveTimeout = null;
        this.pendingGroupId = null;
      }
      const gid = groupId ?? this.selectedGroupId;
      const current = this.getSettings(gid);
      await useShowGroups().updateSettings(gid, { ...current });
    },
  },
});
