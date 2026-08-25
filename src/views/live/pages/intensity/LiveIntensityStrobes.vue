<script setup lang="ts">
import { Card, CardBody, CardTitle, Table } from "daisy-ui-kit";
import { storeToRefs } from "pinia";

import FlashButton from "@/components/inputs/FlashButton.vue";
import Knob from "@/components/inputs/Knob.vue";
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import { useShowGroups } from "@/stores/show/show-groups.ts";

const { groups } = storeToRefs(useShowGroups());
const liveButtons = useLiveButtons();
</script>

<template>
  <Card class="bg-base-200 h-full overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <CardTitle is="h2" bold size="2xl">Strobes</CardTitle>
      <div class="min-h-0 flex-1 overflow-auto">
        <Table sm>
          <thead>
            <tr>
              <th></th>
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
                  :model-value="liveButtons.isFlashActive(group.id, 'strobe')"
                  @update:model-value="liveButtons.setFlash(group.id, 'strobe', $event)"
                >
                  Strobe
                </FlashButton>
              </td>
              <td>
                <FlashButton
                  :model-value="liveButtons.isFlashActive(group.id, 'randomStrobe')"
                  @update:model-value="liveButtons.setFlash(group.id, 'randomStrobe', $event)"
                >
                  Random
                </FlashButton>
              </td>
            </tr>
          </tbody>
        </Table>
      </div>
    </CardBody>
  </Card>
</template>
