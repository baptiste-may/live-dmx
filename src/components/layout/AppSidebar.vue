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
  SettingsIcon,
  UndoIcon,
  XIcon,
} from "lucide-vue-next";
import { computed, ref } from "vue";

import TextForm from "@/components/inputs/TextForm.vue";
import { useApp } from "@/stores/app";
import { NAVIGATION_SECTIONS, useNavigation } from "@/stores/navigation";
import { useSettings } from "@/stores/settings.ts";
import { useShow } from "@/stores/show/show";
import { useShowChanges } from "@/stores/show/show-changes";
import { useShowIdentity } from "@/stores/show/show-identity.ts";

const app = useApp();
const show = useShow();
const changes = useShowChanges();
const identity = useShowIdentity();
const navigation = useNavigation();
const settings = useSettings();

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

const showActions = computed(() => [
  {
    label: "New show",
    icon: PlusIcon,
    disabled: false,
    handler: () => app.createShow(),
  },
  {
    label: "Open show",
    icon: FolderOpenIcon,
    disabled: false,
    handler: () => app.askLoadShow(),
  },
  {
    label: "Save show",
    icon: SaveIcon,
    disabled: !show.canSave,
    handler: () => changes.save(),
  },
  {
    label: "Undo",
    icon: UndoIcon,
    disabled: !show.canUndo,
    handler: () => changes.undo(),
  },
  {
    label: "Redo",
    icon: RedoIcon,
    disabled: !show.canRedo,
    handler: () => changes.redo(),
  },
  {
    label: "Close show",
    icon: XIcon,
    disabled: false,
    handler: () => app.closeShow(),
  },
]);
</script>

<template>
  <Menu
    class="bg-base-200 rounded-r-box flex h-full flex-col px-0 transition-[width] duration-200"
    :class="isCollapsed ? 'w-16' : 'w-64'"
  >
    <MenuTitle v-if="!isCollapsed" class="mx-1 flex flex-col gap-0 pr-8">
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

    <MenuTitle v-if="!isCollapsed" class="mx-2 mt-2">Show</MenuTitle>
    <MenuItem
      v-for="action in showActions"
      :key="action.label"
      :disabled="action.disabled"
      :class="['px-2', { 'tooltip tooltip-right w-full': isCollapsed }]"
      :data-tip="isCollapsed ? action.label : undefined"
    >
      <button
        type="button"
        :disabled="action.disabled"
        :class="{ 'justify-center': isCollapsed }"
        :aria-label="action.label"
        @click="action.handler"
      >
        <component :is="action.icon" class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">{{ action.label }}</span>
      </button>
    </MenuItem>

    <template v-for="section in NAVIGATION_SECTIONS" :key="section.title">
      <MenuTitle v-if="!isCollapsed" class="mx-2 mt-4">{{ section.title }}</MenuTitle>
      <li v-if="isCollapsed" class="border-base-content/10 my-1 border-t" aria-hidden="true" />
      <MenuItem
        v-for="{ id, name, icon } in section.pages"
        :key="id"
        :class="['px-2', { 'tooltip tooltip-right w-full': isCollapsed }]"
        :data-tip="isCollapsed ? name : undefined"
      >
        <button
          type="button"
          :class="[
            { 'menu-active': id === navigation.currentPage },
            { 'justify-center': isCollapsed },
          ]"
          :aria-label="name"
          @click="navigation.navigate(id)"
        >
          <component :is="icon" class="h-4 w-4 shrink-0" />
          <span v-if="!isCollapsed">{{ name }}</span>
        </button>
      </MenuItem>
    </template>

    <MenuItem
      class="mt-auto px-2"
      :class="{ 'tooltip tooltip-right w-full': isCollapsed }"
      :data-tip="isCollapsed ? 'Settings' : undefined"
    >
      <button
        type="button"
        :class="{ 'justify-center': isCollapsed }"
        aria-label="Settings"
        @click="settings.openModal"
      >
        <SettingsIcon class="h-4 w-4 shrink-0" />
        <span v-if="!isCollapsed">Settings</span>
      </button>
    </MenuItem>

    <MenuItem
      class="px-2"
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
