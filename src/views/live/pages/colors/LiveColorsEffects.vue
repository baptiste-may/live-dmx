<script setup lang="ts">
import { Card, CardBody, CardTitle, Table } from "daisy-ui-kit";
import { storeToRefs } from "pinia";

import Knob from "@/components/inputs/Knob.vue";
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import LiveColorsEffectsRow from "@/views/live/pages/colors/effects/LiveColorsEffectsRow.vue";

const { colorGroups } = storeToRefs(useShowGroups());
const liveButtons = useLiveButtons();
</script>

<template>
  <Card class="bg-base-200 h-full overflow-hidden">
    <CardBody class="relative flex h-full min-h-0 flex-col gap-4">
      <CardTitle is="h2" bold size="2xl">Effects</CardTitle>
      <CardTitle class="absolute top-4 right-1/2 translate-x-1/2 font-normal">
        BPM :
        <Knob
          :model-value="liveButtons.colorEffectBpm"
          :min="0"
          :max="240"
          :default-value="60"
          @update:model-value="liveButtons.setColorEffectBpm($event)"
        />
      </CardTitle>
      <div class="min-h-0 flex-1 overflow-auto">
        <Table sm class="w-auto">
          <tbody>
            <LiveColorsEffectsRow v-for="group in colorGroups" :key="group.id" :group="group" />
          </tbody>
        </Table>
      </div>
    </CardBody>
  </Card>
</template>
