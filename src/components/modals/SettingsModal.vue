<script setup lang="ts">
import { Button, Modal, ModalAction, ModalBox } from "daisy-ui-kit";
import { SettingsIcon, XIcon } from "lucide-vue-next";

import { useSettings } from "@/stores/settings.ts";

import SettingsAbout from "./settings/SettingsAbout.vue";
import SettingsGdtf from "./settings/SettingsGdtf.vue";
import SettingsGlobal from "./settings/SettingsGlobal.vue";
import SettingsLive from "./settings/SettingsLive.vue";

const settings = useSettings();
</script>

<template>
  <Modal v-if="settings.isModalOpen" v-model="settings.isModalOpen">
    <ModalBox class="max-w-3xl">
      <div class="flex items-center justify-between">
        <h3 class="flex items-center gap-2 text-lg font-bold">
          <SettingsIcon class="h-5 w-5" />
          Settings
        </h3>
        <Button ghost sm square aria-label="Close settings" @click="settings.closeModal">
          <XIcon class="h-4 w-4" />
        </Button>
      </div>

      <div class="mt-4 grid grid-cols-1 gap-5 md:grid-cols-2">
        <SettingsGlobal />
        <SettingsLive />
        <SettingsGdtf />
        <SettingsAbout />
      </div>

      <ModalAction>
        <Button primary @click="settings.closeModal">Close</Button>
      </ModalAction>
    </ModalBox>
  </Modal>
</template>
