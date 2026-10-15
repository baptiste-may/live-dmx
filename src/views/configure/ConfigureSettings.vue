<script setup lang="ts">
import {
  Badge,
  Card,
  CardBody,
  CardTitle,
  Input,
  Join,
  Label,
  Range,
  Select,
  Toggle,
} from "daisy-ui-kit";
import { computed } from "vue";

import Fader from "@/components/inputs/Fader.vue";
import { useShowConfiguration } from "@/stores/show/show-configuration.ts";

import ConfigureSettingsGrid from "./settings/ConfigureSettingsGrid.vue";
import type { ColorPreference, PixelPreference } from "./types";

const configure = useShowConfiguration();
const settings = computed(() => configure.selectedSettings);

const colorPreference = computed<ColorPreference | "None">({
  get: () => (configure.hasColor ? settings.value.colorPreference : "None"),
  set: (value: ColorPreference | "None") => {
    if (value !== "None") settings.value.colorPreference = value;
  },
});

const pixelPreference = computed<PixelPreference | "None">({
  get: () => (configure.hasPixels ? settings.value.pixelPreference : "None"),
  set: (value: PixelPreference | "None") => {
    if (value !== "None") settings.value.pixelPreference = value;
  },
});

const pixelGroupCount = computed<number>({
  get: () => settings.value.pixelGroupCount,
  set: (val: number) => {
    if (typeof val !== "number" || Number.isNaN(val)) return;
    settings.value.pixelGroupCount = Math.max(2, Math.min(val, configure.maxPixels));
  },
});

const panRange = computed<[number, number]>({
  get: () =>
    [
      Math.max(
        configure.groupPanBounds[0],
        Math.min(settings.value.panMin, configure.groupPanBounds[1]),
      ),
      Math.min(
        configure.groupPanBounds[1],
        Math.max(settings.value.panMax, configure.groupPanBounds[0]),
      ),
    ] as [number, number],
  set: (range: [number, number]) => {
    const [min, max] = range;
    settings.value.panMin = Math.max(
      configure.groupPanBounds[0],
      Math.min(min, max, configure.groupPanBounds[1]),
    );
    settings.value.panMax = Math.min(
      configure.groupPanBounds[1],
      Math.max(min, max, configure.groupPanBounds[0]),
    );
  },
});

const tiltRange = computed<[number, number]>({
  get: () =>
    [
      Math.max(
        configure.groupTiltBounds[0],
        Math.min(settings.value.tiltMin, configure.groupTiltBounds[1]),
      ),
      Math.min(
        configure.groupTiltBounds[1],
        Math.max(settings.value.tiltMax, configure.groupTiltBounds[0]),
      ),
    ] as [number, number],
  set: (range: [number, number]) => {
    const [min, max] = range;
    settings.value.tiltMin = Math.max(
      configure.groupTiltBounds[0],
      Math.min(min, max, configure.groupTiltBounds[1]),
    );
    settings.value.tiltMax = Math.min(
      configure.groupTiltBounds[1],
      Math.max(min, max, configure.groupTiltBounds[0]),
    );
  },
});

const panMin = computed<number>({
  get: () => settings.value.panMin,
  set: (val: number) => {
    if (typeof val !== "number" || Number.isNaN(val)) return;
    const clamped = Math.max(
      configure.groupPanBounds[0],
      Math.min(val, configure.groupPanBounds[1]),
    );
    settings.value.panMin = clamped;
    if (clamped > settings.value.panMax) {
      settings.value.panMax = clamped;
    }
  },
});

const panMax = computed<number>({
  get: () => settings.value.panMax,
  set: (val: number) => {
    if (typeof val !== "number" || Number.isNaN(val)) return;
    const clamped = Math.max(
      configure.groupPanBounds[0],
      Math.min(val, configure.groupPanBounds[1]),
    );
    settings.value.panMax = clamped;
    if (clamped < settings.value.panMin) {
      settings.value.panMin = clamped;
    }
  },
});

const tiltMin = computed<number>({
  get: () => settings.value.tiltMin,
  set: (val: number) => {
    if (typeof val !== "number" || Number.isNaN(val)) return;
    const clamped = Math.max(
      configure.groupTiltBounds[0],
      Math.min(val, configure.groupTiltBounds[1]),
    );
    settings.value.tiltMin = clamped;
    if (clamped > settings.value.tiltMax) {
      settings.value.tiltMax = clamped;
    }
  },
});

const tiltMax = computed<number>({
  get: () => settings.value.tiltMax,
  set: (val: number) => {
    if (typeof val !== "number" || Number.isNaN(val)) return;
    const clamped = Math.max(
      configure.groupTiltBounds[0],
      Math.min(val, configure.groupTiltBounds[1]),
    );
    settings.value.tiltMax = clamped;
    if (clamped < settings.value.tiltMin) {
      settings.value.tiltMin = clamped;
    }
  },
});
</script>

