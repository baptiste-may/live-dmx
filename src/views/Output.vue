<script setup lang="ts">
import { onMounted } from "vue";

import { useOutputInterfaces } from "@/stores/output/output-interfaces.ts";
import { useOutputUniverses } from "@/stores/output/output-universes.ts";

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
  <div class="text-base-content grid h-full w-full grid-cols-1 gap-4 p-4 lg:grid-cols-[1fr_auto]">
    <OutputUniverses />
    <OutputInterfaces />
  </div>
</template>
