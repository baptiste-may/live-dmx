<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    category?: string | string[];
    size?: number | string;
    color?: string;
    strokeWidth?: number | string;
  }>(),
  {
    category: "",
    size: 24,
    color: "currentColor",
    strokeWidth: 2,
  },
);

// Module-level static resources (evaluated ONLY ONCE on module load)
const svgModules = import.meta.glob("@/assets/icons/fixtures/*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

// Pre-extract raw SVG content into a clean lookup map ONCE at startup
const rawSvgCache = new Map<string, string>();
for (const [path, content] of Object.entries(svgModules)) {
  const fileName = path.split("/").pop()?.replace(".svg", "") || "";
  if (fileName) {
    rawSvgCache.set(fileName, content);
  }
}

const categoryKeyMap: Record<string, string> = {
  par: "par",
  wash: "wash",
  spot: "spot",
  "moving head": "moving-head",
  scanner: "scanner",
  "barrel scanner": "scanner",
  "pixel bar": "pixel-bar",
  strobe: "strobe",
  blinder: "blinder",
  laser: "laser",
  smoke: "smoke",
  hazer: "smoke",
  matrix: "matrix",
  dimmer: "dimmer",
  effect: "effect",
  flower: "effect",
  fan: "fan",
  stand: "stand",
  structure: "stand",
};

const priorityKeys = [
  "moving head",
  "scanner",
  "barrel scanner",
  "laser",
  "strobe",
  "blinder",
  "pixel bar",
  "matrix",
  "smoke",
  "hazer",
  "par",
  "spot",
  "wash",
  "flower",
  "effect",
  "dimmer",
  "fan",
  "stand",
  "structure",
];

// High-performance memoization cache for processed SVG strings
const processedSvgCache = new Map<string, string>();

function escapeAttr(val: unknown): string {
  return String(val)
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function getProcessedSvg(
  key: string,
  size: number | string,
  color: string,
  strokeWidth: number | string,
): string {
  const cacheKey = `${key}_${size}_${color}_${strokeWidth}`;
  const cached = processedSvgCache.get(cacheKey);
  if (cached !== undefined) {
    return cached;
  }

  let raw = rawSvgCache.get(key) || rawSvgCache.get("generic") || "";
  if (!raw) return "";

  raw = raw.replace(/<svg\s+([^>]*)/, (_, attrs) => {
    let newAttrs = attrs
      .replace(/\bwidth="[^"]*"/g, "")
      .replace(/\bheight="[^"]*"/g, "")
      .replace(/\bstroke-width="[^"]*"/g, "")
      .replace(/\bstroke="[^"]*"/g, "");

    newAttrs += ` width="${escapeAttr(size)}" height="${escapeAttr(size)}" stroke="${escapeAttr(color)}" stroke-width="${escapeAttr(strokeWidth)}"`;
    return `<svg ${newAttrs.trim()}`;
  });

  processedSvgCache.set(cacheKey, raw);
  return raw;
}

const iconKey = computed(() => {
  const categoriesList = Array.isArray(props.category)
    ? props.category
    : props.category
      ? [props.category]
      : [];

  const normalized = categoriesList.map((c) => c.trim().toLowerCase()).filter((c) => c.length > 0);

  for (const key of priorityKeys) {
    if (normalized.some((cat) => cat.includes(key))) {
      return categoryKeyMap[key] || "generic";
    }
  }

  if (normalized.some((cat) => cat.includes("color changer") || cat.includes("wash"))) {
    return "wash";
  }

  return "generic";
});

const svgContent = computed(() => {
  return getProcessedSvg(iconKey.value, props.size, props.color, props.strokeWidth);
});
</script>

<template>
  <!-- eslint-disable-next-line vue/no-v-html -->
  <span class="inline-flex shrink-0 items-center justify-center" v-html="svgContent" />
</template>
