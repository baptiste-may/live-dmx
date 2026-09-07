<script setup lang="ts">
import { Button, Menu, MenuItem, MenuTitle } from "daisy-ui-kit";
import {
  FolderOpenIcon,
  PanelLeftCloseIcon,
  PanelLeftOpenIcon,
  PenLineIcon,
  PlusIcon,
  RedoIcon,
  SaveIcon,
  UndoIcon,
  XIcon,
} from "lucide-vue-next";
import { ref } from "vue";

import TextForm from "@/components/inputs/TextForm.vue";
import { useApp } from "@/stores/app";
import { EVERY_PAGES, useNavigation } from "@/stores/navigation";
import { useShow } from "@/stores/show/show";
import { useShowChanges } from "@/stores/show/show-changes";
import { useShowIdentity } from "@/stores/show/show-identity.ts";

const app = useApp();
const show = useShow();
const changes = useShowChanges();
const identity = useShowIdentity();
const navigation = useNavigation();

const isCollapsed = ref(false);

function toggleSidebar() {
  isCollapsed.value = !isCollapsed.value;
  if (isCollapsed.value) {
    identity.nameForm.isEditing = false;
  }
}

function startEditingShowName() {
  identity.nameForm.nameInput = show.identity.displayName;
  identity.nameForm.isEditing = true;
}

function handleSaveShowName(newName: string) {
  const trimmed = newName.trim();
  if (trimmed) {
    show.identity.updateDisplayName(trimmed);
  }
  identity.nameForm.isEditing = false;
}

function handleCancelShowName() {
  identity.nameForm.isEditing = false;
}
</script>

<template>
  <Menu
    class="bg-base-200 rounded-r-box flex h-full flex-col transition-[width] duration-200"
    :class="isCollapsed ? 'w-16' : 'w-64'"
  >
    <MenuTitle v-if="!isCollapsed" class="flex flex-col gap-0 pr-6">
      <span
        v-if="!identity.nameForm.isEditing"
        class="text-base-content flex items-center gap-2 text-2xl whitespace-nowrap"
      >
        {{ show.identity.displayName }}
        <Button primary soft square sm aria-label="Edit show name" @click="startEditingShowName">
          <PenLineIcon class="h-4 w-4" />
        </Button>
      </span>
      <TextForm
        v-else
        v-model="identity.nameForm.nameInput"
        @save="handleSaveShowName"
        @cancel="handleCancelShowName"
      />
      <span v-if="show.hasUnsavedChanges" class="text-xs font-normal normal-case italic opacity-50">
        (unsaved)
      </span>
    </MenuTitle>

    <MenuTitle v-if="!isCollapsed" class="mt-2">Show</MenuTitle>
    <MenuItem
      :class="{ 'tooltip tooltip-right w-full': isCollapsed }"
      :data-tip="isCollapsed ? 'New show' : undefined"
    >
      <button
        type="button"
        :class="{ 'justify-center': isCollapsed }"
        aria-label="New show"
        @click="app.createShow"
      >
        <PlusIcon class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">New show</span>
      </button>
    </MenuItem>
    <MenuItem
      :class="{ 'tooltip tooltip-right w-full': isCollapsed }"
      :data-tip="isCollapsed ? 'Open show' : undefined"
    >
      <button
        type="button"
        :class="{ 'justify-center': isCollapsed }"
        aria-label="Open show"
        @click="app.askLoadShow"
      >
        <FolderOpenIcon class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">Open show</span>
      </button>
    </MenuItem>
    <MenuItem
      :disabled="!show.canSave"
      :class="{ 'tooltip tooltip-right w-full': isCollapsed }"
      :data-tip="isCollapsed ? 'Save show' : undefined"
    >
      <button
        type="button"
        :disabled="!show.canSave"
        :class="{ 'justify-center': isCollapsed }"
        aria-label="Save show"
        @click="() => changes.save()"
      >
        <SaveIcon class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">Save show</span>
      </button>
    </MenuItem>
    <MenuItem
      :disabled="!show.canUndo"
      :class="{ 'tooltip tooltip-right w-full': isCollapsed }"
      :data-tip="isCollapsed ? 'Undo' : undefined"
    >
      <button
        type="button"
        :disabled="!show.canUndo"
        :class="{ 'justify-center': isCollapsed }"
        aria-label="Undo"
        @click="() => changes.undo()"
      >
        <UndoIcon class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">Undo</span>
      </button>
    </MenuItem>
    <MenuItem
      :disabled="!show.canRedo"
      :class="{ 'tooltip tooltip-right w-full': isCollapsed }"
      :data-tip="isCollapsed ? 'Redo' : undefined"
    >
      <button
        type="button"
        :disabled="!show.canRedo"
        :class="{ 'justify-center': isCollapsed }"
        aria-label="Redo"
        @click="() => changes.redo()"
      >
        <RedoIcon class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">Redo</span>
      </button>
    </MenuItem>
    <MenuItem
      :class="{ 'tooltip tooltip-right w-full': isCollapsed }"
      :data-tip="isCollapsed ? 'Close show' : undefined"
    >
      <button
        type="button"
        :class="{ 'justify-center': isCollapsed }"
        aria-label="Close show"
        @click="app.closeShow"
      >
        <XIcon class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">Close show</span>
      </button>
    </MenuItem>

    <MenuTitle v-if="!isCollapsed" class="mt-4">Windows</MenuTitle>
    <li v-if="isCollapsed" class="border-base-content/10 my-1 border-t" aria-hidden="true" />
    <MenuItem
      v-for="{ id, name, icon } in EVERY_PAGES"
      :key="id"
      :class="[
        { 'menu-active': id === navigation.currentPage },
        { 'tooltip tooltip-right w-full': isCollapsed },
      ]"
      :data-tip="isCollapsed ? name : undefined"
    >
      <button
        type="button"
        :class="[{ active: id === navigation.currentPage }, { 'justify-center': isCollapsed }]"
        :aria-label="name"
        @click="navigation.navigate(id)"
      >
        <component :is="icon" class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">{{ name }}</span>
      </button>
    </MenuItem>

    <MenuItem
      class="mt-auto"
      :class="{ 'tooltip tooltip-right w-full': isCollapsed }"
      :data-tip="isCollapsed ? 'Show sidebar' : undefined"
    >
      <button
        type="button"
        :class="{ 'justify-center': isCollapsed }"
        :aria-label="isCollapsed ? 'Show sidebar' : 'Hide sidebar'"
        @click="toggleSidebar"
      >
        <PanelLeftOpenIcon v-if="isCollapsed" class="h-4 w-4 shrink-0" />
        <PanelLeftCloseIcon v-else class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">Hide</span>
      </button>
    </MenuItem>
  </Menu>
</template>

<style scoped>
@starting-style {
  :deep(.tooltip:before),
  :deep(.tooltip:after) {
    opacity: 0;
  }
}

:deep(.tooltip:not(:hover):not(:focus-visible):before),
:deep(.tooltip:not(:hover):not(:focus-visible):after) {
  opacity: 0 !important;
  transition: none !important;
}
</style>
