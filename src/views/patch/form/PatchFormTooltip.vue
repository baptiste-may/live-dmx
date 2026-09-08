<script setup lang="ts">
import { Badge, Card, CardBody } from "daisy-ui-kit";
import {
  BoxIcon,
  CableIcon,
  FocusIcon,
  SlidersIcon,
  SunIcon,
  WeightIcon,
  ZapIcon,
} from "lucide-vue-next";
import { computed, inject, onUnmounted, type Ref, ref, watch } from "vue";

import { FixtureIcon } from "@/components/icons";
import { failedPreviewKeys, formatChannelName } from "@/libs/fixture.ts";
import { useFixtureBrowser } from "@/stores/fixture-browser.ts";
import type { FixtureType, NamedChannel } from "@/types/fixtures.ts";

const props = defineProps<{
  fixture: FixtureType;
  mode: number;
}>();

const fixtureBrowser = useFixtureBrowser();
const isTooltipOpen = inject<Ref<boolean>>("isTooltipOpen", ref(true));
const isLoadingPreview = ref(false);

const manufacturerName = computed(() => {
  const m = (fixtureBrowser.manufacturers ?? {})[props.fixture.manufacturerKey];
  return m?.name || props.fixture.manufacturerKey;
});
const modeData = computed(() => props.fixture.modes?.[props.mode]);
const channels = computed(() => modeData.value?.channels ?? []);

const dmxChannelCount = computed(() => {
  return modeData.value?.dmxfootprint ?? channels.value.length;
});

const isPlaceholder = computed(() => {
  const chs = channels.value;
  return (
    chs.length === 0 ||
    chs.every((c, i) => {
      const name = typeof c === "string" ? c : (c as NamedChannel)?.name || "";
      return !name || name === `Channel ${i + 1}`;
    })
  );
});

const getPreviewKey = (fixture: FixtureType, modeName: string) => {
  const mfg = fixture.manufacturerKey || fixture.manufacturer || "";
  const name = fixture.fixtureKey || fixture.name;
  return `${mfg}:${name}:${fixture.rid || ""}:${modeName.trim().toLowerCase()}`;
};

const loadPreview = async () => {
  if (!props.fixture.rid || !isPlaceholder.value || isLoadingPreview.value) return;
  const mName = modeData.value?.name;
  if (!mName) return;

  const key = getPreviewKey(props.fixture, mName);
  if (failedPreviewKeys.has(key)) return;

  isLoadingPreview.value = true;
  try {
    await fixtureBrowser.fetchMode(props.fixture, mName, { persist: false });
  } catch {
    failedPreviewKeys.add(key);
  } finally {
    isLoadingPreview.value = false;
  }
};

let timer: ReturnType<typeof setTimeout> | null = null;

watch(
  [() => isTooltipOpen.value, () => props.fixture, () => props.mode],
  ([open]) => {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    const mName = modeData.value?.name;
    const key = mName ? getPreviewKey(props.fixture, mName) : "";
    if (open && props.fixture.rid && isPlaceholder.value && (!key || !failedPreviewKeys.has(key))) {
      timer = setTimeout(() => {
        void loadPreview();
      }, 250);
    }
  },
  { immediate: true },
);

onUnmounted(() => {
  if (timer) {
    clearTimeout(timer);
  }
});

const physical = computed(() => modeData.value?.physical ?? props.fixture.physical);

const power = computed(() => {
  return physical.value?.power ? `${physical.value.power}W` : null;
});

const beamAngle = computed(() => {
  const lens = physical.value?.lens;
  if (!lens?.degreesMinMax) return null;
  const [min, max] = lens.degreesMinMax;
  return min === max ? `${min}°` : `${min}° – ${max}°`;
});

const weight = computed(() => {
  return physical.value?.weight ? `${physical.value.weight}kg` : null;
});

const dmxConnector = computed(() => {
  return physical.value?.DMXconnector ?? null;
});

