<script setup lang="ts">
import { Card, CardBody, CardTitle, Table } from "daisy-ui-kit";
import { storeToRefs } from "pinia";

import Knob from "@/components/inputs/Knob.vue";
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import { useSettings } from "@/stores/settings.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import LiveIntensityEffectsRow from "@/views/live/pages/intensity/effects/LiveIntensityEffectsRow.vue";

const { groups } = storeToRefs(useShowGroups());
const liveButtons = useLiveButtons();
const settings = useSettings();
</script>

<template>
  <div class="relative flex h-full min-h-0 flex-col pt-4">
    <div
      class="bg-base-300 border-base-content/20 absolute -top-1.5 left-1/2 z-10 flex -translate-x-1/2 items-center gap-3 rounded-full border px-3 py-0.5 shadow-sm"
    >
      <div class="flex items-center gap-2">
        <span class="text-xs font-semibold">BPM :</span>
        <Knob
          aria-label="BPM"
          size="sm"
          :model-value="liveButtons.effectBpm"
          :min="0"
          :max="240"
          :default-value="60"
          @update:model-value="liveButtons.setEffectBpm($event)"
        />
      </div>
      <div class="flex items-center gap-2">
        <span class="text-xs font-semibold">Width :</span>
        <Knob
          aria-label="Width"
          size="sm"
          unit="%"
          :model-value="liveButtons.effectWidth"
          :min="0"
          :max="100"
          :default-value="100"
          @update:model-value="liveButtons.setEffectWidth($event)"
        />
      </div>
    </div>
    <Card class="bg-base-200 min-h-0 flex-1 overflow-hidden">
      <CardBody class="flex h-full min-h-0 flex-col gap-4">
        <CardTitle is="h2" v-if="settings.live.showSectionTitles" bold size="2xl">
          Effects
        </CardTitle>
        <div class="min-h-0 flex-1 overflow-auto">
          <Table sm class="w-auto">
            <tbody>
              <LiveIntensityEffectsRow
                v-for="group in groups.filter((g) => g.id !== 0)"
                :key="group.id"
                :group="group"
              />
            </tbody>
          </Table>
        </div>
      </CardBody>
    </Card>
  </div>
</template>
