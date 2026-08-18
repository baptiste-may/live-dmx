<script setup lang="ts">
import { Card, CardBody, Text } from "daisy-ui-kit";

import PatchVisualizerForm from "@/views/patch/visualizer/PatchVisualizerForm.vue";
import PatchVisualizerGrid from "@/views/patch/visualizer/PatchVisualizerGrid.vue";
</script>

<template>
  <Card class="bg-base-200 h-full w-fit overflow-hidden">
    <CardBody class="flex h-full flex-col gap-4">
      <Text is="h2" bold size="2xl">Visualizer</Text>
      <PatchVisualizerForm />
      <PatchVisualizerGrid class="min-h-0 flex-1" />
    </CardBody>
  </Card>
</template>
