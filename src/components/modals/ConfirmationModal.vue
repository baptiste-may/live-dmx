<script setup lang="ts">
import { Button, Modal, ModalAction, ModalBox } from "daisy-ui-kit";
import { CheckIcon, XIcon } from "lucide-vue-next";

import { useConfirmation } from "@/stores/confirmation.ts";

const confirmation = useConfirmation();
</script>

<template>
  <Modal v-if="confirmation.isModalOpen" v-model="confirmation.isModalOpen">
    <ModalBox>
      <h3 class="text-lg font-bold">Confirmation</h3>
      <p>{{ confirmation.reason }}</p>
      <ModalAction>
        <Button error @click="confirmation.isModalOpen = false">
          <XIcon class="h-4 w-4" />
          Cancel
        </Button>
        <Button
          success
          @click="
            confirmation.callback?.();
            confirmation.isModalOpen = false;
          "
        >
          <CheckIcon class="h-4 w-4" />
          Confirm
        </Button>
      </ModalAction>
    </ModalBox>
  </Modal>
</template>
