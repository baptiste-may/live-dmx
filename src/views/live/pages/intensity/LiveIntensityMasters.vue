<script setup lang="ts">
import { Badge, Card, CardBody, CardTitle } from "daisy-ui-kit";
import { storeToRefs } from "pinia";

import Fader from "@/components/inputs/Fader.vue";
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
      <CardTitle is="h2" v-if="settings.live.showSectionTitles" bold size="2xl">Masters</CardTitle>
      <div class="flex h-full min-h-0 w-full gap-4 overflow-x-auto overflow-y-auto">
        <div
          v-for="group in groups"
          :key="group.id"
          class="flex h-full flex-col items-center gap-2"
        >
          <Fader
            :model-value="liveButtons.getGroupMaster(group.id)"
            :label="group.id === 0 ? 'Grand Master' : group.name"
            :error="group.id === 0"
            @update:model-value="liveButtons.setGroupMaster(group.id, $event)"
          />
          <Badge neutral>
            {{ group.id === 0 ? "Grand Master" : group.name }}
          </Badge>
        </div>
      </div>
    </CardBody>
  </Card>
</template>
