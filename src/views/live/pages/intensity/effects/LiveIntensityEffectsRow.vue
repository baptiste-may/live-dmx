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
        :model-value="liveButtons.getGroupEffect(group.id).effect === key"
        @update:model-value="
          liveButtons.setGroupEffectType(group.id, $event ? (key as IntensityEffectType) : null)
        "
      >
        <component :is="effect.icon" class="h-4 w-4" />
        {{ effect.label }}
      </ToggleButton>
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td v-for="(alignment, key) in alignments" :key="key" class="px-1">
      <ToggleButton
        :model-value="liveButtons.getGroupEffect(group.id).alignment === key"
        color="secondary"
        @update:model-value="
          liveButtons.setGroupEffectAlignment(group.id, $event ? (key as AlignmentType) : null)
        "
      >
        <component :is="alignment.icon" class="h-4 w-4" />
        {{ alignment.label }}
      </ToggleButton>
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td class="px-1">
      <ToggleButton
        :model-value="liveButtons.getGroupEffect(group.id).bounce"
        color="success"
        @update:model-value="liveButtons.setGroupEffectBounce(group.id, $event)"
      >
        <VolleyballIcon class="h-4 w-4" />
        Bounce
      </ToggleButton>
    </td>
    <td class="px-1">
      <Divider neutral horizontal class="mx-1 h-6 w-auto" />
    </td>
    <td class="px-1">
      <Button neutral sm @click="handleEffectsReset">Reset</Button>
    </td>
  </tr>
</template>
