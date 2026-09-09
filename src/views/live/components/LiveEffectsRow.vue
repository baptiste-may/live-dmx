<script setup lang="ts">
import { Button, Divider } from "daisy-ui-kit";
import { VolleyballIcon } from "lucide-vue-next";
import type { Component } from "vue";

import Knob from "@/components/inputs/Knob.vue";
import ToggleButton from "@/components/inputs/ToggleButton.vue";
import type { AlignmentType } from "@/types/live";
import { alignments } from "@/types/live";

withDefaults(
  defineProps<{
    groupName: string;
    effects: Record<string, { label: string; icon: Component }>;
    currentEffect: string | null;
    currentAlignment: AlignmentType | null;
    isBounce: boolean;
    width?: number;
  }>(),
  {
    width: 100,
  },
);

const emit = defineEmits<{
  (e: "select-effect", effect: string | null): void;
  (e: "select-alignment", alignment: AlignmentType | null): void;
  (e: "toggle-bounce", bounce: boolean): void;
  (e: "set-width", width: number): void;
  (e: "reset"): void;
}>();
</script>

<template>
  <tr>
    <th class="max-w-48 truncate px-2 text-end whitespace-nowrap" :title="groupName">
      {{ groupName }} :
    </th>
    <td v-for="(effect, key) in effects" :key="key" class="px-1">
      <ToggleButton
        :icon="effect.icon"
        :label="effect.label"
        :model-value="currentEffect === key"
        @update:model-value="emit('select-effect', $event ? (key as string) : null)"
      />
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td v-for="(alignment, key) in alignments" :key="key" class="px-1">
      <ToggleButton
        :icon="alignment.icon"
        :label="alignment.label"
        :model-value="currentAlignment === (key as AlignmentType)"
        color="secondary"
        @update:model-value="emit('select-alignment', $event ? (key as AlignmentType) : null)"
      />
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td class="px-1">
      <ToggleButton
        :icon="VolleyballIcon"
        label="Bounce"
        :model-value="isBounce"
        color="success"
        @update:model-value="emit('toggle-bounce', $event)"
      />
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td class="px-1">
      <div class="flex items-center justify-center">
        <Knob
          aria-label="Width"
          title="Width"
          size="sm"
          unit="%"
          :model-value="width"
          :min="0"
          :max="100"
          :default-value="100"
          @update:model-value="emit('set-width', $event)"
        />
      </div>
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td class="px-1">
      <Button neutral sm square class="h-12 w-12 rounded-lg" @click="emit('reset')"> Reset </Button>
    </td>
  </tr>
</template>
