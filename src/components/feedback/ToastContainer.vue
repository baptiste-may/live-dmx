<script setup lang="ts">
import { Alert, Button, Toast } from "daisy-ui-kit";
import { CircleCheckIcon, CircleXIcon, InfoIcon, TriangleAlertIcon } from "lucide-vue-next";

import { ToastType, useToastStore } from "@/stores/toast.ts";

const toastStore = useToastStore();

const getIcon = (type: ToastType) => {
  switch (type) {
    case ToastType.SUCCESS:
      return CircleCheckIcon;
    case ToastType.WARNING:
      return TriangleAlertIcon;
    case ToastType.ERROR:
      return CircleXIcon;
    default:
      return InfoIcon;
  }
};

const truncateMessage = (message: string, length = 100) => {
  if (message.length <= length) return message;
  return message.substring(0, length) + "...";
};
</script>

<template>
  <Toast end bottom class="z-9999">
    <Alert v-for="toast in toastStore.toasts" :key="toast.id" :type="toast.type" role="alert">
      <component :is="getIcon(toast.type)" class="h-5 w-5" />
      <span :title="toast.message">{{ truncateMessage(toast.message) }}</span>
      <Button ghost xs circle @click="toastStore.removeToast(toast.id)"> ✕ </Button>
    </Alert>
  </Toast>
</template>
