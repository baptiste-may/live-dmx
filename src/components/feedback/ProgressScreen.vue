<script setup lang="ts">
import { Progress } from "daisy-ui-kit";
import { storeToRefs } from "pinia";

import { useShow } from "@/stores/show/show.ts";

defineProps<{
  title: string;
}>();

const { progress } = storeToRefs(useShow());
</script>

<template>
  <div class="flex h-screen w-screen flex-col items-center justify-center gap-4">
    <p class="text-lg font-bold">{{ title }}</p>
    <Progress class="w-56" :value="progress"></Progress>
  </div>
</template>
