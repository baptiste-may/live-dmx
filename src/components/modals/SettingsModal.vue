<script setup lang="ts">
import { Button, Divider, Join, Modal, ModalAction, ModalBox, Toggle } from "daisy-ui-kit";
import { MonitorIcon, MoonIcon, SettingsIcon, SunIcon, XIcon } from "lucide-vue-next";

import { useSettings } from "@/stores/settings.ts";

const settings = useSettings();
</script>

<template>
  <Modal v-if="settings.isModalOpen" v-model="settings.isModalOpen">
    <ModalBox class="max-w-md">
      <div class="flex items-center justify-between">
        <h3 class="flex items-center gap-2 text-lg font-bold">
          <SettingsIcon class="h-5 w-5" />
          Settings
        </h3>
        <Button ghost sm square aria-label="Close settings" @click="settings.closeModal">
          <XIcon class="h-4 w-4" />
        </Button>
      </div>

      <div class="mt-4 flex flex-col gap-5">
        <!-- Global Section -->
        <div class="flex flex-col gap-2">
          <h4 class="text-base-content/70 text-xs font-bold tracking-wider uppercase">Global</h4>

          <div class="bg-base-300/40 flex flex-col gap-2 rounded-lg p-3">
            <Join class="w-full">
              <Button
                join
                sm
                class="flex-1"
                :primary="settings.theme === 'dark'"
                :outline="settings.theme !== 'dark'"
                aria-label="Dark theme"
                @click="settings.setTheme('dark')"
              >
                <MoonIcon class="h-4 w-4" />
                Dark
              </Button>
              <Button
                join
                sm
                class="flex-1"
                :primary="settings.theme === 'light'"
                :outline="settings.theme !== 'light'"
                aria-label="Light theme"
                @click="settings.setTheme('light')"
              >
                <SunIcon class="h-4 w-4" />
                Light
              </Button>
              <Button
                join
                sm
                class="flex-1"
                :primary="settings.theme === 'system'"
                :outline="settings.theme !== 'system'"
                aria-label="System theme"
                @click="settings.setTheme('system')"
              >
                <MonitorIcon class="h-4 w-4" />
                System
              </Button>
            </Join>
          </div>
        </div>

        <Divider class="my-0" />

        <!-- Live Section -->
        <div class="flex flex-col gap-2">
          <h4 class="text-base-content/70 text-xs font-bold tracking-wider uppercase">Live</h4>

          <div class="bg-base-300/40 flex flex-col gap-4 rounded-lg p-3">
            <!-- Toggle: Show button labels -->
            <label class="flex cursor-pointer items-center justify-between gap-4">
              <span class="text-sm font-medium">Show button labels</span>
              <Toggle
                primary
                :model-value="settings.live.showButtonLabels"
                aria-label="Show button labels"
                @update:model-value="settings.setShowButtonLabels"
              />
            </label>

            <!-- Toggle: Show section titles -->
            <label class="flex cursor-pointer items-center justify-between gap-4">
              <span class="text-sm font-medium">Show section titles</span>
              <Toggle
                primary
                :model-value="settings.live.showSectionTitles"
                aria-label="Show section titles"
                @update:model-value="settings.setShowSectionTitles"
              />
            </label>
          </div>
        </div>
      </div>

      <ModalAction>
        <Button primary @click="settings.closeModal">Close</Button>
      </ModalAction>
    </ModalBox>
  </Modal>
</template>
