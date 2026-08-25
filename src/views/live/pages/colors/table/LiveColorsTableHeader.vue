<script setup lang="ts">
import { ArrowDownIcon } from "lucide-vue-next";
import { storeToRefs } from "pinia";

import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import { colors } from "@/types/live.ts";

const liveButtons = useLiveButtons();
const { groups } = storeToRefs(useShowGroups());

function handleSelectAll(colorKey: string) {
  const groupIds = groups.value.map((g) => g.id);
  liveButtons.setAllGroupsColor(colorKey, groupIds);
}
</script>

<template>
  <tr>
    <th></th>
    <td v-for="(color, key) in colors" :key="key" class="px-1">
      <button
        class="flex h-16 w-16 cursor-pointer items-center justify-center rounded-lg text-black transition-opacity hover:opacity-80"
        :style="{ backgroundColor: color }"
        :title="`Apply ${key} to all groups`"
        @click="handleSelectAll(key as string)"
      >
        <ArrowDownIcon class="h-6 w-6" />
      </button>
    </td>
  </tr>
</template>
