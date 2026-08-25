<script setup lang="ts">
import LiveIntensityEffects from "@/views/live/pages/intensity/LiveIntensityEffects.vue";
import LiveIntensityMasters from "@/views/live/pages/intensity/LiveIntensityMasters.vue";
import LiveIntensityStrobes from "@/views/live/pages/intensity/LiveIntensityStrobes.vue";
</script>

<template>
  <div class="grid h-full min-h-0 grid-rows-[2fr_3fr] gap-4">
    <LiveIntensityEffects class="min-h-0" />
    <div class="grid min-h-0 w-full grid-cols-[auto_1fr] gap-4">
      <LiveIntensityStrobes class="min-h-0" />
      <LiveIntensityMasters class="min-h-0" />
    </div>
  </div>
</template>
