<script setup lang="ts">
import { Badge, Button, Card, CardBody, Text } from "daisy-ui-kit";
import { RefreshCwIcon } from "lucide-vue-next";

import { useOutputInterfaces } from "@/stores/output/output-interfaces.ts";
import { useOutputUniverses } from "@/stores/output/output-universes.ts";

import OutputInterfaceCard from "./interfaces/OutputInterfaceCard.vue";

const interfacesStore = useOutputInterfaces();
const universesStore = useOutputUniverses();
</script>

<template>
  <Card class="bg-base-200 h-full w-sm overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Text is="h2" bold size="2xl">Physical Interfaces</Text>
          <Badge neutral soft size="sm"> {{ interfacesStore.interfaces.length }} detected </Badge>
        </div>

        <Button
          ghost
          square
          sm
          aria-label="Rescan physical interfaces"
          :disabled="interfacesStore.isScanning"
          @click="interfacesStore.scanInterfaces"
        >
          <RefreshCwIcon
            class="h-4 w-4 transition-transform"
            :class="[interfacesStore.isScanning ? 'animate-spin' : '']"
          />
        </Button>
      </div>

      <div
        v-if="interfacesStore.interfaces.length > 0"
        class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto pr-1"
      >
        <OutputInterfaceCard
          v-for="item in interfacesStore.interfaces"
          :key="item.id"
          :item="item"
          :assigned-universes="universesStore.assignedUniversesByInterface[item.id] || []"
        />
      </div>

      <div
        v-else
        class="border-base-300 rounded-box flex flex-1 flex-col items-center justify-center gap-3 border border-dashed p-8 text-center"
      >
        <span class="text-base-content/60 text-sm">No physical interfaces detected</span>
        <Button
          primary
          sm
          :disabled="interfacesStore.isScanning"
          @click="interfacesStore.scanInterfaces"
        >
          <RefreshCwIcon class="h-4 w-4" />
          Rescan Devices
        </Button>
      </div>
    </CardBody>
  </Card>
</template>
