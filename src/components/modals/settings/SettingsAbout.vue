<script setup lang="ts">
import { Badge, Button, Progress } from "daisy-ui-kit";
import { ArrowUpIcon, InfoIcon, LoaderIcon, RefreshCcwIcon } from "lucide-vue-next";

import { useUpdates } from "@/stores/updates";

const updates = useUpdates();
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h4
        class="text-base-content/70 flex items-center gap-1.5 text-xs font-bold tracking-wider uppercase"
      >
        <InfoIcon class="h-3.5 w-3.5" />
        About & Updates
      </h4>
      <Badge v-if="updates.isBusy" info soft xs>
        {{ updates.status === "checking" ? "Checking..." : "Updating..." }}
      </Badge>
      <Badge v-else-if="updates.hasUpdate" warning soft xs>Update available</Badge>
      <Badge v-else-if="updates.status === 'up-to-date'" success soft xs>Up to date</Badge>
    </div>

    <div class="bg-base-300/40 flex flex-col gap-3 rounded-lg p-3">
      <div class="flex items-center justify-between text-sm">
        <span class="text-base-content/70">App version</span>
        <span class="font-semibold tabular-nums">v{{ updates.appVersion || "…" }}</span>
      </div>
      <div class="flex items-center justify-between text-sm">
        <span class="text-base-content/70">Tauri version</span>
        <span class="font-semibold tabular-nums">{{ updates.tauriVersion || "…" }}</span>
      </div>

      <div v-if="updates.hasUpdate" class="text-sm font-medium">
        Update <span class="text-warning font-bold">v{{ updates.info?.version }}</span> is
        available.
      </div>
      <p v-if="updates.status === 'error'" class="text-error text-xs">{{ updates.error }}</p>
      <p v-if="updates.status === 'up-to-date'" class="text-base-content/60 text-xs">
        Your app is up to date. Updates are checked against GitHub releases.
      </p>

      <Progress
        v-if="updates.status === 'downloading'"
        primary
        class="w-full"
        :value="updates.downloaded"
        :max="updates.contentLength || 1"
      />

      <Button
        primary
        sm
        :disabled="updates.isBusy"
        :data-testid="updates.hasUpdate ? 'install-update' : 'check-updates'"
        @click="updates.requestUpdate"
      >
        <LoaderIcon v-if="updates.isBusy" class="h-3.5 w-3.5 animate-spin" />
        <ArrowUpIcon v-else-if="updates.hasUpdate" class="h-3.5 w-3.5" />
        <RefreshCcwIcon v-else class="h-3.5 w-3.5" />
        {{
          updates.status === "checking"
            ? "Checking..."
            : updates.status === "downloading"
              ? `Downloading... ${Math.round(updates.progress * 100)}%`
              : updates.status === "installing"
                ? "Installing..."
                : updates.hasUpdate
                  ? `Install update v${updates.info?.version}`
                  : "Check for updates"
        }}
      </Button>
    </div>
  </div>
</template>
