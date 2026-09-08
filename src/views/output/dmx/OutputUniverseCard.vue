<script setup lang="ts">
import { Badge } from "daisy-ui-kit";
import { computed } from "vue";

const props = defineProps<{
  universe: number;
  output: string;
  enabled: boolean;
  channels: number[];
}>();

const targetLabel = computed(() => {
  if (!props.enabled) return "Disabled";
  if (props.output === "artnet") return "Art-Net";
  if (props.output === "sacn") return "sACN";
  if (props.output === "osc") return "OSC";
  if (props.output === "kinet") return "KiNET";
  if (props.output === "opc") return "OPC";
  if (props.output === "none") return "None";
  return props.output;
});
</script>

<template>
  <div
    class="border-base-300 bg-base-100/50 rounded-box flex aspect-square w-64 shrink-0 flex-col gap-2.5 border p-3 shadow-xs"
  >
    <!-- Card Header -->
    <div class="flex items-center justify-between gap-2">
      <span class="text-sm font-bold">Universe {{ universe }}</span>

      <Badge
        :primary="enabled && output !== 'none'"
        :neutral="!enabled || output === 'none'"
        soft
        size="xs"
        class="font-medium"
      >
        {{ targetLabel }}
      </Badge>
    </div>

    <!-- 512-channel Visualizer Grid filling the card -->
    <div
      class="border-base-300/60 bg-base-300/30 grid min-h-0 w-full flex-1 gap-[1px] rounded border p-1.5"
      style="
        grid-template-columns: repeat(32, minmax(0, 1fr));
        grid-template-rows: repeat(16, minmax(0, 1fr));
      "
    >
      <div
        v-for="(val, idx) in channels"
        :key="idx"
        class="h-full w-full rounded-[0.5px] transition-colors duration-75"
        :class="[val > 0 ? 'bg-primary' : 'bg-base-content/15']"
        :style="{ opacity: val > 0 ? 0.35 + (val / 255) * 0.65 : 0.5 }"
        :title="`CH ${idx + 1}: ${val}`"
      />
    </div>
  </div>
</template>
