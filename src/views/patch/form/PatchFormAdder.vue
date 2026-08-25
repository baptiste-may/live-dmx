<script setup lang="ts">
import { Badge, Button, Card, CardBody, Input } from "daisy-ui-kit";
import { PlusIcon, SlidersIcon } from "lucide-vue-next";
import { computed, watch } from "vue";

import { FixtureIcon } from "@/components/icons";
import NumberStepper from "@/components/inputs/NumberStepper.vue";
import { formatChannelName } from "@/libs/fixture.ts";
import { useFixtureBrowser } from "@/stores/fixture-browser.ts";
import { usePatchForm } from "@/stores/patch/patch-form.ts";
import { useShowPatch } from "@/stores/show/show-patch.ts";

const form = usePatchForm();
const fixtureBrowser = useFixtureBrowser();
const showPatch = useShowPatch();

watch(
  () => showPatch.fixtures,
  () => {
    if (!form.isFixtureSelected()) {
      form.setDefaultLocation();
    }
  },
  { immediate: true },
);

const manufacturerName = computed(() => {
  const manufacturerKey = form.selectedFixture?.manufacturerKey;
  const m = (fixtureBrowser.manufacturers ?? {})[manufacturerKey ?? ""];
  return m?.name || form.selectedFixture?.manufacturerKey;
});
</script>

<template>
  <Card class="bg-base-300 shrink-0 shadow-inner">
    <CardBody class="flex flex-col gap-4 p-4">
      <!-- Selected Fixture & Channel Details -->
      <Transition name="card-slide">
        <div
          v-if="form.selectedFixture && form.selectedMode"
          class="bg-base-200/80 border-base-content/10 flex flex-col gap-2 rounded-xl border p-3"
        >
          <div class="border-base-content/10 flex items-center justify-between gap-2 border-b pb-2">
            <div class="flex min-w-0 items-center gap-2">
              <FixtureIcon
                :category="form.selectedFixture.categories"
                :size="20"
                class="text-primary shrink-0"
              />
              <div class="truncate">
                <span class="text-base-content block truncate text-sm font-bold">
                  {{ form.selectedFixture.name }}
                </span>
                <span class="block truncate text-xs opacity-60">
                  {{ manufacturerName }}
                </span>
              </div>
            </div>
            <Badge sm primary outline>
              {{ form.selectedMode.name }}
            </Badge>
          </div>

          <!-- Full Scrollable Channel List -->
          <div class="mt-1 flex flex-col gap-1">
            <div
              class="text-base-content/70 flex items-center gap-1 text-[11px] font-bold tracking-wider uppercase"
            >
              <SlidersIcon class="text-primary h-3.5 w-3.5" />
              {{ form.selectedMode.channels?.length ?? 0 }} channels
            </div>
            <div
              v-if="form.selectedMode.channels && form.selectedMode.channels.length > 0"
              class="border-base-content/10 bg-base-100/50 flex max-h-36 flex-col gap-1 overflow-y-auto rounded-lg border p-1"
            >
              <div
                v-for="(ch, idx) in form.selectedMode.channels"
                :key="idx"
                class="bg-base-200/60 hover:bg-base-200 flex items-center justify-between gap-2 rounded p-1 text-xs"
              >
                <div class="flex min-w-0 items-center gap-1.5">
                  <Badge xs primary> Ch {{ idx + 1 }} </Badge>
                  <span class="text-base-content truncate font-medium">
                    {{ formatChannelName(ch) }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Transition>

      <!-- Patch PatchVisualizerForm Controls -->
      <form class="grid grid-cols-2 items-center gap-4" @submit.prevent="form.patch">
        <span class="text-lg font-bold opacity-70">Universe :</span>
        <NumberStepper v-model="form.universe" :min="1" :max="512" />
        <span class="text-lg font-bold opacity-70">Start Address :</span>
        <NumberStepper v-model="form.address" :min="1" :max="512" />
        <span class="text-lg font-bold opacity-70">Count :</span>
        <NumberStepper v-model="form.count" :min="1" :max="512" />
        <span class="text-lg font-bold opacity-70">Name :</span>
        <Input v-model="form.name" type="text" />
        <Button type="submit" primary class="col-span-2 mt-2" :disabled="!form.isFixtureSelected()">
          <PlusIcon class="h-5 w-5" />
          Patch
        </Button>
      </form>
    </CardBody>
  </Card>
</template>

<style scoped>
.card-slide-enter-active,
.card-slide-leave-active {
  transition:
    opacity 0.5s ease,
    transform 0.5s ease;
}

.card-slide-enter-from,
.card-slide-leave-to {
  opacity: 0;
  transform: translateY(12px);
}
</style>
