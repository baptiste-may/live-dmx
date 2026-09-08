<script setup lang="ts">
import { Card, CardBody, CardTitle, Table } from "daisy-ui-kit";
import { CloudIcon, SparklesIcon, SunIcon, ZapIcon } from "lucide-vue-next";
import { storeToRefs } from "pinia";

import FlashButton from "@/components/inputs/FlashButton.vue";
import Knob from "@/components/inputs/Knob.vue";
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import { useSettings } from "@/stores/settings.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";

const { groups } = storeToRefs(useShowGroups());
const liveButtons = useLiveButtons();
const settings = useSettings();
</script>

<template>
  <Card class="bg-base-200 h-full overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <CardTitle is="h2" v-if="settings.live.showSectionTitles" bold size="2xl">Flashs</CardTitle>
      <div class="min-h-0 flex-1 overflow-auto">
        <Table sm>
          <thead>
            <tr>
              <th></th>
              <th colspan="2" class="text-center">Fade</th>
              <th colspan="2" class="text-center">Speed</th>
            </tr>
            <tr>
              <th></th>
              <th class="text-center">
                <div class="flex justify-center">
                  <Knob
                    :model-value="liveButtons.simpleFlashFade"
                    color="warning"
                    :min="0"
                    :max="3"
                    :step="0.1"
                    :default-value="0"
                    unit="s"
                    @update:model-value="liveButtons.setSimpleFlashFade($event)"
                  />
                </div>
              </th>
              <th class="text-center">
                <div class="flex justify-center">
                  <Knob
                    :model-value="liveButtons.whiteFlashFade"
                    color="warning"
                    :min="0"
                    :max="3"
                    :step="0.1"
                    :default-value="0"
                    unit="s"
                    @update:model-value="liveButtons.setWhiteFlashFade($event)"
                  />
                </div>
              </th>
              <th class="text-center">
                <div class="flex justify-center">
                  <Knob
                    :model-value="liveButtons.strobeRate"
                    color="warning"
                    :min="0"
                    :max="100"
                    @update:model-value="liveButtons.setStrobeRate($event)"
                  />
                </div>
              </th>
              <th class="text-center">
                <div class="flex justify-center">
                  <Knob
                    :model-value="liveButtons.randomStrobeRate"
                    color="warning"
                    :min="0"
                    :max="100"
                    @update:model-value="liveButtons.setRandomStrobeRate($event)"
                  />
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="group in groups" :key="group.id">
              <th class="text-end">{{ group.name }} :</th>
              <td>
                <FlashButton
                  :icon="SunIcon"
                  label="Simple"
                  :model-value="liveButtons.isFlashActive(group.id, 'simple')"
                  @update:model-value="liveButtons.setFlash(group.id, 'simple', $event)"
                />
              </td>
              <td>
                <FlashButton
                  :icon="CloudIcon"
                  label="White"
                  :model-value="liveButtons.isFlashActive(group.id, 'white')"
                  @update:model-value="liveButtons.setFlash(group.id, 'white', $event)"
                />
              </td>
              <td>
                <FlashButton
                  :icon="ZapIcon"
                  label="Strobe"
                  :model-value="liveButtons.isFlashActive(group.id, 'strobe')"
                  @update:model-value="liveButtons.setFlash(group.id, 'strobe', $event)"
                />
              </td>
              <td>
                <FlashButton
                  :icon="SparklesIcon"
                  label="Random"
                  :model-value="liveButtons.isFlashActive(group.id, 'randomStrobe')"
                  @update:model-value="liveButtons.setFlash(group.id, 'randomStrobe', $event)"
                />
              </td>
            </tr>
          </tbody>
        </Table>
      </div>
    </CardBody>
  </Card>
</template>
