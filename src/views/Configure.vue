<script setup lang="ts">
import { Card, CardBody, CardTitle } from "daisy-ui-kit";

import { useShowConfigurationSync } from "@/libs/show-configuration-sync.ts";

import ConfigureSelection from "./configure/ConfigureSelection.vue";
import ConfigureSettings from "./configure/ConfigureSettings.vue";

useShowConfigurationSync();
</script>

<template>
  <div class="text-base-content flex h-full w-full gap-4 overflow-hidden p-4">
    <h1 class="sr-only">Configure</h1>
    <div class="flex h-full w-sm flex-col gap-4">
      <ConfigureSelection />
      <ConfigureSettings />
    </div>
    <Card class="bg-base-200 h-full w-full overflow-hidden">
      <CardBody class="flex h-full min-h-0 flex-col gap-4">
        <CardTitle is="h2" bold size="2xl">Layout</CardTitle>
      </CardBody>
    </Card>
  </div>
</template>