const bulbType = computed(() => {
  return physical.value?.bulb?.type ?? null;
});

const dimensions = computed(() => {
  const d = physical.value?.dimensions;
  if (!d || d.length !== 3) return null;
  return `${d[0]} × ${d[1]} × ${d[2]} mm`;
});
</script>

<template>
  <Card
    class="fixture-tooltip border-base-content/15 bg-base-200 w-[520px] max-w-[540px] rounded-2xl text-left shadow-2xl"
  >
    <CardBody class="flex min-h-0 flex-col gap-3 p-4">
      <!-- Top row: Manufacturer on left, Capabilities / Badges on top right -->
      <div class="flex items-center justify-between gap-2">
        <Badge primary xs outline class="max-w-[200px] shrink-0 truncate font-medium">
          {{ manufacturerName }}
        </Badge>
        <div class="ml-auto flex flex-wrap items-center justify-end gap-1.5">
          <Badge v-if="fixture.isGeneric" info xs class="shrink-0 font-semibold"> Generic </Badge>
          <Badge v-else-if="fixture.isInstalled" success xs outline class="shrink-0 font-semibold">
            Installed
          </Badge>
          <Badge v-else-if="fixture.rid" warning xs outline class="shrink-0 font-semibold">
            GDTF-Share
          </Badge>
          <Badge v-if="fixture.uploader" neutral xs class="shrink-0 font-medium">
            {{ fixture.uploader }}
          </Badge>
          <Badge
            v-for="cat in fixture.categories"
            :key="cat"
            accent
            xs
            class="shrink-0 font-bold tracking-wide shadow-xs"
          >
            {{ cat }}
          </Badge>
        </div>
      </div>

      <!-- Title row: Fixture Icon & Name & Mode -->
      <div class="flex items-center gap-2.5">
        <FixtureIcon :category="fixture.categories" :size="24" class="text-primary shrink-0" />
        <div class="min-w-0 flex-1">
          <h3 class="text-base-content truncate text-base leading-tight font-bold">
            {{ fixture.name }}
          </h3>
          <span class="text-base-content/70 block truncate text-xs font-medium">
            Mode: {{ modeData?.name }}
          </span>
          <span v-if="fixture.revision" class="text-base-content/60 block truncate text-[11px]">
            Revision: {{ fixture.revision }}
          </span>
        </div>
      </div>

      <!-- Fixture description/comment if available -->
      <p
        v-if="fixture.description || fixture.comment"
        class="text-base-content/60 -mt-1 line-clamp-2 text-xs italic"
      >
        {{ fixture.description || fixture.comment }}
      </p>

      <!-- Physical Specs Summary (Clean 2-column dark box from Inspector design) -->
      <div
        v-if="power || beamAngle || bulbType || weight || dmxConnector || dimensions"
        class="bg-base-300/60 border-base-content/5 grid grid-cols-2 gap-2 rounded-xl border p-2.5 text-xs"
      >
        <div v-if="power" class="flex items-center gap-2" title="Power consumption">
          <ZapIcon class="text-warning h-4 w-4 shrink-0" />
          <div class="min-w-0">
            <span class="text-base-content/50 block text-[10px] font-semibold uppercase"
              >Power</span
            >
            <span class="text-base-content block truncate font-medium">{{ power }}</span>
          </div>
        </div>
        <div v-if="beamAngle" class="flex items-center gap-2" title="Beam angle">
          <FocusIcon class="text-info h-4 w-4 shrink-0" />
          <div class="min-w-0">
            <span class="text-base-content/50 block text-[10px] font-semibold uppercase">Beam</span>
            <span class="text-base-content block truncate font-medium">{{ beamAngle }}</span>
          </div>
        </div>
        <div v-if="bulbType" class="flex items-center gap-2" title="Lamp type">
          <SunIcon class="h-4 w-4 shrink-0 text-amber-400" />
          <div class="min-w-0">
            <span class="text-base-content/50 block text-[10px] font-semibold uppercase">Lamp</span>
            <span class="text-base-content block truncate font-medium">{{ bulbType }}</span>
          </div>
        </div>
        <div v-if="weight" class="flex items-center gap-2" title="Weight">
          <WeightIcon class="text-base-content/70 h-4 w-4 shrink-0" />
          <div class="min-w-0">
            <span class="text-base-content/50 block text-[10px] font-semibold uppercase"
              >Weight</span
            >
            <span class="text-base-content block truncate font-medium">{{ weight }}</span>
          </div>
        </div>
        <div v-if="dmxConnector" class="flex items-center gap-2" title="DMX connector">
          <CableIcon class="text-base-content/70 h-4 w-4 shrink-0" />
          <div class="min-w-0">
            <span class="text-base-content/50 block text-[10px] font-semibold uppercase">DMX</span>
            <span class="text-base-content block truncate font-medium">{{ dmxConnector }}</span>
          </div>
        </div>
        <div v-if="dimensions" class="flex items-center gap-2" title="Dimensions">
          <BoxIcon class="text-base-content/70 h-4 w-4 shrink-0" />
          <div class="min-w-0">
            <span class="text-base-content/50 block text-[10px] font-semibold uppercase"
              >Dimensions</span
            >
            <span class="text-base-content block truncate font-medium">{{ dimensions }}</span>
          </div>
        </div>
      </div>

      <!-- Channels Header -->
      <div class="border-base-content/10 flex items-center justify-between border-b pt-1 pb-1.5">
        <div
          class="text-base-content/70 flex items-center gap-1.5 text-xs font-bold tracking-wider uppercase"
        >
          <SlidersIcon class="text-primary h-3.5 w-3.5" />
          <span>Channels</span>
        </div>
        <Badge sm primary outline class="font-medium"> {{ dmxChannelCount }} channels </Badge>
      </div>

      <!-- When fixture is not yet downloaded from GDTF-Share -->
      <div
        v-if="isPlaceholder && fixture.rid"
        class="bg-base-300/40 border-base-content/10 flex flex-col items-center justify-center gap-2 rounded-xl border border-dashed p-6 text-center"
      >
        <template v-if="isLoadingPreview">
          <span class="loading loading-spinner text-primary loading-md"></span>
          <div class="flex flex-col gap-1">
            <span class="text-base-content/80 text-sm font-semibold">
              Loading channels preview...
            </span>
            <span class="text-base-content/50 max-w-sm text-xs">
              Fetching channel definitions from GDTF-Share into preview cache.
            </span>
          </div>
        </template>
        <template v-else>
          <SlidersIcon class="text-base-content/30 h-8 w-8" />
          <div class="flex flex-col gap-1">
            <span class="text-base-content/80 text-sm font-semibold">
              {{ dmxChannelCount }} DMX Channels
            </span>
            <span class="text-base-content/50 max-w-sm text-xs">
              Complete channel names and attributes will be downloaded from GDTF-Share when patched.
            </span>
          </div>
        </template>
      </div>

      <!-- Multi-column Channel Grid (Clean 2-column layout from Inspector design) -->
      <div v-else class="max-h-64 overflow-y-auto pr-1">
        <div class="grid grid-cols-2 gap-1.5">
          <div
            v-for="(ch, idx) in channels"
            :key="idx"
            class="bg-base-300/50 hover:bg-base-300 border-base-content/5 flex items-center gap-2 rounded-lg border px-2.5 py-1.5 text-xs transition-colors"
          >
            <Badge xs primary class="shrink-0 font-mono font-bold">
              {{ idx + 1 }}
            </Badge>
            <span class="text-base-content truncate font-medium" :title="formatChannelName(ch)">
              {{ formatChannelName(ch) }}
            </span>
          </div>
        </div>
      </div>
    </CardBody>
  </Card>
</template>
