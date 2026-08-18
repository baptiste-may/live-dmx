<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button, Card, CardBody } from "daisy-ui-kit";
import { PlusIcon, UploadIcon, XIcon } from "lucide-vue-next";

import { useApp } from "@/stores/app";

import RecentShowsList from "./RecentShowsList.vue";

const app = useApp();
const appWindow = getCurrentWindow();
</script>

<template>
  <div class="flex h-full w-full flex-col items-center justify-center gap-4">
    <h1 class="text-center text-7xl font-bold tracking-tight">Live DMX</h1>

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

        <RecentShowsList />

        <Button error class="mt-4" @click="appWindow.close()">
          <XIcon />
          Exit
        </Button>
      </CardBody>
    </Card>
  </div>
</template>
