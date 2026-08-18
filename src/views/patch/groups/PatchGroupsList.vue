<script setup lang="ts">
import { Button, Input, Join } from "daisy-ui-kit";
import { FolderPlusIcon } from "lucide-vue-next";
import { ref } from "vue";

import { usePatchGroups } from "@/stores/patch/patch-groups.ts";
import PatchGroupsListItem from "@/views/patch/groups/list/PatchGroupsListItem.vue";

const groups = usePatchGroups();

const newGroupName = ref("");

async function handleNewGroup() {
  const groupId = await groups.create(newGroupName.value);
  await groups.select(groupId);
  newGroupName.value = "";
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
      Groups ({{ groups.groups.length }})
    </h2>

    <div class="flex flex-wrap gap-2">
      <PatchGroupsListItem v-for="group in groups.groups" :key="group.id" :group="group" />
    </div>
  </div>
</template>
