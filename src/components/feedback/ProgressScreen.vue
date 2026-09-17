<script setup lang="ts">
import { storeToRefs } from "pinia";
import { computed } from "vue";

import { useShow } from "@/stores/show/show.ts";

defineProps<{
  title: string;
}>();

const { progress, statusMessage } = storeToRefs(useShow());

const clamped = computed(() => Math.min(100, Math.max(0, progress.value ?? 0)));
</script>

<template>
  <div class="flex h-screen w-screen flex-col items-center justify-center gap-6">
    <p class="text-lg font-bold">{{ title }}</p>
    <div
      class="bg-base-200 h-3 w-1/2 overflow-hidden rounded-full"
      role="progressbar"
      :aria-valuenow="clamped"
      aria-valuemin="0"
      aria-valuemax="100"
    >
      <div
        class="bg-primary h-full rounded-full transition-[width] duration-500 ease-out"
        :class="clamped === 0 ? 'animate-pulse' : ''"
        :style="{ width: clamped === 0 ? '25%' : `${clamped}%` }"
      ></div>
    </div>
    <p class="text-sm tabular-nums opacity-70">{{ clamped }}%</p>
    <p v-if="statusMessage" class="-mt-4 text-sm opacity-70">{{ statusMessage }}</p>
  </div>
</template>
