<script setup lang="ts">
import { Card, CardBody, Text } from "daisy-ui-kit";

import PatchFormAdder from "./form/PatchFormAdder.vue";
import PatchFormBrowser from "./form/PatchFormBrowser.vue";
</script>

<template>
  <Card class="bg-base-200 h-full w-sm overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <Text is="h2" bold size="2xl">Fixtures</Text>
      <PatchFormBrowser />
      <PatchFormAdder />
    </CardBody>
  </Card>
</template>
