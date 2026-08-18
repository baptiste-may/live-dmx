<script setup lang="ts">
import NumberStepper from "@/components/inputs/NumberStepper.vue";
import { usePatchVisualizer } from "@/stores/patch/patch-visualizer.ts";

const visualizer = usePatchVisualizer();
</script>

<template>
  <div class="flex items-center gap-2">
    <span class="text-lg font-bold opacity-70">Universe :</span>
    <NumberStepper v-model="visualizer.currentUniverse" :min="1" :max="512" />
  </div>
</template>
