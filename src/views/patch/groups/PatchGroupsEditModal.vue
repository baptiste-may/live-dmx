<script setup lang="ts">
import { Button, Fieldset, Input, Join, Modal, ModalAction, ModalBox } from "daisy-ui-kit";
import { XIcon } from "lucide-vue-next";
import { computed } from "vue";

import NumberStepper from "@/components/inputs/NumberStepper.vue";
import { usePatchGroups } from "@/stores/patch/patch-groups";
import { useShowGroups } from "@/stores/show/show-groups";

const patchGroups = usePatchGroups();
const showGroups = useShowGroups();

const fixturesS = computed(() => (patchGroups.table.selectedFixtures.length > 1 ? "s" : ""));
</script>

<template>
  <Modal v-if="patchGroups.table.modal.isOpen" v-model="patchGroups.table.modal.isOpen">
    <ModalBox class="w-fit">
      <h3 class="text-lg font-bold">
        Edit {{ patchGroups.table.selectedFixtures.length }} fixture{{ fixturesS }}
      </h3>
      <Fieldset :legend="`Fixture${fixturesS} name`">
        <Join>
          <Input v-model="patchGroups.table.modal.nameInput" join />
          <Button primary join @click="patchGroups.updateSelectedFixturesName">Update</Button>
        </Join>
      </Fieldset>
      <div class="flex gap-2">
        <Fieldset legend="Start address">
          <NumberStepper v-model="patchGroups.table.modal.startAddress" :min="1" :max="512" />
          <Button primary join @click="patchGroups.updateSelectedFixturesAddress">Update</Button>
        </Fieldset>
        <Fieldset legend="Start universe">
          <NumberStepper v-model="patchGroups.table.modal.startUniverse" :min="1" :max="512" />
          <Button primary join @click="patchGroups.updateSelectedFixturesUniverse">Update</Button>
        </Fieldset>
      </div>
      <Fieldset :legend="`Fixture${fixturesS} groups`">
        <div class="flex flex-wrap gap-2">
          <Button
            v-for="group in showGroups.groups.filter((g) => g.id !== 0)"
            :key="group.id"
            primary
            :outline="!patchGroups.isGroupSelected(group.id)"
            class="w-min whitespace-nowrap"
            @click="patchGroups.toggleFixtureGroup(group.id)"
          >
            {{ group.id }} - {{ group.name }}
          </Button>
        </div>
      </Fieldset>
      <ModalAction>
        <Button primary @click="patchGroups.table.modal.isOpen = false">
          <XIcon class="mr-1 h-4 w-4" />
          Close
        </Button>
      </ModalAction>
    </ModalBox>
  </Modal>
</template>
