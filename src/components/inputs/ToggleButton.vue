<script setup lang="ts">
import { Button } from "daisy-ui-kit";
import { CheckIcon } from "lucide-vue-next";

defineProps<{
  color?: string;
  cell?: boolean;
  label?: string;
}>();

const modelValue = defineModel<boolean>({ default: false });
</script>

<template>
  <Button
    v-bind="$attrs"
    :aria-pressed="modelValue"
    :aria-label="label || ($attrs['aria-label'] as string) || undefined"
    :color="modelValue ? 'error' : color || 'warning'"
    :square="cell"
    sm
    :class="[cell ? 'rounded-lg border-0' : '', cell && !color ? 'bg-warning' : '']"
    :style="cell && color ? { backgroundColor: color } : undefined"
    @click="modelValue = !modelValue"
  >
    <slot v-if="!cell" />
    <template v-else>
      <CheckIcon v-if="modelValue" class="h-6 w-6" :stroke-width="4" />
      <span v-if="label" class="sr-only">{{ label }}</span>
      <span v-else class="sr-only"><slot /></span>
    </template>
  </Button>
</template>
