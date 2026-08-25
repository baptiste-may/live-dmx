<script setup lang="ts">
import { useLiveNavigation } from "@/stores/live/live-navigation";

const navigation = useLiveNavigation();
</script>

<template>
  <KeepAlive v-if="navigation.currentPage">
    <component :is="navigation.currentPage?.component" class="min-h-0 flex-1" />
  </KeepAlive>
</template>
