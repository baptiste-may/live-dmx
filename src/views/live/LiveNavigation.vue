<script setup lang="ts">
import { Button, Join } from "daisy-ui-kit";

import { EVERY_LIVE_PAGES, useLiveNavigation } from "@/stores/live/live-navigation";

const navigation = useLiveNavigation();
</script>

<template>
  <Join>
    <Button
      v-for="(page, id) in EVERY_LIVE_PAGES"
      :key="id"
      join
      primary
      :outline="navigation.currentPageId !== id"
      @click="navigation.navigate(id)"
    >
      <component :is="page.icon" class="h-4 w-4" />
      {{ page.name }}
    </Button>
  </Join>
</template>
