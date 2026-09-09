<script setup lang="ts">
import PatchForm from "./patch/PatchForm.vue";
import PatchGroups from "./patch/PatchGroups.vue";
import PatchVisualizer from "./patch/PatchVisualizer.vue";
</script>

<template>
  <div
    class="text-base-content grid h-full min-h-0 w-full min-w-0 grid-cols-[minmax(0,auto)_minmax(0,1fr)_minmax(0,auto)] gap-4 overflow-x-auto p-4"
  >
    <PatchForm />
    <PatchGroups />
    <PatchVisualizer />
  </div>
</template>
