<script setup lang="ts">
import { Button, Divider } from "daisy-ui-kit";
import { VolleyballIcon } from "lucide-vue-next";

import ToggleButton from "@/components/inputs/ToggleButton.vue";
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import type { FixtureGroup } from "@/stores/show/show-groups.ts";
import type { AlignmentType, ColorEffectType } from "@/types/live.ts";
import { alignments, colorEffects } from "@/types/live.ts";

const props = defineProps<{
  group: FixtureGroup;
}>();

const liveButtons = useLiveButtons();

function handleEffectsReset() {
  liveButtons.resetGroupColorEffect(props.group.id);
}
</script>

<template>
  <tr>
    <th class="px-2 text-end">{{ group.name }} :</th>
    <td v-for="(effect, key) in colorEffects" :key="key" class="px-1">
      <ToggleButton
        :icon="effect.icon"
        :label="effect.label"
        :model-value="liveButtons.getGroupColorEffect(group.id).effect === key"
        @update:model-value="
          liveButtons.setGroupColorEffectType(group.id, $event ? (key as ColorEffectType) : null)
        "
      />
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td v-for="(alignment, key) in alignments" :key="key" class="px-1">
      <ToggleButton
        :icon="alignment.icon"
        :label="alignment.label"
        :model-value="liveButtons.getGroupColorEffect(group.id).alignment === key"
        color="secondary"
        @update:model-value="
          liveButtons.setGroupColorEffectAlignment(group.id, $event ? (key as AlignmentType) : null)
        "
      />
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td class="px-1">
      <ToggleButton
        :icon="VolleyballIcon"
        label="Bounce"
        :model-value="liveButtons.getGroupColorEffect(group.id).bounce"
        color="success"
        @update:model-value="liveButtons.setGroupColorEffectBounce(group.id, $event)"
      />
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td class="px-1">
      <Button neutral sm square class="h-12 w-12 rounded-lg" @click="handleEffectsReset">
        Reset
      </Button>
    </td>
  </tr>
</template>
