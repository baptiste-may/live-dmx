<script setup lang="ts">
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import type { FixtureGroup } from "@/stores/show/show-groups";
import type { AlignmentType, IntensityEffectType } from "@/types/live";
import { intensityEffects } from "@/types/live";
import LiveEffectsRow from "@/views/live/components/LiveEffectsRow.vue";

defineProps<{
  group: FixtureGroup;
}>();

const liveButtons = useLiveButtons();
</script>

<template>
  <LiveEffectsRow
    :group-name="group.name"
    :effects="intensityEffects"
    :current-effect="liveButtons.getGroupEffect(group.id).effect"
    :current-alignment="liveButtons.getGroupEffect(group.id).alignment"
    :is-bounce="liveButtons.getGroupEffect(group.id).bounce"
    @select-effect="liveButtons.setGroupEffectType(group.id, $event as IntensityEffectType | null)"
    @select-alignment="
      liveButtons.setGroupEffectAlignment(group.id, $event as AlignmentType | null)
    "
    @toggle-bounce="liveButtons.setGroupEffectBounce(group.id, $event)"
    @reset="liveButtons.resetGroupEffect(group.id)"
  />
</template>
