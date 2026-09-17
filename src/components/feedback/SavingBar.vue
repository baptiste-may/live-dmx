<script setup lang="ts">
import { storeToRefs } from "pinia";
import { onUnmounted, ref, watch } from "vue";

import { useShow } from "@/stores/show/show.ts";

const MIN_VISIBLE_MS = 400;
const FINISH_MS = 200;
const TRICKLE_MS = 120;
const TRICKLE_CEILING = 90;

const { isSaving } = storeToRefs(useShow());

const visible = ref(false);
const progress = ref(0);
let startedAt = 0;
let trickleTimer: ReturnType<typeof setInterval> | null = null;
let finishTimer: ReturnType<typeof setTimeout> | null = null;

function clearTimers() {
  if (trickleTimer) clearInterval(trickleTimer);
  if (finishTimer) clearTimeout(finishTimer);
  trickleTimer = null;
  finishTimer = null;
}

function start() {
  clearTimers();
  visible.value = true;
  progress.value = 8;
  startedAt = Date.now();
  trickleTimer = setInterval(() => {
    const remaining = TRICKLE_CEILING - progress.value;
    progress.value = Math.min(TRICKLE_CEILING, progress.value + Math.max(0.5, remaining * 0.1));
  }, TRICKLE_MS);
}

function finish() {
  if (!visible.value) return;
  clearTimers();
  const elapsed = Date.now() - startedAt;
  finishTimer = setTimeout(
    () => {
      progress.value = 100;
      finishTimer = setTimeout(() => {
        visible.value = false;
        progress.value = 0;
      }, FINISH_MS);
    },
    Math.max(0, MIN_VISIBLE_MS - elapsed),
  );
}

watch(isSaving, (saving) => (saving ? start() : finish()), { immediate: true });

onUnmounted(clearTimers);
</script>

<template>
  <div class="pointer-events-none fixed inset-x-0 top-0 z-50 h-1">
    <div
      v-if="visible"
      class="bg-primary h-full rounded-r-full transition-[width] duration-200 ease-out"
      role="progressbar"
      :aria-valuenow="Math.round(progress)"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-label="Saving show"
      :style="{ width: `${progress}%` }"
    ></div>
  </div>
</template>
