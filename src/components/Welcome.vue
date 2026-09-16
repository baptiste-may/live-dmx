<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Badge, Button, Card, CardBody } from "daisy-ui-kit";
import {
  ArrowUpIcon,
  FolderInputIcon,
  LoaderIcon,
  PlusIcon,
  RefreshCcwIcon,
  UploadIcon,
  XIcon,
} from "lucide-vue-next";
import { computed, onMounted } from "vue";

import { useApp } from "@/stores/app";
import { useUpdates } from "@/stores/updates";

import RecentShowsList from "./RecentShowsList.vue";

const app = useApp();
const appWindow = getCurrentWindow();
const updates = useUpdates();

onMounted(() => {
  updates.init();
});

const updateLabel = computed(() => {
  switch (updates.status) {
    case "available":
      return `Install update v${updates.info?.version ?? ""}`;
    case "checking":
      return "Checking for updates...";
    case "downloading":
      return `Downloading... ${Math.round(updates.progress * 100)}%`;
    case "installing":
      return "Installing update...";
    case "error":
      return "Retry update check";
    default:
      return "Check for updates";
  }
});

const updateIcon = computed(() => {
  switch (updates.status) {
    case "checking":
    case "downloading":
    case "installing":
      return LoaderIcon;
    case "error":
      return RefreshCcwIcon;
    default:
      return ArrowUpIcon;
  }
});

const showUpdateButton = computed(
  () => updates.hasUpdate || updates.isBusy || updates.status === "error",
);

function handleUpdateClick() {
  updates.requestUpdate();
}
</script>

<template>
  <div class="flex h-full w-full flex-col items-center justify-center gap-6">
    <div class="flex flex-col items-center gap-3">
      <h1 class="text-center text-7xl font-bold tracking-tight">Live DMX</h1>
      <div class="flex items-center gap-2.5">
        <Badge neutral lg class="font-semibold tabular-nums">
          v{{ updates.appVersion || "…" }}
        </Badge>
        <Button
          v-if="showUpdateButton"
          sm
          :warning="updates.hasUpdate"
          :neutral="updates.status === 'error'"
          :disabled="updates.isBusy"
          :title="updateLabel"
          :aria-label="updateLabel"
          @click="handleUpdateClick"
        >
          <component
            :is="updateIcon"
            class="h-4 w-4"
            :class="[updates.isBusy ? 'animate-spin' : '']"
          />
          {{ updateLabel }}
        </Button>
      </div>
    </div>

    <Card class="bg-base-200 shadow-xl">
      <CardBody>
        <Button outline primary @click="app.createShow">
          <PlusIcon />
          New show
        </Button>
        <Button primary @click="app.askLoadShow">
          <UploadIcon />
          Load show
        </Button>
        <Button outline primary @click="app.askImportMvr">
          <FolderInputIcon />
          Import from MVR
        </Button>

        <RecentShowsList />

        <Button error class="mt-4" @click="appWindow.close()">
          <XIcon />
          Exit
        </Button>
      </CardBody>
    </Card>
  </div>
</template>
