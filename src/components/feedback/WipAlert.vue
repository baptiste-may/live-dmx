<script setup lang="ts">
import { Alert } from "daisy-ui-kit";
import { TriangleAlertIcon } from "lucide-vue-next";
</script>

<template>
  <Alert warning role="alert" class="absolute top-4 right-4 z-50 w-auto">
    <TriangleAlertIcon />
    <span>This app is still in development.</span>
  </Alert>
</template>
