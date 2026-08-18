<script setup lang="ts">
import { useElementSize } from "@vueuse/core";
import { Badge } from "daisy-ui-kit";
import { computed, ref } from "vue";

import { usePatchVisualizer } from "@/stores/patch/patch-visualizer";
import { useShowPatch } from "@/stores/show/show-patch";

const TOTAL_ITEMS = 512;
const ITEM_HEIGHT_PX = 24; // w-6 / h-6 = 24px (1.5rem)
const GAP_PX = 2; // gap-0.5 (2px)

const showPatch = useShowPatch();
const visualizer = usePatchVisualizer();

const grid = ref<HTMLDivElement | null>(null);
const { height } = useElementSize(grid);

const columns = computed(() => {
  const h = height.value;

  if (!h || h <= 0) {
    return 32;
  }

  const maxRows = Math.max(1, Math.floor((h + GAP_PX) / (ITEM_HEIGHT_PX + GAP_PX)));
  return Math.max(1, Math.ceil(TOTAL_ITEMS / maxRows));
});

const sections = computed(
  (): {
    x: number;
    y: number;
    size: number;
    id: number;
    name: string;
  }[] => {
    const cols = columns.value;
    const currentUniverse = visualizer.currentUniverse;
    const result: {
      x: number;
      y: number;
      size: number;
      id: number;
      name: string;
    }[] = [];

    const universeFixtures = showPatch.fixtures.filter((f) => f.universe === currentUniverse);

    for (const f of universeFixtures) {
      const channelCount = f.mode?.channels?.length ?? 1;
      let startAddr0 = f.address - 1;
      let remaining = channelCount;

      while (remaining > 0 && startAddr0 < TOTAL_ITEMS) {
        const x = startAddr0 % cols;
        const y = Math.floor(startAddr0 / cols);
        const segmentSize = Math.min(remaining, cols - x);

        result.push({
          x,
          y,
          size: segmentSize,
          id: f.id,
          name: f.name,
        });

        remaining -= segmentSize;
        startAddr0 += segmentSize;
      }
    }

    return result;
  },
);
</script>

<template>
  <div
    ref="grid"
    class="relative grid h-full min-h-0 w-full gap-0.5 overflow-hidden"
    :style="{ gridTemplateColumns: `repeat(${columns}, minmax(0, max-content))` }"
  >
    <Badge
      v-for="i in TOTAL_ITEMS"
      :key="i"
      primary
      soft
      class="flex h-6 w-6 items-center justify-center rounded-none p-0 text-xs"
    >
      {{ i }}
    </Badge>
    <Badge
      v-for="(section, i) in sections"
      :key="i"
      primary
      class="absolute flex h-6 items-center justify-start overflow-hidden rounded-none text-xs whitespace-nowrap"
      :style="{
        top: `${(ITEM_HEIGHT_PX + GAP_PX) * section.y}px`,
        left: `${(ITEM_HEIGHT_PX + GAP_PX) * section.x}px`,
        width: `${(ITEM_HEIGHT_PX + GAP_PX) * section.size - GAP_PX}px`,
      }"
    >
      <span class="text-neutral/75">{{ section.id }}</span>
      <span>{{ section.name }}</span>
    </Badge>
  </div>
</template>
