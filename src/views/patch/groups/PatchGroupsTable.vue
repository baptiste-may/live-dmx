<script setup lang="ts">
import { Checkbox, Table } from "daisy-ui-kit";
import { computed } from "vue";

import { usePatchGroups } from "@/stores/patch/patch-groups";
import { useShowPatch } from "@/stores/show/show-patch";
import PatchGroupsEditModal from "@/views/patch/groups/PatchGroupsEditModal.vue";
import PatchGroupsTableActions from "@/views/patch/groups/table/PatchGroupsTableActions.vue";
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

function toggleFixtureInSelection(set: Set<number>, id: number, checked: boolean) {
  if (checked) {
    set.add(id);
  } else {
    set.delete(id);
  }
}

function handleToggleSelectAll(checked: boolean) {
  const selectedSet = new Set(patchGroups.table.selectedFixtures);
  displayedFixtures.value.forEach((f) => toggleFixtureInSelection(selectedSet, f.id, checked));
  patchGroups.table.selectedFixtures = Array.from(selectedSet);
}

function handleUpdateSelection(fixtureId: number, checked: boolean) {
  const selectedSet = new Set(patchGroups.table.selectedFixtures);
  toggleFixtureInSelection(selectedSet, fixtureId, checked);
  patchGroups.table.selectedFixtures = Array.from(selectedSet);
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col gap-4">
    <div class="min-h-0 flex-1 overflow-auto">
      <Table class="w-full min-w-[42rem]">
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
              <PatchGroupsTableActions :selected-fixtures="patchGroups.table.selectedFixtures" />
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
  <PatchGroupsEditModal />
</template>
