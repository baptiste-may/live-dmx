<script setup lang="ts">
import PatchForm from "./patch/PatchForm.vue";
import PatchGroups from "./patch/PatchGroups.vue";
import PatchVisualizer from "./patch/PatchVisualizer.vue";
</script>

<template>
  <div class="text-base-content grid h-full w-full grid-cols-[auto_1fr_auto] gap-4 p-4">
    <PatchForm />
    <PatchGroups />
    <PatchVisualizer />
  </div>
</template>
