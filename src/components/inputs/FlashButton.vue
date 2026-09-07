<script setup lang="ts">
import type { Component } from "vue";

import ToggleButton from "./ToggleButton.vue";

withDefaults(
  defineProps<{
    color?: string;
    cell?: boolean;
    label?: string;
    icon?: Component;
    size?: "xs" | "sm" | "md" | "lg" | "xl";
  }>(),
  {
    color: undefined,
    cell: false,
    label: undefined,
    icon: undefined,
    size: undefined,
  },
);

const modelValue = defineModel<boolean>({ default: false });
</script>

<template>
  <ToggleButton
    v-model="modelValue"
    mode="momentary"
    :color="color"
    :cell="cell"
    :label="label"
    :icon="icon"
    :size="size"
    v-bind="$attrs"
  />
</template>
