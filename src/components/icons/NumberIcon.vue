<script setup lang="ts">
import { Badge } from "daisy-ui-kit";

withDefaults(
  defineProps<{
    number?: number | string;
  }>(),
  {
    number: "",
  },
);
</script>

<template>
  <Badge neutral class="flex aspect-square items-center justify-center p-0 font-bold">
    {{ number }}
  </Badge>
</template>
