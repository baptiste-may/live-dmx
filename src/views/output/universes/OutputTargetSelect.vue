<script setup lang="ts">
import { type PhysicalInterface, VIRTUAL_PROTOCOLS, type VirtualProtocol } from "@/types/output.ts";

withDefaults(
  defineProps<{
    modelValue: string;
    physicalInterfaces: PhysicalInterface[];
    virtualProtocols?: VirtualProtocol[];
    disabled?: boolean;
  }>(),
  {
    virtualProtocols: () => VIRTUAL_PROTOCOLS,
    disabled: false,
  },
);

defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();
</script>

<template>
  <select
    :value="modelValue"
    :disabled="disabled"
    class="select select-bordered select-sm w-full font-medium"
    aria-label="Select output destination"
    @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <option value="none">None</option>
    <optgroup label="Physical">
      <option v-for="phy in physicalInterfaces" :key="phy.id" :value="phy.id">
        {{ phy.name }} ({{ phy.device }})
      </option>
    </optgroup>
    <optgroup label="Virtual">
      <option v-for="proto in virtualProtocols" :key="proto.id" :value="proto.id">
        {{ proto.name }}
      </option>
    </optgroup>
  </select>
</template>
