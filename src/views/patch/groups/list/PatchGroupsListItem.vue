<script setup lang="ts">
import { Button, Join } from "daisy-ui-kit";
import { Edit2Icon, Trash2Icon } from "lucide-vue-next";
import { computed, ref } from "vue";

import TextForm from "@/components/inputs/TextForm.vue";
import { useConfirmation } from "@/stores/confirmation.ts";
import { usePatchGroups } from "@/stores/patch/patch-groups";
import { type FixtureGroup, useShowGroups } from "@/stores/show/show-groups";
import { ToastType, useToastStore } from "@/stores/toast";

const { group } = defineProps<{
  group: FixtureGroup;
}>();

const patchGroups = usePatchGroups();
const showGroups = useShowGroups();
const confirmation = useConfirmation();

const isEditing = ref(false);
const groupName = ref(group.name);

const isSelected = computed(() => patchGroups.selectedGroup === group.id);

function handleCancelEdit() {
  groupName.value = group.name;
  isEditing.value = false;
}

async function handleSaveEdit() {
  try {
    await showGroups.rename(group.id, groupName.value);
    isEditing.value = false;
  } catch (error: unknown) {
    useToastStore().addToast(`Failed to rename group: ${String(error)}`, ToastType.ERROR);
  }
}

async function handleRemove() {
  try {
    await showGroups.remove(group.id);
    if (patchGroups.selectedGroup === group.id) {
      patchGroups.selectedGroup = 0;
    }
  } catch (error: unknown) {
    useToastStore().addToast(`Failed to delete group: ${String(error)}`, ToastType.ERROR);
  }
}
</script>

<template>
  <div class="flex flex-wrap gap-1">
    <!-- Inline Editing Mode -->
    <TextForm
      v-if="isEditing"
      v-model="groupName"
      @save="handleSaveEdit"
      @cancel="handleCancelEdit"
    />

    <!-- Normal View Mode -->
    <Join v-else>
      <Button
        join
        :color="group.id === 0 ? 'secondary' : 'primary'"
        :outline="!isSelected"
        @click.stop="patchGroups.select(group.id)"
      >
        {{ group.id === 0 ? group.name : `${group.id} - ${group.name}` }}
      </Button>
      <Button v-if="isSelected && group.id !== 0" join warning soft @click.stop="isEditing = true">
        <Edit2Icon class="h-4 w-4" />
      </Button>
      <Button
        v-if="isSelected && group.id !== 0"
        join
        error
        soft
        @click.stop="
          confirmation.ask(
            `Are you sure you want to delete the group '${group.name}'?`,
            handleRemove,
          )
        "
      >
        <Trash2Icon class="h-4 w-4" />
      </Button>
    </Join>
  </div>
</template>
