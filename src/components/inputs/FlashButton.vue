<script setup lang="ts">
import { Button } from "daisy-ui-kit";

const modelValue = defineModel<boolean>({ default: false });
</script>

<template>
  <Button
    v-bind="$attrs"
    :color="modelValue ? 'error' : 'warning'"
    sm
    @mousedown="modelValue = true"
    @mouseup="modelValue = false"
    @mouseleave="modelValue = false"
  >
    <slot />
  </Button>
</template>
