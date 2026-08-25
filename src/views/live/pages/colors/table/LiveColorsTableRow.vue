<script setup lang="ts">
import ToggleButton from "@/components/inputs/ToggleButton.vue";
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import type { FixtureGroup } from "@/stores/show/show-groups";
import { colors } from "@/types/live.ts";

const props = defineProps<{
  group: FixtureGroup;
}>();

const liveButtons = useLiveButtons();

function handleColorSelect(colorKey: string, active: boolean) {
  liveButtons.setGroupColor(props.group.id, active ? colorKey : null);
}

function handleSecondaryColorSelect(colorKey: string, active: boolean) {
  liveButtons.setGroupSecondaryColor(props.group.id, active ? colorKey : null);
}
</script>

<template>
  <tr>
    <th class="px-2 text-end">{{ group.name }} :</th>
    <td v-for="(color, key) in colors" :key="key" class="px-1">
      <div class="flex flex-col items-center gap-2">
        <ToggleButton
          cell
          :color="color"
          class="h-12 w-12"
          :model-value="liveButtons.getGroupColor(group.id) === key"
          @update:model-value="handleColorSelect(key as string, $event)"
        />
        <ToggleButton
          cell
          :color="color"
          class="h-8 w-8"
          :model-value="liveButtons.getGroupSecondaryColor(group.id) === key"
          @update:model-value="handleSecondaryColorSelect(key as string, $event)"
        />
      </div>
    </td>
  </tr>
</template>
