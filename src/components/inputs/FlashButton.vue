<script setup lang="ts">
import { Button } from "daisy-ui-kit";

const modelValue = defineModel<boolean>({ default: false });

function handleKeyDown(event: KeyboardEvent) {
  if (event.code === "Space" || event.key === " " || event.key === "Enter") {
    event.preventDefault();
    modelValue.value = true;
  }
}

function handleKeyUp(event: KeyboardEvent) {
  if (event.code === "Space" || event.key === " " || event.key === "Enter") {
    event.preventDefault();
    modelValue.value = false;
  }
}
</script>

<template>
  <Button
    v-bind="$attrs"
    :color="modelValue ? 'error' : 'warning'"
    sm
    @mousedown="modelValue = true"
    @mouseup="modelValue = false"
    @mouseleave="modelValue = false"
    @keydown="handleKeyDown"
    @keyup="handleKeyUp"
    @blur="modelValue = false"
  >
    <slot />
  </Button>
</template>
