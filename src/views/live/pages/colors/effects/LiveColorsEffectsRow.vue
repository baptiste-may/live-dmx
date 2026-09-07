<script setup lang="ts">
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import type { FixtureGroup } from "@/stores/show/show-groups.ts";
import type { AlignmentType, ColorEffectType } from "@/types/live.ts";
import { colorEffects } from "@/types/live.ts";
import LiveEffectsRow from "@/views/live/components/LiveEffectsRow.vue";

defineProps<{
  group: FixtureGroup;
}>();

const liveButtons = useLiveButtons();
</script>

<template>
  <LiveEffectsRow
    :group-name="group.name"
    :effects="colorEffects"
    :current-effect="liveButtons.getGroupColorEffect(group.id).effect"
    :current-alignment="liveButtons.getGroupColorEffect(group.id).alignment"
    :is-bounce="liveButtons.getGroupColorEffect(group.id).bounce"
    @select-effect="liveButtons.setGroupColorEffectType(group.id, $event as ColorEffectType | null)"
    @select-alignment="
      liveButtons.setGroupColorEffectAlignment(group.id, $event as AlignmentType | null)
    "
    @toggle-bounce="liveButtons.setGroupColorEffectBounce(group.id, $event)"
    @reset="liveButtons.resetGroupColorEffect(group.id)"
  />
</template>
