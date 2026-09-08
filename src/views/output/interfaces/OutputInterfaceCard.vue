<script setup lang="ts">
import { Badge } from "daisy-ui-kit";
import { CheckCircle2Icon, UsbIcon } from "lucide-vue-next";

import type { PhysicalInterface } from "@/types/output.ts";

withDefaults(
  defineProps<{
    item: PhysicalInterface;
    assignedUniverses?: number[];
  }>(),
  {
    assignedUniverses: () => [],
  },
);
</script>

<template>
  <div
    class="bg-base-100 border-base-300 rounded-box flex flex-col gap-2 border p-3.5 shadow-sm transition-all"
  >
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <UsbIcon class="text-base-content/70 h-4 w-4 shrink-0" />
        <span class="font-bold">{{ item.name }}</span>
      </div>

      <Badge v-if="item.status === 'connected'" success soft size="sm"> Connected </Badge>
      <Badge v-else-if="item.status === 'busy'" warning soft size="sm"> Busy </Badge>
      <Badge v-else error soft size="sm"> Disconnected </Badge>
    </div>

    <div class="flex flex-col gap-1">
      <span class="text-sm font-medium">{{ item.device }}</span>
      <div class="text-base-content/60 flex items-center gap-2 text-xs">
        <span
          >Port: <code class="font-mono">{{ item.port }}</code></span
        >
        <span>•</span>
        <span>{{ item.fps }} FPS</span>
      </div>
    </div>

    <div class="border-base-300/60 mt-1 border-t pt-2">
      <div
        v-if="assignedUniverses.length > 0"
        class="text-info flex items-center gap-1.5 text-xs font-medium"
      >
        <CheckCircle2Icon class="h-3.5 w-3.5 shrink-0" />
        <span>Routing Universe {{ assignedUniverses.join(", ") }}</span>
      </div>
      <div v-else class="text-base-content/40 text-xs italic">Not assigned to any universe</div>
    </div>
  </div>
</template>
