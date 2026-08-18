<script setup lang="ts">
import { Button, Join } from "daisy-ui-kit";
import { Edit2Icon, Trash2Icon } from "lucide-vue-next";
import { computed, ref } from "vue";

import TextForm from "@/components/inputs/TextForm.vue";
import { useConfirmation } from "@/stores/confirmation.ts";
import type { FixtureGroup } from "@/stores/patch/patch-groups";
import { usePatchGroups } from "@/stores/patch/patch-groups";

const { group } = defineProps<{
  group: FixtureGroup;
}>();

const groups = usePatchGroups();
const confirmation = useConfirmation();

const isEditing = ref(false);
const groupName = ref(group.name);

const isSelected = computed(() => groups.selectedGroup === group.id);

function handleCancelEdit() {
  groupName.value = group.name;
  isEditing.value = false;
}

function handleSaveEdit() {
  groups.rename(group.id, groupName.value);
  isEditing.value = false;
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
        @click.stop="groups.select(group.id)"
      >
        {{ group.name }}
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
          confirmation.ask(`Are you sure you want to delete the group '${group.name}'?`, () =>
            groups.remove(group.id),
          )
        "
      >
        <Trash2Icon class="h-4 w-4" />
      </Button>
    </Join>
  </div>
</template>
