<script setup lang="ts">
import { Button, Card, CardBody, Text } from "daisy-ui-kit";
import { PauseIcon, PlayIcon } from "lucide-vue-next";
import { onMounted, onUnmounted } from "vue";

import { useOutputDmx } from "@/stores/output/output-dmx.ts";
import { useOutputUniverses } from "@/stores/output/output-universes.ts";

import OutputUniverseCard from "./OutputUniverseCard.vue";

const dmxStore = useOutputDmx();
const universesStore = useOutputUniverses();

let intervalId: ReturnType<typeof setInterval> | null = null;

onMounted(() => {
  void dmxStore.fetchAllDmxData();
  intervalId = setInterval(() => {
    void dmxStore.fetchAllDmxData();
  }, 100);
});

onUnmounted(() => {
  if (intervalId) {
    clearInterval(intervalId);
    intervalId = null;
  }
});
</script>

<template>
  <Card class="bg-base-200 w-full overflow-hidden">
    <CardBody class="flex flex-col gap-3 p-4">
      <!-- Section Header -->
      <div class="flex items-center justify-between">
        <Text is="h2" bold size="xl">DMX Output</Text>

        <Button
          ghost
          square
          xs
          class="bg-base-300/50"
          :title="dmxStore.isPaused ? 'Resume live update' : 'Pause live update'"
          :aria-label="dmxStore.isPaused ? 'Resume live update' : 'Pause live update'"
          @click="dmxStore.togglePause"
        >
          <PlayIcon v-if="dmxStore.isPaused" class="h-3.5 w-3.5" />
          <PauseIcon v-else class="text-success h-3.5 w-3.5" />
        </Button>
      </div>

      <!-- Horizontal list of universe squares -->
      <div
        v-if="universesStore.universes.length > 0"
        class="flex flex-row items-center gap-4 overflow-x-auto pb-1"
      >
        <OutputUniverseCard
          v-for="u in universesStore.universes"
          :key="u.universe"
          :universe="u.universe"
          :output="u.output"
          :enabled="u.enabled"
          :channels="dmxStore.getUniverseData(u.universe)"
        />
      </div>

      <div
        v-else
        class="border-base-300 rounded-box flex items-center justify-center border border-dashed py-8 text-center"
      >
        <span class="text-base-content/60 text-sm">No universes configured</span>
      </div>
    </CardBody>
  </Card>
</template>
