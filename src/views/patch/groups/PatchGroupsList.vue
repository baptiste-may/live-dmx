<script setup lang="ts">
import { Button, Input, Join } from "daisy-ui-kit";
import { FolderPlusIcon } from "lucide-vue-next";
import { ref } from "vue";

import { usePatchGroups } from "@/stores/patch/patch-groups.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";
import { ToastType, useToastStore } from "@/stores/toast";
import PatchGroupsListItem from "@/views/patch/groups/list/PatchGroupsListItem.vue";

const showGroups = useShowGroups();
const patchGroups = usePatchGroups();

const newGroupName = ref("");

async function handleNewGroup() {
  try {
    const groupId = await showGroups.create(newGroupName.value);
    patchGroups.select(groupId);
    newGroupName.value = "";
  } catch (error: unknown) {
    useToastStore().addToast(`Failed to create group: ${String(error)}`, ToastType.ERROR);
  }
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <Join>
      <Input v-model="newGroupName" placeholder="New group name..." join class="w-full" />
      <Button success join @click="handleNewGroup">
        <FolderPlusIcon class="h-4 w-4" />
        Add Group
      </Button>
    </Join>

    <h2 class="text-base-content/60 text-xs font-semibold tracking-wider uppercase">
      Groups ({{ showGroups.groups.length }})
    </h2>

    <div class="flex flex-wrap gap-2">
      <PatchGroupsListItem v-for="group in showGroups.groups" :key="group.id" :group="group" />
    </div>
  </div>
</template>
