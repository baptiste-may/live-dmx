<script setup lang="ts">
import App from "@/App.vue";
import ToastContainer from "@/components/feedback/ToastContainer.vue";
import WipAlert from "@/components/feedback/WipAlert.vue";
</script>

<template>
  <div class="relative h-screen w-screen overflow-hidden">
    <WipAlert />
    <ToastContainer />
    <App />
  </div>
</template>
