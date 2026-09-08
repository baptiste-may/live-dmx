<script setup lang="ts">
import { Badge, Button, Input } from "daisy-ui-kit";
import { CheckIcon, GlobeIcon, LogOutIcon, RotateCwIcon } from "lucide-vue-next";
import { ref } from "vue";

import { useFixtureBrowser } from "@/stores/fixture-browser";

const browser = useFixtureBrowser();
const username = ref("");
const password = ref("");
const isSubmitting = ref(false);
const isSyncing = ref(false);
const errorMessage = ref<string | null>(null);

async function handleConnect() {
  if (!username.value.trim() || !password.value) return;

  isSubmitting.value = true;
  errorMessage.value = null;
  try {
    const success = await browser.setCredentials(username.value.trim(), password.value);
    if (success) {
      username.value = "";
      password.value = "";
      try {
        await browser.syncCatalog();
      } catch {
        errorMessage.value = "Connected, but catalog sync failed. You can retry syncing below.";
      }
    } else {
      errorMessage.value = "Login failed. Please verify your username and password.";
    }
  } catch {
    errorMessage.value =
      "Failed to connect to GDTF-Share. Please check your network and credentials.";
  } finally {
    isSubmitting.value = false;
  }
}

async function handleSync() {
  isSyncing.value = true;
  errorMessage.value = null;
  try {
    await browser.syncCatalog();
  } catch {
    errorMessage.value = "Catalog sync failed. Please try again.";
  } finally {
    isSyncing.value = false;
  }
}

async function handleLogout() {
  errorMessage.value = null;
  try {
    await browser.logout();
  } catch {
    errorMessage.value = "Disconnect failed. Please try again.";
  }
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex items-center justify-between">
      <h4
        class="text-base-content/70 flex items-center gap-1.5 text-xs font-bold tracking-wider uppercase"
      >
        <GlobeIcon class="h-3.5 w-3.5" />
        GDTF-Share Library
      </h4>
      <Badge v-if="browser.isAuthenticated" success soft xs>Connected</Badge>
      <Badge v-else ghost xs>Not Connected</Badge>
    </div>

    <div class="bg-base-300/40 flex flex-col gap-3 rounded-lg p-3">
      <p class="text-base-content/70 text-xs">
        Connect your free GDTF-Share account to search and download manufacturer fixture profiles
        directly into your show.
      </p>

      <p v-if="errorMessage" class="text-error text-xs">
        {{ errorMessage }}
      </p>

      <!-- Authenticated State -->
      <template v-if="browser.isAuthenticated">
        <div class="flex flex-col gap-2">
          <Button primary sm :disabled="isSyncing || isSubmitting" @click="handleSync">
            <RotateCwIcon class="h-3.5 w-3.5" :class="[isSyncing ? 'animate-spin' : '']" />
            {{ isSyncing ? "Syncing Catalog..." : "Sync Online Catalog" }}
          </Button>
          <Button error outline sm :disabled="isSyncing || isSubmitting" @click="handleLogout">
            <LogOutIcon class="h-3.5 w-3.5" />
            Disconnect Account
          </Button>
        </div>
      </template>

      <!-- Unauthenticated State -->
      <form v-else class="flex flex-col gap-2.5" @submit.prevent="handleConnect">
        <div>
          <label
            for="gdtf-username"
            class="text-base-content/70 block text-xs font-semibold uppercase"
          >
            Username / Email
          </label>
          <Input
            id="gdtf-username"
            v-model="username"
            type="text"
            placeholder="GDTF-Share username"
            sm
            class="mt-1 w-full"
            required
          />
        </div>

        <div>
          <label
            for="gdtf-password"
            class="text-base-content/70 block text-xs font-semibold uppercase"
          >
            Password
          </label>
          <Input
            id="gdtf-password"
            v-model="password"
            type="password"
            placeholder="Password"
            sm
            class="mt-1 w-full"
            required
          />
        </div>

        <p class="text-base-content/60 text-xs">
          Don't have an account? Create one for free at
          <a
            href="https://gdtf-share.com/signup"
            target="_blank"
            rel="noopener noreferrer"
            class="text-primary underline"
          >
            gdtf-share.com
          </a>
        </p>

        <Button
          type="submit"
          primary
          sm
          :disabled="isSubmitting || !username.trim() || !password"
          class="mt-1"
        >
          <CheckIcon class="h-3.5 w-3.5" />
          {{ isSubmitting ? "Connecting..." : "Connect & Sync" }}
        </Button>
      </form>
    </div>
  </div>
</template>
