<script setup lang="ts">
import { Button, Join } from "daisy-ui-kit";
import { SlidersIcon, TrashIcon } from "lucide-vue-next";
import { computed } from "vue";

import { useConfirmation } from "@/stores/confirmation";
import { usePatchGroups } from "@/stores/patch/patch-groups.ts";
import { useShowPatch } from "@/stores/show/show-patch";

const { selectedFixtures } = defineProps<{
  selectedFixtures: number[];
}>();

const showPatch = useShowPatch();
const patchGroups = usePatchGroups();
const confirmation = useConfirmation();

const confirmationMsg = computed(() => {
  let fixtureMsg: string;
  if (selectedFixtures.length === 1) {
    fixtureMsg = `the fixture '${showPatch.getFixture(selectedFixtures[0])?.name}'`;
  } else {
    fixtureMsg = `these ${selectedFixtures.length} fixtures`;
  }
  return `Are you sure you want to delete ${fixtureMsg}?`;
});
</script>

<template>
  <Join>
    <Button
      sm
      join
      square
      warning
      :disabled="selectedFixtures.length === 0"
      @click="patchGroups.openModal(selectedFixtures)"
    >
      <SlidersIcon class="h-3.5 w-3.5" />
    </Button>
    <Button
      sm
      join
      square
      error
      :disabled="selectedFixtures.length === 0"
      @click="
        confirmation.ask(confirmationMsg, () => {
          showPatch.removeFixtures(selectedFixtures);
          patchGroups.table.selectedFixtures = patchGroups.table.selectedFixtures.filter(
            (id) => !selectedFixtures.includes(id),
          );
        })
      "
    >
      <TrashIcon class="h-3.5 w-3.5" />
    </Button>
  </Join>
</template>
