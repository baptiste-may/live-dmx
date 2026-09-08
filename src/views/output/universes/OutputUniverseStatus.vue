<script setup lang="ts">
import { ActivityIcon, AlertCircleIcon, AlertTriangleIcon, CircleSlashIcon } from "lucide-vue-next";
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    universe: number;
    enabled: boolean;
    output: string;
    physicalStatus?: "connected" | "disconnected" | "busy";
    isMissingPhysical?: boolean;
  }>(),
  {
    physicalStatus: undefined,
    isMissingPhysical: false,
  },
);

export type OutputStatusType = "sending" | "error" | "idle" | "disabled";

const status = computed<OutputStatusType>(() => {
  if (!props.enabled) {
    return "disabled";
  }
  if (props.output === "none") {
    return "idle";
  }
  if (props.isMissingPhysical || props.physicalStatus === "disconnected") {
    return "error";
  }
  return "sending";
});

const statusDetails = computed(() => {
  switch (status.value) {
    case "disabled":
      return {
        label: "Disabled",
        tooltip: `Universe ${props.universe}: Output disabled`,
        icon: CircleSlashIcon,
        classes: "bg-base-200 text-base-content/40 border-base-300",
      };
    case "error":
      return {
        label: "Error",
        tooltip: props.isMissingPhysical
          ? `Universe ${props.universe}: Interface not found`
          : `Universe ${props.universe}: Interface disconnected`,
        icon: AlertTriangleIcon,
        classes: "bg-error/15 text-error border-error/30",
      };
    case "idle":
      return {
        label: "Idle",
        tooltip: `Universe ${props.universe}: No route assigned`,
        icon: AlertCircleIcon,
        classes: "bg-warning/15 text-warning border-warning/30",
      };
    case "sending":
    default:
      return {
        label: "Sending data",
        tooltip: `Universe ${props.universe}: Sending data`,
        icon: ActivityIcon,
        classes: "bg-success/15 text-success border-success/30",
      };
  }
});
</script>

<template>
  <div
    class="tooltip tooltip-right flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border transition-colors"
    :class="statusDetails.classes"
    :data-tip="statusDetails.tooltip"
    :aria-label="statusDetails.tooltip"
    role="status"
  >
    <component
      :is="statusDetails.icon"
      class="h-5 w-5"
      :class="[status === 'sending' ? 'animate-pulse' : '']"
    />
  </div>
</template>
