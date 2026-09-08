<script setup lang="ts">
import { onMounted } from "vue";

import { useOutputInterfaces } from "@/stores/output/output-interfaces.ts";
import { useOutputUniverses } from "@/stores/output/output-universes.ts";

import OutputDmxMonitor from "./output/dmx/OutputDmxMonitor.vue";
import OutputInterfaces from "./output/OutputInterfaces.vue";
import OutputUniverses from "./output/OutputUniverses.vue";

const universesStore = useOutputUniverses();
const interfacesStore = useOutputInterfaces();

onMounted(() => {
  void universesStore.fetchUniverses();
  void interfacesStore.fetchInterfaces();
});
</script>

<template>
  <div class="text-base-content flex h-full w-full flex-col gap-4 overflow-hidden p-4">
    <div class="grid min-h-0 flex-1 grid-cols-1 gap-4 lg:grid-cols-[1fr_auto]">
      <OutputUniverses />
      <OutputInterfaces />
    </div>
    <OutputDmxMonitor class="shrink-0" />
  </div>
</template>
