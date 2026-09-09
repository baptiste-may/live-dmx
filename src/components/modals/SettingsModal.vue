<script setup lang="ts">
import { Button, Divider, Join, Modal, ModalAction, ModalBox, Range, Toggle } from "daisy-ui-kit";
import { MonitorIcon, MoonIcon, SettingsIcon, SunIcon, XIcon, ZoomInIcon } from "lucide-vue-next";

import { useSettings } from "@/stores/settings.ts";

import SettingsGdtf from "./settings/SettingsGdtf.vue";

const settings = useSettings();
</script>

<template>
  <Modal v-if="settings.isModalOpen" v-model="settings.isModalOpen">
    <ModalBox class="max-w-lg">
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

          <div class="bg-base-300/40 flex flex-col gap-4 rounded-lg p-3">
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

            <div class="flex flex-col gap-1.5">
              <div class="flex items-center justify-between text-sm font-medium">
                <span class="flex items-center gap-1.5">
                  <ZoomInIcon class="text-base-content/70 h-4 w-4" />
                  Scale
                </span>
                <div class="flex items-center gap-2">
                  <span class="text-xs font-semibold tabular-nums">{{ settings.scale }}%</span>
                  <Button
                    v-if="settings.scale !== 100"
                    ghost
                    xs
                    aria-label="Reset scale"
                    @click="settings.setScale(100)"
                  >
                    Reset
                  </Button>
                </div>
              </div>
              <Range
                primary
                sm
                class="!w-full"
                :min="50"
                :max="150"
                :step="5"
                :model-value="settings.scale"
                aria-label="Scale"
                @update:model-value="settings.setScale"
              />
              <div
                class="text-base-content/40 flex w-full justify-between px-2.5 text-xs select-none"
              >
                <span>|</span>
                <span>|</span>
                <span>|</span>
                <span>|</span>
                <span>|</span>
              </div>
              <div
                class="text-base-content/60 flex w-full justify-between px-2.5 text-[10px] select-none"
              >
                <button
                  type="button"
                  class="hover:text-base-content cursor-pointer"
                  @click="settings.setScale(50)"
                >
                  50%
                </button>
                <button
                  type="button"
                  class="hover:text-base-content cursor-pointer"
                  @click="settings.setScale(75)"
                >
                  75%
                </button>
                <button
                  type="button"
                  class="hover:text-base-content cursor-pointer font-bold"
                  @click="settings.setScale(100)"
                >
                  100%
                </button>
                <button
                  type="button"
                  class="hover:text-base-content cursor-pointer"
                  @click="settings.setScale(125)"
                >
                  125%
                </button>
                <button
                  type="button"
                  class="hover:text-base-content cursor-pointer"
                  @click="settings.setScale(150)"
                >
                  150%
                </button>
              </div>
            </div>
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

        <Divider class="my-0" />

        <!-- GDTF-Share Section -->
        <SettingsGdtf />
      </div>

      <ModalAction>
        <Button primary @click="settings.closeModal">Close</Button>
      </ModalAction>
    </ModalBox>
  </Modal>
</template>
