<script setup lang="ts">
import { Button } from "daisy-ui-kit";
import { CheckIcon } from "lucide-vue-next";

defineProps<{
  color?: string;
  cell?: boolean;
}>();

const modelValue = defineModel<boolean>({ default: false });
</script>

<template>
  <Button
    v-bind="$attrs"
    :color="modelValue ? 'error' : color || 'warning'"
    :square="cell"
    sm
    :class="[cell ? 'rounded-lg border-0' : '']"
    :style="cell ? `background-color: ${color || 'warning'}` : ''"
    @click="modelValue = !modelValue"
  >
    <slot v-if="!cell" />
    <CheckIcon v-else-if="modelValue" class="h-6 w-6" :stroke-width="4" />
  </Button>
</template>
