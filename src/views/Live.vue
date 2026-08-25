<script setup lang="ts">
import LiveContent from "@/views/live/LiveContent.vue";
import LiveNavigation from "@/views/live/LiveNavigation.vue";
</script>

<template>
  <div class="flex h-full flex-col gap-4 p-4">
    <LiveNavigation />
    <LiveContent class="min-h-0 flex-1" />
  </div>
</template>
