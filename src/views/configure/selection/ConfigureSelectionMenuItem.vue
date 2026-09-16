<script setup lang="ts">
import { MenuItem } from "daisy-ui-kit";
import { computed } from "vue";

import { useShowConfiguration } from "@/stores/show/show-configuration.ts";

const props = defineProps<{ groupId: number }>();
const configure = useShowConfiguration();
const group = computed(() => configure.getGroupRow(props.groupId));
</script>

<template>
  <MenuItem class="w-full">
    <button
      type="button"
      class="flex w-full items-center justify-between rounded-lg text-left transition-colors"
      :class="
        configure.selectedGroupId === group?.id
          ? 'bg-primary text-primary-content'
          : 'hover:bg-base-300'
      "
      :aria-pressed="configure.selectedGroupId === group?.id"
      @click="configure.selectGroup(props.groupId)"
    >
      <span class="truncate">{{ group?.id === 0 ? "" : `${group?.id} - ` }}{{ group?.name }}</span>
      <span class="text-xs opacity-70">{{ group?.fixtureCount }}</span>
    </button>
    <ul v-if="group?.children.length">
      <ConfigureSelectionMenuItem
        v-for="child in group.children"
        :key="child.id"
        :group-id="child.id"
      />
    </ul>
  </MenuItem>
</template>
