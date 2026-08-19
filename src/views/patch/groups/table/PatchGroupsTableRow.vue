<script setup lang="ts">
import { Badge, Checkbox } from "daisy-ui-kit";
import { computed } from "vue";

import { useShowGroups } from "@/stores/show/show-groups";
import type { PatchedFixture } from "@/types/patch";
import PatchGroupsTableForm from "@/views/patch/groups/table/PatchGroupsTableForm.vue";

const patchNumberFormat = new Intl.NumberFormat(undefined, {
  minimumIntegerDigits: 3,
});

const { patch } = defineProps<{
  patch: PatchedFixture;
  isSelected?: boolean;
}>();

const emit = defineEmits<{
  (e: "update-selection", fixtureId: number, checked: boolean): void;
}>();

const showGroups = useShowGroups();

const groupsData = computed(() => patch.groups.map((groupId) => showGroups.get(groupId)));
</script>

<template>
  <tr :class="isSelected ? 'bg-base-100' : ''">
    <td>
      <Checkbox
        :model-value="isSelected"
        @update:model-value="(checked: boolean) => emit('update-selection', patch.id, checked)"
      />
    </td>
    <td>{{ patch.id }}</td>
    <td>{{ patch.name }}</td>
    <td>{{ patch.fixtureType.name }}</td>
    <td>{{ patch.universe }}.{{ patchNumberFormat.format(patch.address) }}</td>
    <td class="flex flex-wrap gap-1">
      <Badge v-for="group in groupsData" :key="group?.id" xs primary>
        {{ group?.name }}
      </Badge>
    </td>
    <td>
      <PatchGroupsTableForm :selected-fixtures="[patch.id]" />
    </td>
  </tr>
</template>
