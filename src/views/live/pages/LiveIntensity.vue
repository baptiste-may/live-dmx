<script setup lang="ts">
import LiveIntensityEffects from "@/views/live/pages/intensity/LiveIntensityEffects.vue";
import LiveIntensityFlashs from "@/views/live/pages/intensity/LiveIntensityFlashs.vue";
import LiveIntensityMasters from "@/views/live/pages/intensity/LiveIntensityMasters.vue";
</script>

<template>
  <div class="grid h-full min-h-0 grid-cols-[1fr_auto] gap-4 p-4">
    <div class="grid min-h-0 w-full grid-rows-[2fr_3fr] gap-4">
      <LiveIntensityEffects class="min-h-0" />
      <LiveIntensityMasters class="min-h-0" />
    </div>
    <LiveIntensityFlashs class="min-h-0" />
  </div>
</template>
