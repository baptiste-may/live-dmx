<script setup lang="ts">
import { Card, CardBody, CardTitle, Table } from "daisy-ui-kit";
import {
  FlipHorizontal2Icon,
  FoldHorizontalIcon,
  MoveLeftIcon,
  MoveRightIcon,
} from "lucide-vue-next";
import { storeToRefs } from "pinia";

import Knob from "@/components/inputs/Knob.vue";
import ToggleButton from "@/components/inputs/ToggleButton.vue";
import { useLiveButtons } from "@/stores/live/live-buttons.ts";
import type { ColorFadeOrder } from "@/types/live.ts";

const liveButtons = useLiveButtons();
const { colorFadeSpeed, colorFadeOrder } = storeToRefs(liveButtons);

function handleOrderSelect(order: ColorFadeOrder, active: boolean) {
  liveButtons.setColorFadeOrder(active ? order : null);
}
</script>

<template>
  <Card class="bg-base-200 h-full overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <CardTitle is="h2" bold size="2xl">Fade</CardTitle>
      <Table sm>
        <tbody>
          <tr>
            <th>Speed</th>
          </tr>
          <tr>
            <td>
              <Knob
                size="lg"
                :min="0"
                :max="3"
                :step="0.25"
                :default-value="1"
                unit="s"
                :model-value="colorFadeSpeed"
                @update:model-value="liveButtons.setColorFadeSpeed($event)"
              />
            </td>
          </tr>
          <tr>
            <th>Order</th>
          </tr>
          <tr>
            <td>
              <ToggleButton
                aria-label="Fade order left"
                :icon="MoveLeftIcon"
                :model-value="colorFadeOrder === 'left'"
                color="secondary"
                @update:model-value="handleOrderSelect('left', $event)"
              />
            </td>
          </tr>
          <tr>
            <td>
              <ToggleButton
                aria-label="Fade order right"
                :icon="MoveRightIcon"
                :model-value="colorFadeOrder === 'right'"
                color="secondary"
                @update:model-value="handleOrderSelect('right', $event)"
              />
            </td>
          </tr>
          <tr>
            <td>
              <ToggleButton
                aria-label="Fade order wing"
                :icon="FlipHorizontal2Icon"
                :model-value="colorFadeOrder === 'wing'"
                color="secondary"
                @update:model-value="handleOrderSelect('wing', $event)"
              />
            </td>
          </tr>
          <tr>
            <td>
              <ToggleButton
                aria-label="Fade order center"
                :icon="FoldHorizontalIcon"
                :model-value="colorFadeOrder === 'center'"
                color="secondary"
                @update:model-value="handleOrderSelect('center', $event)"
              />
            </td>
          </tr>
        </tbody>
      </Table>
    </CardBody>
  </Card>
</template>
