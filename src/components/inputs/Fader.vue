<script setup lang="ts">
import { useElementSize } from "@vueuse/core";
import { Range } from "daisy-ui-kit";
import { useTemplateRef } from "vue";

defineOptions({
  inheritAttrs: false,
});

const modelValue = defineModel<number>({ default: 0 });

const faderBox = useTemplateRef("fader-box");
const { height } = useElementSize(faderBox);
</script>

<template>
  <div ref="fader-box" class="relative h-full w-8">
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
</template>
