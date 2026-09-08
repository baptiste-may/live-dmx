<script setup lang="ts">
import { computed, ref } from "vue";

const props = withDefaults(
  defineProps<{
    modelValue?: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    readonly?: boolean;
    label?: string;
    ariaLabel?: string;
    unit?: string;
    showValue?: boolean;
    color?:
      | "primary"
      | "secondary"
      | "accent"
      | "info"
      | "success"
      | "warning"
      | "error"
      | "neutral";
    size?: "sm" | "md" | "lg" | "xl" | number | string;
    defaultValue?: number;
  }>(),
  {
    modelValue: 0,
    min: 0,
    max: 255,
    step: 1,
    disabled: false,
    readonly: false,
    label: undefined,
    ariaLabel: undefined,
    unit: "",
    showValue: true,
    color: "primary",
    size: "md",
    defaultValue: undefined,
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", value: number): void;
  (e: "change", value: number): void;
}>();

const isDragging = ref(false);
const isFocused = ref(false);

let startY = 0;
let startValue = 0;

const ratio = computed(() => {
  const range = props.max - props.min;
  if (range <= 0) return 0;
  const clamped = Math.min(Math.max(props.modelValue, props.min), props.max);
  return (clamped - props.min) / range;
});

const sizeInPx = computed(() => {
  if (typeof props.size === "number") return props.size;
  switch (props.size) {
    case "sm":
      return 36;
    case "lg":
      return 64;
    case "xl":
      return 80;
    case "md":
    default:
      if (typeof props.size === "string" && !isNaN(parseInt(props.size))) {
        return parseInt(props.size);
      }
      return 48;
  }
});

const textSizeClass = computed(() => {
  if (sizeInPx.value <= 40) {
    if (props.unit && String(props.modelValue).length >= 3) {
      return "text-[9px]";
    }
    return "text-[10px]";
  }
  if (sizeInPx.value <= 52) return "text-xs";
  return "text-sm";
});

const colorClassMap: Record<string, { stroke: string; bg: string; text: string }> = {
  primary: { stroke: "stroke-primary", bg: "bg-primary", text: "text-primary" },
  secondary: {
    stroke: "stroke-secondary",
    bg: "bg-secondary",
    text: "text-secondary",
  },
  accent: { stroke: "stroke-accent", bg: "bg-accent", text: "text-accent" },
  info: { stroke: "stroke-info", bg: "bg-info", text: "text-info" },
  success: { stroke: "stroke-success", bg: "bg-success", text: "text-success" },
  warning: { stroke: "stroke-warning", bg: "bg-warning", text: "text-warning" },
  error: { stroke: "stroke-error", bg: "bg-error", text: "text-error" },
  neutral: { stroke: "stroke-neutral", bg: "bg-neutral", text: "text-neutral" },
};

const currentColorClasses = computed(() => {
  return colorClassMap[props.color] || colorClassMap.primary;
});

// SVG 270-degree arc calculations:
// Circle r=38 -> Circumference = 2 * PI * 38 = 238.761
// 270 degrees arc length = 0.75 * 238.761 = 179.07
const arcLength = 179.1;
const dashArray = `${arcLength} 238.8`;
const strokeDashoffset = computed(() => {
  return arcLength * (1 - ratio.value);
});

function clampAndStep(val: number, step = props.step): number {
  let rounded = Math.round((val - props.min) / step) * step + props.min;
  const precision = (step.toString().split(".")[1] || "").length;
  rounded = Number(rounded.toFixed(precision));
  return Math.min(Math.max(rounded, props.min), props.max);
}

function updateValue(newValue: number, step = props.step) {
  if (props.disabled || props.readonly) return;
  const clamped = clampAndStep(newValue, step);
  if (clamped !== props.modelValue) {
    emit("update:modelValue", clamped);
    emit("change", clamped);
  }
}

function onPointerDown(e: PointerEvent) {
  if (props.disabled || props.readonly) return;
  isDragging.value = true;
  startY = e.clientY;
  startValue = props.modelValue;

  const target = e.currentTarget as HTMLElement;
  if (target && typeof target.setPointerCapture === "function") {
    target.setPointerCapture(e.pointerId);
  }
}

function onPointerMove(e: PointerEvent) {
  if (!isDragging.value || props.disabled || props.readonly) return;

  const deltaY = startY - e.clientY;
  const range = props.max - props.min;
  const dragSens = 150;
  const scale = e.shiftKey ? 0.1 : 1;
  const effectiveStep = props.step * scale;

  const deltaValue = (deltaY / dragSens) * range * scale;
  updateValue(startValue + deltaValue, effectiveStep);
}

function onPointerUp(e: PointerEvent) {
  if (isDragging.value) {
    isDragging.value = false;
    const target = e.currentTarget as HTMLElement;
    if (
      target &&
      typeof target.hasPointerCapture === "function" &&
      target.hasPointerCapture(e.pointerId)
    ) {
      target.releasePointerCapture(e.pointerId);
    }
  }
}

function onWheel(e: WheelEvent) {
  if (props.disabled || props.readonly) return;
  e.preventDefault();

  const direction = e.deltaY < 0 ? 1 : -1;
  let multiplier = props.step;
  let effectiveStep = props.step;
  if (e.shiftKey) {
    multiplier *= 0.1;
    effectiveStep *= 0.1;
  } else if (e.ctrlKey || e.metaKey) {
    multiplier *= 10;
    effectiveStep *= 10;
  }

  updateValue(props.modelValue + direction * multiplier, effectiveStep);
}

function onKeyDown(e: KeyboardEvent) {
  if (props.disabled || props.readonly) return;

  let delta = 0;
  const effectiveStep = e.shiftKey ? props.step * 0.1 : props.step;

  switch (e.key) {
    case "ArrowUp":
    case "ArrowRight":
      delta = effectiveStep;
      break;
    case "ArrowDown":
    case "ArrowLeft":
      delta = -effectiveStep;
      break;
    case "PageUp":
      delta = props.step * 10;
      break;
    case "PageDown":
      delta = -props.step * 10;
      break;
    case "Home":
      updateValue(props.min);
      e.preventDefault();
      return;
    case "End":
      updateValue(props.max);
      e.preventDefault();
      return;
    default:
      return;
  }

  e.preventDefault();
  updateValue(props.modelValue + delta, effectiveStep);
}

function onDblClick() {
  if (props.disabled || props.readonly) return;
  const resetVal = props.defaultValue !== undefined ? props.defaultValue : props.min;
  updateValue(resetVal);
}
</script>

<template>
  <div
    class="inline-flex flex-col items-center gap-1 select-none"
    :class="[disabled ? 'cursor-not-allowed opacity-50' : 'cursor-grab active:cursor-grabbing']"
  >
    <span v-if="label" class="text-base-content/80 text-xs font-semibold">
      {{ label }}
    </span>

    <div
      tabindex="0"
      role="slider"
      :aria-label="ariaLabel || label || 'Knob control'"
      :aria-valuenow="modelValue"
      :aria-valuemin="min"
      :aria-valuemax="max"
      :aria-disabled="disabled"
      :aria-valuetext="`${modelValue}${unit}`"
      class="focus-visible:ring-primary relative flex touch-none items-center justify-center rounded-full focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2"
      :style="{ width: `${sizeInPx}px`, height: `${sizeInPx}px` }"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @wheel="onWheel"
      @keydown="onKeyDown"
      @dblclick="onDblClick"
      @focus="isFocused = true"
      @blur="isFocused = false"
    >
      <!-- Background & Active 270-degree Arc Gauge -->
      <svg class="absolute inset-0 h-full w-full" viewBox="0 0 100 100">
        <!-- Track Background Arc -->
        <circle
          cx="50"
          cy="50"
          r="38"
          fill="none"
          stroke-width="7"
          stroke-linecap="round"
          class="stroke-base-content/15"
          :stroke-dasharray="dashArray"
          :stroke-dashoffset="0"
          transform="rotate(135 50 50)"
        />
        <!-- Active Progress Arc -->
        <circle
          cx="50"
          cy="50"
          r="38"
          fill="none"
          stroke-width="7"
          stroke-linecap="round"
          :class="[currentColorClasses.stroke]"
          :stroke-dasharray="dashArray"
          :stroke-dashoffset="strokeDashoffset"
          transform="rotate(135 50 50)"
          class="transition-[stroke-dashoffset] duration-75 ease-out"
        />
      </svg>

      <!-- Centered Value Display -->
      <span
        v-if="showValue"
        class="text-base-content pointer-events-none z-10 text-center leading-none font-bold tracking-tight"
        :class="textSizeClass"
      >
        {{ modelValue }}{{ unit }}
      </span>
    </div>
  </div>
</template>
