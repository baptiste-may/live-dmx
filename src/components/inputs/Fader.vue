<script setup lang="ts">
import { useElementSize } from "@vueuse/core";
import { Range } from "daisy-ui-kit";
import { computed, useAttrs, useTemplateRef } from "vue";

defineOptions({
  inheritAttrs: false,
});

const props = withDefaults(
  defineProps<{
    showScale?: boolean;
    scaleStep?: number;
    scaleUnit?: string;
  }>(),
  {
    scaleStep: 25,
    scaleUnit: "",
  },
);

const attrs = useAttrs();

const modelValue = defineModel<number>({ default: 0 });

const faderBox = useTemplateRef("fader-box");
const { height } = useElementSize(faderBox);

const scaleTicks = computed(() => {
  if (!props.showScale || !(props.scaleStep > 0)) return [];
  const min = Number(attrs.min ?? 0);
  const max = Number(attrs.max ?? 100);
  const ticks: number[] = [];
  for (let value = max; value >= min; value -= props.scaleStep) {
    ticks.push(value);
  }
  return ticks;
});

const thumbOffset = computed(() => {
  if (!props.showScale) return 0;
  const input = faderBox.value?.querySelector<HTMLInputElement>("input[type='range']");
  const size = input
    ? Number.parseFloat(getComputedStyle(input).getPropertyValue("--range-thumb-size"))
    : 0;
  const thumb = Number.isFinite(size) && size > 0 ? size : 32;
  return (thumb * 1.5) / 2;
});
</script>

<template>
  <div class="flex min-h-0 flex-1 items-stretch">
    <div ref="fader-box" class="relative min-h-0 w-8">
      <Range
        v-model="modelValue"
        v-bind="$attrs"
        xl
        class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 scale-150 -rotate-90"
        :style="{
          width: `${height / 1.5}px`,
        }"
      />
    </div>
    <div
      v-if="showScale"
      :style="{ paddingTop: `${thumbOffset}px`, paddingBottom: `${thumbOffset}px` }"
      class="flex w-14 flex-col justify-between text-left text-xs leading-none tabular-nums opacity-70 select-none"
    >
      <div v-for="tick in scaleTicks" :key="tick" class="flex items-center gap-2">
        <span class="h-px w-4 bg-current opacity-60" />
        <span>{{ tick }}{{ scaleUnit }}</span>
      </div>
    </div>
  </div>
</template>
