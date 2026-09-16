<script setup lang="ts">
import { useShowConfiguration } from "@/stores/show/show-configuration.ts";

const configure = useShowConfiguration();
</script>

<template>
  <div
    class="pointer-events-none absolute inset-y-0 right-0 left-0 border border-current/55 text-[0.625rem] font-semibold text-current/55"
    role="presentation"
    aria-hidden="true"
  >
    <div
      class="absolute inset-0"
      :style="{
        backgroundImage: [
          'linear-gradient(to right, color-mix(in oklab, currentColor 18%, transparent) 1px, transparent 1px)',
          'linear-gradient(to bottom, color-mix(in oklab, currentColor 18%, transparent) 1px, transparent 1px)',
        ].join(', '),
        backgroundSize: '25% 100%, 100% 25%',
      }"
    />
    <span v-if="configure.hasPan" class="absolute top-1/2 right-2 translate-y-[200%] leading-none"
      >{{ configure.groupPanBounds[1] }}°</span
    >
    <span v-if="configure.hasPan" class="absolute top-1/2 left-2 translate-y-[200%] leading-none"
      >{{ configure.groupPanBounds[0] }}°</span
    >
    <span v-if="configure.hasTilt" class="absolute top-2 right-1/2 translate-x-[200%] leading-none"
      >{{ configure.groupTiltBounds[1] }}°</span
    >
    <span
      v-if="configure.hasTilt"
      class="absolute right-1/2 bottom-2 translate-x-[200%] leading-none"
      >{{ configure.groupTiltBounds[0] }}°</span
    >
  </div>
</template>
