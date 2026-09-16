<script setup lang="ts">
import { Toggle } from "daisy-ui-kit";

import { useSettings } from "@/stores/settings.ts";

const settings = useSettings();
</script>

<template>
  <div class="flex flex-col gap-2">
    <h4 class="text-base-content/70 text-xs font-bold tracking-wider uppercase">Live</h4>

    <div class="bg-base-300/40 flex flex-col gap-4 rounded-lg p-3">
      <label class="flex cursor-pointer items-center justify-between gap-4">
        <span class="text-sm font-medium">Show button labels</span>
        <Toggle
          primary
          :model-value="settings.live.showButtonLabels"
          aria-label="Show button labels"
          @update:model-value="settings.setShowButtonLabels"
        />
      </label>

      <label class="flex cursor-pointer items-center justify-between gap-4">
        <span class="text-sm font-medium">Show section titles</span>
        <Toggle
          primary
          :model-value="settings.live.showSectionTitles"
          aria-label="Show section titles"
          @update:model-value="settings.setShowSectionTitles"
        />
      </label>
    </div>
  </div>
</template>
