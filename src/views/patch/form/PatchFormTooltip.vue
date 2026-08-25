<script setup lang="ts">
import { Badge, Card, CardBody, CardTitle, Divider } from "daisy-ui-kit";
import { computed } from "vue";

import { FixtureIcon } from "@/components/icons";
import { formatChannelName } from "@/libs/fixture.ts";
import { useFixtureBrowser } from "@/stores/fixture-browser.ts";
import type { FixtureType } from "@/types/fixtures.ts";

const props = defineProps<{
  fixture: FixtureType;
  mode: number;
}>();

const fixtureBrowser = useFixtureBrowser();

const manufacturerName = computed(() => {
  const m = (fixtureBrowser.manufacturers ?? {})[props.fixture.manufacturerKey];
  return m?.name || props.fixture.manufacturerKey;
});
const modeData = computed(() => props.fixture.modes?.[props.mode]);
const channels = computed(() => modeData.value?.channels ?? []);
</script>

<template>
  <Card class="min-w-72">
    <CardBody>
      <CardTitle class="flex flex-col items-start">
        <Badge primary xs outline>
          {{ manufacturerName }}
        </Badge>
        <div class="flex items-center gap-2">
          <FixtureIcon
            :category="fixture.categories"
            :size="20"
            class="text-primary shrink-0 opacity-90"
          />
          <h2 class="text-base-content truncate text-base leading-snug font-bold tracking-tight">
            {{ fixture.name }} ({{ modeData?.name }})
          </h2>
        </div>
      </CardTitle>
      <Divider
        class="text-base-content/70 before:bg-base-content/20 after:bg-base-content/20 my-1 text-xs"
      >
        {{ channels.length }} channels
      </Divider>
      <div class="flex max-h-72 flex-col gap-1 overflow-y-auto">
        <div
          v-for="(ch, idx) in channels"
          :key="idx"
          class="bg-base-100 hover:bg-base-200/80 border-base-content/5 flex items-center justify-between gap-2 rounded-lg border p-1.5 text-xs transition-colors"
        >
          <div class="flex min-w-0 flex-1 items-center gap-2">
            <Badge xs primary> Ch {{ idx + 1 }} </Badge>
            <span class="text-base-content truncate text-xs font-medium">
              {{ formatChannelName(ch) }}
            </span>
          </div>
        </div>
      </div>
    </CardBody>
  </Card>
</template>
