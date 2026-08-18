<script setup lang="ts">
import { Button, Modal, ModalAction, ModalBox } from "daisy-ui-kit";
import { SaveIcon } from "lucide-vue-next";

import { useShowChanges } from "@/stores/show/show-changes.ts";
import { useShowIdentity } from "@/stores/show/show-identity.ts";

const changes = useShowChanges();
const identity = useShowIdentity();
</script>

<template>
  <Modal v-if="changes.isUnsavedChangesModalOpen" v-model="changes.isUnsavedChangesModalOpen">
    <ModalBox class="border-warning border shadow-2xl">
      <h3 class="flex items-center gap-2 text-lg font-bold">
        <SaveIcon class="h-5 w-5" />
        Unsaved Changes
      </h3>
      <p class="py-4">
        You have unsaved changes in <span class="font-semibold">{{ identity.displayName }}</span
        >. What would you like to do?
      </p>
      <ModalAction class="flex items-center justify-between">
        <Button ghost @click="changes.handleCancel">Cancel</Button>
        <div class="flex gap-2">
          <Button outline error @click="changes.discardAndExecute"> Discard Changes </Button>
          <Button primary @click="changes.saveAndExecute">Save & Continue</Button>
        </div>
      </ModalAction>
    </ModalBox>
  </Modal>
</template>
