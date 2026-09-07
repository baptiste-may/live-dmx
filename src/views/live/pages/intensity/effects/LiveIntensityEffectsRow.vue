<script setup lang="ts">
import { Button, Divider } from "daisy-ui-kit";
import { VolleyballIcon } from "lucide-vue-next";

import ToggleButton from "@/components/inputs/ToggleButton.vue";
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import type { FixtureGroup } from "@/stores/show/show-groups";
import type { AlignmentType, IntensityEffectType } from "@/types/live";
import { alignments, intensityEffects } from "@/types/live";

const props = defineProps<{
  group: FixtureGroup;
}>();

const liveButtons = useLiveButtons();

function handleEffectsReset() {
  liveButtons.resetGroupEffect(props.group.id);
}
</script>

<template>
  <tr>
    <th class="px-2 text-end">{{ group.name }} :</th>
    <td v-for="(effect, key) in intensityEffects" :key="key" class="px-1">
      <ToggleButton
        :icon="effect.icon"
        :label="effect.label"
        :model-value="liveButtons.getGroupEffect(group.id).effect === key"
        @update:model-value="
          liveButtons.setGroupEffectType(group.id, $event ? (key as IntensityEffectType) : null)
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
        :model-value="liveButtons.getGroupEffect(group.id).alignment === key"
        color="secondary"
        @update:model-value="
          liveButtons.setGroupEffectAlignment(group.id, $event ? (key as AlignmentType) : null)
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
        :model-value="liveButtons.getGroupEffect(group.id).bounce"
        color="success"
        @update:model-value="liveButtons.setGroupEffectBounce(group.id, $event)"
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
