<script setup lang="ts">
import { Card, CardBody, CardTitle, Menu } from "daisy-ui-kit";

import { useShowConfiguration } from "@/stores/show/show-configuration.ts";

import ConfigureSelectionMenuItem from "./selection/ConfigureSelectionMenuItem.vue";

const configure = useShowConfiguration();
</script>

<template>
  <Card class="bg-base-200 h-fit w-sm overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <CardTitle is="h2" bold size="2xl">Selection</CardTitle>
      <Menu class="min-h-0 w-full overflow-y-auto">
        <ConfigureSelectionMenuItem
          v-for="group in configure.groupRows"
          :key="group.id"
          :group-id="group.id"
        />
      </Menu>
    </CardBody>
  </Card>
</template>
