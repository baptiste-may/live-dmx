<script setup lang="ts">
import ConfirmationModal from "@/components/modals/ConfirmationModal.vue";
import UnsavedChangesModal from "@/components/modals/UnsavedChangesModal.vue";
import { useNavigation } from "@/stores/navigation.ts";

import AppSidebar from "./AppSidebar.vue";

const navigation = useNavigation();
</script>

<template>
  <div class="flex h-full w-full overflow-hidden">
    <AppSidebar />
    <ConfirmationModal />
    <UnsavedChangesModal />

    <main class="bg-base-100 relative flex-1 overflow-hidden">
      <KeepAlive v-if="navigation.getCurrentPage()">
        <component :is="navigation.getCurrentPage()?.component" />
      </KeepAlive>
    </main>
  </div>
</template>
