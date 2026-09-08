<script setup lang="ts">
import { Button } from "daisy-ui-kit";
import { CheckIcon } from "lucide-vue-next";
import type { Component } from "vue";
import { computed } from "vue";

import { useSettings } from "@/stores/settings.ts";

const props = withDefaults(
  defineProps<{
    color?: string;
    cell?: boolean;
    label?: string;
    icon?: Component;
    size?: "xs" | "sm" | "md" | "lg" | "xl";
    mode?: "toggle" | "momentary";
  }>(),
  {
    color: undefined,
    cell: false,
    label: undefined,
    icon: undefined,
    size: undefined,
    mode: "toggle",
  },
);

const modelValue = defineModel<boolean>({ default: false });
const settings = useSettings();
const showLabel = computed(() => Boolean(props.label && settings.live.showButtonLabels));

function handleClick() {
  if (props.mode !== "momentary") {
    modelValue.value = !modelValue.value;
  }
}

function handlePointerDown() {
  if (props.mode === "momentary") {
    modelValue.value = true;
  }
}

function handlePointerUp() {
  if (props.mode === "momentary") {
    modelValue.value = false;
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (
    props.mode === "momentary" &&
    (event.code === "Space" || event.key === " " || event.key === "Enter")
  ) {
    event.preventDefault();
    modelValue.value = true;
  }
}

function handleKeyUp(event: KeyboardEvent) {
  if (
    props.mode === "momentary" &&
    (event.code === "Space" || event.key === " " || event.key === "Enter")
  ) {
    event.preventDefault();
    modelValue.value = false;
  }
}
</script>

<template>
  <Button
    v-bind="$attrs"
    :aria-pressed="modelValue"
    :aria-label="label || ($attrs['aria-label'] as string) || undefined"
    :color="modelValue ? 'error' : color || 'warning'"
    square
    :size="size || (cell ? 'sm' : 'lg')"
    class="rounded-lg border-0"
    :class="[cell && !color ? 'bg-warning' : '', cell && color ? 'outline outline-black' : '']"
    :style="cell && color ? { backgroundColor: color } : undefined"
    @click="handleClick"
    @mousedown="handlePointerDown"
    @mouseup="handlePointerUp"
    @mouseleave="handlePointerUp"
    @keydown="handleKeyDown"
    @keyup="handleKeyUp"
    @blur="handlePointerUp"
  >
    <template v-if="cell">
      <CheckIcon v-if="modelValue" class="h-6 w-6" :stroke-width="4" />
      <span v-if="label" class="sr-only">{{ label }}</span>
    </template>
    <div
      v-else
      class="flex h-full w-full flex-col items-center justify-center gap-0.5 p-1 text-center select-none"
    >
      <component
        :is="icon"
        v-if="icon"
        aria-hidden="true"
        class="shrink-0"
        :class="[
          showLabel
            ? size === 'xl'
              ? 'h-5 w-5'
              : 'h-4 w-4'
            : size === 'sm' || size === 'xs'
              ? 'h-4 w-4'
              : 'h-6 w-6',
        ]"
      />
      <span
        v-if="showLabel"
        class="line-clamp-2 max-w-full text-center text-[10px] leading-tight font-medium wrap-break-word whitespace-normal"
      >
        {{ label }}
      </span>
      <span v-else-if="label || $attrs['aria-label']" class="sr-only">
        {{ label || $attrs["aria-label"] }}
      </span>
    </div>
  </Button>
</template>
