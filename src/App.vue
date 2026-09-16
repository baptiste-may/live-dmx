<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onMounted } from "vue";

import ProgressScreen from "@/components/feedback/ProgressScreen.vue";
import AppLayout from "@/components/layout/AppLayout.vue";
import Welcome from "@/components/Welcome.vue";
import { initLogger } from "@/libs/logger";
import { useGlobalShortcuts } from "@/libs/shortcuts";
import { useShowOpener } from "@/libs/show-opener";
import { useSettings } from "@/stores/settings.ts";
import { useShow } from "@/stores/show/show.ts";
import { useShowChanges } from "@/stores/show/show-changes.ts";

const show = useShow();
const changes = useShowChanges();
const settings = useSettings();

useGlobalShortcuts();
useShowOpener();

onMounted(async () => {
  settings.initTheme();
  await initLogger();
  await show.initEventListener();

  try {
    const appWindow = getCurrentWindow();
    if (appWindow && typeof appWindow.onCloseRequested === "function") {
      await appWindow.onCloseRequested(async (event) => {
        if (show.hasUnsavedChanges) {
          event.preventDefault();
          changes.confirmWithSaveGuard(async () => {
            await appWindow.destroy();
          });
        }
      });
    }
  } catch {
    // Ignored in non-Tauri / test environments
  }
});
</script>

<template>
  <ProgressScreen v-if="show.isLoading" title="Loading show..." />
  <ProgressScreen v-else-if="show.isSaving" title="Saving show..." />
  <AppLayout v-else-if="show.isLoaded" />
  <Welcome v-else />
</template>
