<script setup lang="ts">
import NumberFlow from "@number-flow/vue";
import { Button, Input, Join } from "daisy-ui-kit";
import { MinusIcon, PlusIcon } from "lucide-vue-next";
import { ref } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
  }>(),
  {
    min: 1,
    max: 512,
    step: 1,
    disabled: false,
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: number): void;
}>();

const isFocused = ref(false);

function updateValue(val: number) {
  let newValue = Number.isNaN(val) ? props.min : val;
  if (props.min !== undefined && newValue < props.min) {
    newValue = props.min;
  }
  if (props.max !== undefined && newValue > props.max) {
    newValue = props.max;
  }
  emit("update:modelValue", newValue);
}

function decrement() {
  if (props.disabled) return;
  updateValue((props.modelValue || 0) - props.step);
}

function increment() {
  if (props.disabled) return;
  updateValue((props.modelValue || 0) + props.step);
}

function onInput(e: Event) {
  const target = e.target as HTMLInputElement;
  const val = parseInt(target.value, 10);
  if (!Number.isNaN(val)) {
    updateValue(val);
  }
}
</script>

<template>
  <Join>
    <Button
      type="button"
      ghost
      square
      join
      class="border-base-content/20 border border-r-0"
      :disabled="disabled || (min !== undefined && modelValue <= min)"
      aria-label="Decrease"
      @click="decrement"
    >
      <MinusIcon
        class="h-4 w-4 transition-colors"
        :class="
          disabled || (min !== undefined && modelValue <= min)
            ? 'text-base-content/30'
            : 'text-error'
        "
      />
    </Button>

    <div class="join-item relative flex flex-1 items-center justify-center">
      <Input
        type="number"
        :model-value="modelValue"
        :min="min"
        :max="max"
        :step="step"
        :disabled="disabled"
        join
        :class="[
          'border-base-content/20 w-full rounded-none border-x-0 border-y text-center focus:outline-none',
          '[appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none',
          !isFocused && 'caret-base-content text-transparent',
        ]"
        @focus="isFocused = true"
        @blur="isFocused = false"
        @input="onInput"
      />

      <NumberFlow
        v-if="!isFocused"
        :value="modelValue"
        class="text-base-content pointer-events-none absolute text-sm font-bold"
      />
    </div>

    <Button
      type="button"
      ghost
      square
      join
      class="border-base-content/20 border border-l-0"
      :disabled="disabled || (max !== undefined && modelValue >= max)"
      aria-label="Increase"
      @click="increment"
    >
      <PlusIcon
        class="h-4 w-4 transition-colors"
        :class="
          disabled || (max !== undefined && modelValue >= max)
            ? 'text-base-content/30'
            : 'text-success'
        "
      />
    </Button>
  </Join>
</template>
