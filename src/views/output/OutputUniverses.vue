<script setup lang="ts">
import { Badge, Button, Card, CardBody, Text } from "daisy-ui-kit";
import { CircleAlertIcon, PlusIcon, TriangleAlertIcon } from "lucide-vue-next";
import { computed } from "vue";

import { useOutputInterfaces } from "@/stores/output/output-interfaces.ts";
import { useOutputUniverses } from "@/stores/output/output-universes.ts";

import OutputUniverseRow from "./universes/OutputUniverseRow.vue";

const universesStore = useOutputUniverses();
const interfacesStore = useOutputInterfaces();

const errorCount = computed(() => {
  return universesStore.universes.filter((u) => {
    if (!u.enabled) return false;
    if (u.output.startsWith("physical")) {
      const phy = interfacesStore.interfaces.find((p) => p.id === u.output);
      return !phy || phy.status === "disconnected";
    }
    return false;
  }).length;
});

const warningCount = computed(() => {
  return universesStore.universes.filter((u) => {
    if (!u.enabled) return false;
    if (u.output === "none") return true;
    const phy = interfacesStore.interfaces.find((p) => p.id === u.output);
    return Boolean(phy && phy.status === "busy");
  }).length;
});
</script>

<template>
  <Card class="bg-base-200 h-full w-full overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <Text is="h2" bold size="2xl">Universes</Text>

          <div class="flex items-center gap-2">
            <Badge
              :error="errorCount > 0"
              :neutral="errorCount === 0"
              soft
              size="sm"
              class="flex items-center gap-1 font-medium"
              :class="[errorCount === 0 ? 'opacity-60' : '']"
            >
              <TriangleAlertIcon v-if="errorCount > 0" class="h-3 w-3" />
              {{ errorCount }} {{ errorCount === 1 ? "error" : "errors" }}
            </Badge>

            <Badge
              :warning="warningCount > 0"
              :neutral="warningCount === 0"
              soft
              size="sm"
              class="flex items-center gap-1 font-medium"
              :class="[warningCount === 0 ? 'opacity-60' : '']"
            >
              <CircleAlertIcon v-if="warningCount > 0" class="h-3 w-3" />
              {{ warningCount }} {{ warningCount === 1 ? "warning" : "warnings" }}
            </Badge>
          </div>
        </div>
      </div>

      <div
        v-if="universesStore.universes.length > 0"
        class="flex min-h-0 flex-1 flex-col gap-3 overflow-y-auto pr-1"
      >
        <OutputUniverseRow
          v-for="universe in universesStore.universes"
          :key="universe.universe"
          :config="universe"
          :physical-interfaces="interfacesStore.interfaces"
          :can-remove="universesStore.universes.length > 1"
          @update:output="universesStore.setUniverseOutput(universe.universe, $event)"
          @update:enabled="universesStore.setUniverseEnabled(universe.universe, $event)"
          @remove="universesStore.removeUniverse(universe.universe)"
        />

        <Button
          outline
          primary
          sm
          class="mt-1 w-full gap-1.5 border-dashed"
          @click="universesStore.addUniverse"
        >
          <PlusIcon class="h-4 w-4" />
          Add Universe
        </Button>
      </div>

      <div
        v-else
        class="border-base-300 rounded-box flex flex-1 flex-col items-center justify-center gap-3 border border-dashed p-8 text-center"
      >
        <span class="text-base-content/60 text-sm">No universes configured</span>
        <Button primary sm class="gap-1.5" @click="universesStore.addUniverse">
          <PlusIcon class="h-4 w-4" />
          Add First Universe
        </Button>
      </div>
    </CardBody>
  </Card>
</template>
