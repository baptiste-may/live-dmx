<script setup lang="ts">
import { Checkbox, Table } from "daisy-ui-kit";
import { computed } from "vue";

import { usePatchGroups } from "@/stores/patch/patch-groups";
import { useShowPatch } from "@/stores/show/show-patch";
import PatchGroupsTableForm from "@/views/patch/groups/table/PatchGroupsTableForm.vue";
import PatchGroupsTableModal from "@/views/patch/groups/table/PatchGroupsTableModal.vue";
import PatchGroupsTableRow from "@/views/patch/groups/table/PatchGroupsTableRow.vue";

const showPatch = useShowPatch();
const patchGroups = usePatchGroups();

const displayedFixtures = computed(() => {
  if (patchGroups.selectedGroup === 0) {
    return showPatch.fixtures;
  }
  return showPatch.fixtures.filter((f) => f.groups.includes(patchGroups.selectedGroup));
});

const isAllSelected = computed(() => {
  if (displayedFixtures.value.length === 0) return false;
  return displayedFixtures.value.every((f) => patchGroups.table.selectedFixtures.includes(f.id));
});

function handleToggleSelectAll(checked: boolean) {
  const selectedSet = new Set(patchGroups.table.selectedFixtures);
  if (checked) {
    displayedFixtures.value.forEach((f) => selectedSet.add(f.id));
  } else {
    displayedFixtures.value.forEach((f) => selectedSet.delete(f.id));
  }
  patchGroups.table.selectedFixtures = Array.from(selectedSet);
}

function handleUpdateSelection(fixtureId: number, checked: boolean) {
  const selectedSet = new Set(patchGroups.table.selectedFixtures);
  if (checked) {
    selectedSet.add(fixtureId);
  } else {
    selectedSet.delete(fixtureId);
  }
  patchGroups.table.selectedFixtures = Array.from(selectedSet);
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col gap-4">
    <div class="min-h-0 flex-1 overflow-y-auto">
      <Table class="w-full">
        <thead class="bg-base-200 sticky top-0 z-10">
          <tr>
            <th>
              <div class="flex items-center gap-2">
                <Checkbox
                  :model-value="isAllSelected"
                  :disabled="displayedFixtures.length === 0"
                  @update:model-value="handleToggleSelectAll"
                />
              </div>
            </th>
            <th>ID</th>
            <th>Name</th>
            <th>Model</th>
            <th>Patch</th>
            <th>Groups</th>
            <th>
              <PatchGroupsTableForm :selected-fixtures="patchGroups.table.selectedFixtures" />
            </th>
          </tr>
        </thead>
        <tbody>
          <PatchGroupsTableRow
            v-for="patch in displayedFixtures"
            :key="patch.id"
            :patch="patch"
            :is-selected="patchGroups.table.selectedFixtures.includes(patch.id)"
            @update-selection="handleUpdateSelection"
          />
        </tbody>
      </Table>
    </div>
  </div>
  <PatchGroupsTableModal />
</template>
