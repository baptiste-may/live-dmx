import { watch } from "vue";

import { useShowConfiguration } from "@/stores/show/show-configuration.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import { useShowPatch } from "@/stores/show/show-patch.ts";

const OVERVIEW_REFETCH_DELAY_MS = 150;

let overviewTimeout: ReturnType<typeof setTimeout> | null = null;

function scheduleOverviewFetch(fetchAction: () => Promise<unknown>): void {
  if (overviewTimeout !== null) {
    clearTimeout(overviewTimeout);
  }
  overviewTimeout = setTimeout(() => {
    overviewTimeout = null;
    fetchAction().catch(() => {});
  }, OVERVIEW_REFETCH_DELAY_MS);
}

export function useShowConfigurationSync(): ReturnType<typeof useShowConfiguration> {
  const configure = useShowConfiguration();
  const showGroups = useShowGroups();
  const showPatch = useShowPatch();

  configure.fetchOverview().catch(() => {});

  // Synchronize local settings cache with showGroups.settings when changed externally (e.g. undo/redo, load)
  watch(
    () => showGroups.settings,
    (incoming) => {
      configure.reconcileSettings(incoming);
    },
    { deep: true },
  );

  // Reset selected group if it was removed
  watch(
    () => showGroups.groups,
    (groups) => {
      if (!groups.some((group) => group.id === configure.selectedGroupId)) {
        configure.selectedGroupId = 0;
      }
    },
  );

  // Re-fetch backend overview whenever patch fixtures or groups change
  watch(
    () => [showPatch.fixtures, showGroups.groups],
    () => {
      scheduleOverviewFetch(() => configure.fetchOverview());
    },
    { deep: true },
  );

  // Sync edits to the backend when the currently selected group's settings change
  watch(
    () => [configure.selectedGroupId, { ...configure.selectedSettings }] as const,
    ([newGroupId, newSettings], [oldGroupId]) => {
      if (newGroupId !== oldGroupId) {
        return;
      }
      const saved = showGroups.settings[newGroupId];
      if (!saved || JSON.stringify(saved) !== JSON.stringify(newSettings)) {
        configure.scheduleSave(newGroupId);
      }
    },
    { deep: true },
  );

  return configure;
}
