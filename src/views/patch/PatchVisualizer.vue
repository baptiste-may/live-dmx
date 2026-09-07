<script setup lang="ts">
import { Card, CardBody, Text } from "daisy-ui-kit";

import NumberStepper from "@/components/inputs/NumberStepper.vue";
import { usePatchVisualizer } from "@/stores/patch/patch-visualizer";
import PatchVisualizerGrid from "@/views/patch/visualizer/PatchVisualizerGrid.vue";

const visualizer = usePatchVisualizer();
</script>

<template>
  <Card class="bg-base-200 h-full w-fit overflow-hidden">
    <CardBody class="flex h-full flex-col gap-4">
      <Text is="h2" bold size="2xl">Visualizer</Text>
      <div class="flex items-center gap-2">
        <span class="text-lg font-bold opacity-70">Universe :</span>
        <NumberStepper v-model="visualizer.currentUniverse" :min="1" :max="512" />
      </div>
      <PatchVisualizerGrid class="min-h-0 flex-1" />
    </CardBody>
  </Card>
</template>