<template>
  <Card class="bg-base-200 h-full w-full overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <CardTitle is="h2" bold size="2xl" class="flex justify-between">
        Settings
        <Badge info sm>{{ configure.selectedGroup?.name ?? "All" }}</Badge>
      </CardTitle>
      <div class="flex gap-8">
        <div class="flex flex-col gap-4">
          <div class="flex items-center justify-between gap-4">
            <Label for="color-preference">Color preference</Label>
            <Select
              id="color-preference"
              v-model="colorPreference"
              :options="configure.hasColor ? configure.colorOptions : ['None']"
              :disabled="!configure.hasColor"
              class="w-36"
            />
          </div>
          <div class="flex items-center justify-between gap-4">
            <Label for="pixel-preference">Pixel preference</Label>
            <Select
              id="pixel-preference"
              v-model="pixelPreference"
              :options="configure.hasPixels ? configure.pixelOptions : ['None']"
              :disabled="!configure.hasPixels"
              class="w-36"
            />
          </div>
          <div
            v-if="configure.hasPixels && pixelPreference === 'Grouped'"
            class="flex items-center justify-between gap-4"
          >
            <Label for="pixel-group-count">Group count</Label>
            <Input
              id="pixel-group-count"
              v-model.number="pixelGroupCount"
              type="number"
              :min="2"
              :max="configure.maxPixels"
              class="w-36"
              sm
            />
          </div>
          <div class="relative aspect-square min-h-48">
            <ConfigureSettingsGrid />
            <div
              v-if="configure.hasPan"
              class="absolute top-1/2 left-1/2 h-(--range-thumb-size,1.5rem) w-full -translate-x-1/2 -translate-y-1/2"
            >
              <Range
                :key="`pan-${configure.selectedGroupId}-${configure.groupPanBounds[0]}-${configure.groupPanBounds[1]}`"
                v-model="panRange"
                :min="configure.groupPanBounds[0]"
                :max="configure.groupPanBounds[1]"
                aria-label="Pan range"
                primary
                class="w-full"
              />
            </div>
            <div
              v-if="configure.hasTilt"
              class="absolute top-1/2 left-1/2 h-(--range-thumb-size,1.5rem) w-full -translate-x-1/2 -translate-y-1/2 -rotate-90"
            >
              <Range
                :key="`tilt-${configure.selectedGroupId}-${configure.groupTiltBounds[0]}-${configure.groupTiltBounds[1]}`"
                v-model="tiltRange"
                :min="configure.groupTiltBounds[0]"
                :max="configure.groupTiltBounds[1]"
                aria-label="Tilt range"
                primary
                class="w-full"
              />
            </div>
          </div>
          <div class="flex gap-4">
            <Label class="flex items-center justify-between gap-2"> Pan : </Label>
            <Join>
              <Input
                v-model.number="panMin"
                type="number"
                join
                sm
                aria-label="Pan minimum"
                :disabled="!configure.hasPan"
                :min="configure.groupPanBounds[0]"
                :max="configure.groupPanBounds[1]"
              />
              <Input
                v-model.number="panMax"
                type="number"
                join
                sm
                aria-label="Pan maximum"
                :disabled="!configure.hasPan"
                :min="configure.groupPanBounds[0]"
                :max="configure.groupPanBounds[1]"
              />
            </Join>
            <Label class="flex items-center justify-between gap-2">
              Inverse Pan
              <Toggle v-model="settings.inversePan" :disabled="!configure.hasPan" />
            </Label>
          </div>
          <div class="flex gap-4">
            <Label class="flex items-center justify-between gap-2"> Tilt : </Label>
            <Join>
              <Input
                v-model.number="tiltMin"
                type="number"
                join
                sm
                aria-label="Tilt minimum"
                :disabled="!configure.hasTilt"
                :min="configure.groupTiltBounds[0]"
                :max="configure.groupTiltBounds[1]"
              />
              <Input
                v-model.number="tiltMax"
                type="number"
                join
                sm
                aria-label="Tilt maximum"
                :disabled="!configure.hasTilt"
                :min="configure.groupTiltBounds[0]"
                :max="configure.groupTiltBounds[1]"
              />
            </Join>
            <Label class="flex items-center justify-between gap-2">
              Inverse Tilt
              <Toggle v-model="settings.inverseTilt" :disabled="!configure.hasTilt" />
            </Label>
          </div>
        </div>
        <div class="flex flex-1 flex-col items-center justify-between gap-4">
          <Label for="max-power">Max Power</Label>
          <Fader
            v-model="settings.maxPower"
            :min="0"
            :max="100"
            aria-label="Max Power"
            show-scale
            scale-unit="%"
          />
        </div>
      </div>
    </CardBody>
  </Card>
</template>
