<script setup lang="ts">
import { Button, Menu, MenuItem, MenuTitle } from "daisy-ui-kit";
import {
  FolderOpenIcon,
  PenLineIcon,
  PlusIcon,
  RedoIcon,
  SaveIcon,
  UndoIcon,
  XIcon,
} from "lucide-vue-next";

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
  <Menu class="bg-base-200 rounded-r-box flex h-full w-64 flex-col">
    <MenuTitle class="flex flex-col gap-0 pr-6">
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

    <MenuTitle class="mt-2">Show</MenuTitle>
    <MenuItem>
      <button type="button" @click="app.createShow">
        <PlusIcon class="h-4 w-4" />
        New show
      </button>
    </MenuItem>
    <MenuItem>
      <button type="button" @click="app.askLoadShow">
        <FolderOpenIcon class="h-4 w-4" />
        Open show
      </button>
    </MenuItem>
    <MenuItem :disabled="!show.canSave">
      <button type="button" :disabled="!show.canSave" @click="() => changes.save()">
        <SaveIcon class="h-4 w-4" />
        Save show
      </button>
    </MenuItem>
    <MenuItem :disabled="!show.canUndo">
      <button type="button" :disabled="!show.canUndo" @click="() => changes.undo()">
        <UndoIcon class="h-4 w-4" />
        Undo
      </button>
    </MenuItem>
    <MenuItem :disabled="!show.canRedo">
      <button type="button" :disabled="!show.canRedo" @click="() => changes.redo()">
        <RedoIcon class="h-4 w-4" />
        Redo
      </button>
    </MenuItem>
    <MenuItem>
      <button type="button" @click="app.closeShow">
        <XIcon class="h-4 w-4" />
        Close show
      </button>
    </MenuItem>

    <MenuTitle class="mt-4">Windows</MenuTitle>
    <MenuItem
      v-for="{ id, name, icon } in EVERY_PAGES"
      :key="id"
      :class="{ 'menu-active': id === navigation.currentPage }"
    >
      <button
        type="button"
        :class="{ active: id === navigation.currentPage }"
        @click="navigation.navigate(id)"
      >
        <component :is="icon" class="h-4 w-4" />
        {{ name }}
      </button>
    </MenuItem>
  </Menu>
</template>
