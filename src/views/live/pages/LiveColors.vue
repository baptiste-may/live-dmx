<script setup lang="ts">
import LiveColorsEffects from "@/views/live/pages/colors/LiveColorsEffects.vue";
import LiveColorsFade from "@/views/live/pages/colors/LiveColorsFade.vue";
import LiveColorsTable from "@/views/live/pages/colors/LiveColorsTable.vue";
</script>

<template>
  <div class="grid h-full min-h-0 grid-rows-[5fr_3fr] gap-4 p-4">
    <div class="grid min-h-0 w-full grid-cols-[1fr_auto] gap-4">
      <LiveColorsTable class="min-h-0" />
      <LiveColorsFade class="min-h-0" />
    </div>
    <LiveColorsEffects class="min-h-0" />
  </div>
</template>
