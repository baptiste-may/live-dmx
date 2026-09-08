<script setup lang="ts">
import { Badge, Button, Toggle } from "daisy-ui-kit";
import { NetworkIcon, Trash2Icon, UsbIcon } from "lucide-vue-next";
import { computed } from "vue";

import {
  type PhysicalInterface,
  type UniverseOutputConfig,
  VIRTUAL_PROTOCOLS,
} from "@/types/output.ts";

import OutputTargetSelect from "./OutputTargetSelect.vue";
import OutputUniverseStatus from "./OutputUniverseStatus.vue";

const props = defineProps<{
  config: UniverseOutputConfig;
  physicalInterfaces: PhysicalInterface[];
  canRemove: boolean;
}>();

defineEmits<{
  (e: "update:output", value: string): void;
  (e: "update:enabled", value: boolean): void;
  (e: "remove"): void;
}>();

const selectedPhysical = computed(() =>
  props.physicalInterfaces.find((phy) => phy.id === props.config.output),
);
const selectedVirtual = computed(() =>
  VIRTUAL_PROTOCOLS.find((proto) => proto.id === props.config.output),
);

const isPhysical = computed(() => props.config.output.startsWith("physical"));
const isMissingPhysical = computed(() => isPhysical.value && !selectedPhysical.value);
const isVirtual = computed(() => Boolean(selectedVirtual.value));

const routingDescription = computed(() => {
  if (!props.config.enabled) {
    return "Output disabled (muted)";
  }
  if (isMissingPhysical.value) {
    return "Physical interface not found";
  }
  if (isPhysical.value && selectedPhysical.value) {
    return `Outputting via ${selectedPhysical.value.name} (${selectedPhysical.value.port} • ${selectedPhysical.value.device})`;
  }
  if (isVirtual.value && selectedVirtual.value) {
    return selectedVirtual.value.description;
  }
  return "No output target routed";
});
</script>

<template>
  <div
    class="bg-base-100 border-base-300 rounded-box flex flex-col gap-3 border p-4 shadow-sm transition-all sm:flex-row sm:items-center sm:justify-between"
    :class="[!config.enabled ? 'opacity-60' : '']"
  >
    <div class="flex items-center gap-3">
      <OutputUniverseStatus
        :universe="config.universe"
        :enabled="config.enabled"
        :output="config.output"
        :physical-status="selectedPhysical?.status"
        :is-missing-physical="isMissingPhysical"
      />

      <div class="flex flex-col">
        <div class="flex items-center gap-2">
          <span class="font-bold">Universe {{ config.universe }}</span>
          <Badge v-if="!config.enabled" neutral soft size="sm"> Disabled </Badge>
          <Badge v-else-if="isMissingPhysical" error soft size="sm"> Missing Interface </Badge>
          <Badge
            v-else-if="isPhysical && selectedPhysical?.status === 'disconnected'"
            error
            soft
            size="sm"
          >
            Error
          </Badge>
          <Badge v-else-if="isPhysical" info soft size="sm" class="flex items-center gap-1">
            <UsbIcon class="h-3 w-3" />
            Physical
          </Badge>
          <Badge v-else-if="isVirtual" accent soft size="sm" class="flex items-center gap-1">
            <NetworkIcon class="h-3 w-3" />
            Virtual
          </Badge>
          <Badge v-else neutral soft size="sm"> Unassigned </Badge>
        </div>
        <span class="text-base-content/60 text-xs">
          {{ routingDescription }}
        </span>
      </div>
    </div>

    <div class="flex items-center gap-3 sm:self-center">
      <div class="w-56 sm:w-64">
        <OutputTargetSelect
          :model-value="config.output"
          :physical-interfaces="physicalInterfaces"
          @update:model-value="$emit('update:output', $event)"
        />
      </div>
      <Toggle
        :model-value="config.enabled"
        success
        sm
        :aria-label="'Toggle universe ' + config.universe"
        @update:model-value="$emit('update:enabled', $event)"
      />
      <Button
        ghost
        square
        sm
        class="text-error"
        :disabled="!canRemove"
        :aria-label="'Remove universe ' + config.universe"
        @click="$emit('remove')"
      >
        <Trash2Icon class="h-4 w-4" />
      </Button>
    </div>
  </div>
</template>
