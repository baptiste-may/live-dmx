<script setup lang="ts">
import { Badge, Button, Flex, Menu, MenuItem, Skeleton, Text } from "daisy-ui-kit";
import { BrushCleaningIcon, FileIcon, RotateCwIcon, Trash2Icon } from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";

import { formatDate } from "@/libs/date";
import { sliceString } from "@/libs/utils";
import { useApp } from "@/stores/app";
import { useRecentShows } from "@/stores/recent-shows";

const app = useApp();
const recentShows = useRecentShows();

const isRefreshing = ref(false);
const hasUnavailableShows = computed(() => recentShows.shows.some((show) => !show.available));

async function refresh() {
  isRefreshing.value = true;
  try {
    await recentShows.load();
  } finally {
    isRefreshing.value = false;
  }
}

onMounted(refresh);
</script>

<template>
  <Flex justify-between items-center class="mt-2">
    <Text xs bold class="tracking-widest uppercase opacity-50">Recent</Text>
    <Flex>
      <Button
        ghost
        xs
        circle
        aria-label="Clean unavailable shows"
        :disabled="isRefreshing || !hasUnavailableShows"
        @click="recentShows.clean()"
      >
        <BrushCleaningIcon class="h-3 w-3" />
      </Button>
      <Button ghost xs circle aria-label="Refresh shows" :disabled="isRefreshing" @click="refresh">
        <RotateCwIcon class="h-3 w-3" :class="[isRefreshing ? 'animate-spin' : '']" />
      </Button>
    </Flex>
  </Flex>
  <Menu class="bg-base-100 rounded-box flex max-h-96 w-md flex-col gap-2 overflow-y-auto">
    <template v-if="isRefreshing && recentShows.shows.length === 0">
      <MenuItem v-for="i in 3" :key="i" class="p-2">
        <Flex col gap-2 w-full>
          <Skeleton class="h-4 w-1/2" />
          <Skeleton class="h-2 w-full" />
          <Skeleton class="h-2 w-1/4" />
        </Flex>
      </MenuItem>
    </template>
    <MenuItem v-else-if="recentShows.shows.length === 0" disabled>
      <span>No shows found</span>
    </MenuItem>
    <MenuItem
      v-for="show in recentShows.shows"
      :key="show.path"
      class="grid grid-cols-[1fr_auto] items-center gap-1"
    >
      <button
        type="button"
        class="flex min-w-0 flex-1 items-center gap-2 disabled:cursor-not-allowed disabled:opacity-60"
        :disabled="!show.available"
        @click="app.loadShow(show.path)"
      >
        <FileIcon class="h-4 w-4 shrink-0" />
        <Flex col class="w-full">
          <span class="flex justify-between">
            <Text bold class="flex w-full items-center gap-2 truncate">
              {{ show.displayName }}
              <Badge v-if="!show.available" outline xs warning>File not found</Badge>
            </Text>
            <Text xs class="shrink-0 self-start opacity-50">
              {{ formatDate(show.updatedAt) }}
            </Text>
          </span>
          <Text xs italic class="w-full truncate font-mono opacity-70" :title="show.path">
            {{ sliceString(show.path, 42) }}
          </Text>
        </Flex>
      </button>
      <Button
        xs
        neutral
        class="h-full shrink-0"
        :aria-label="`Remove ${show.displayName} from recent shows`"
        @click.stop="recentShows.remove(show.path)"
      >
        <Trash2Icon class="h-3 w-3" />
      </Button>
    </MenuItem>
  </Menu>
</template>
