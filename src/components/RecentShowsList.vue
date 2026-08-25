<script setup lang="ts">
import { Button, Flex, Menu, MenuItem, Skeleton, Text } from "daisy-ui-kit";
import { FileIcon, RotateCwIcon } from "lucide-vue-next";
import { onMounted, ref } from "vue";

import { formatDate } from "@/libs/date";
import { useApp } from "@/stores/app";
import { useRecentShows } from "@/stores/recent-shows";

const app = useApp();
const recentShows = useRecentShows();

const isRefreshing = ref(false);

function refresh() {
  isRefreshing.value = true;
  recentShows.load();
  isRefreshing.value = false;
}

onMounted(refresh);
</script>

<template>
  <Flex justify-between items-center w-full mt-4 px-2>
    <Text xs bold class="tracking-widest uppercase opacity-50">Recent</Text>
    <Button ghost xs circle aria-label="Refresh shows" :disabled="isRefreshing" @click="refresh">
      <RotateCwIcon class="h-3 w-3" :class="[isRefreshing ? 'animate-spin' : '']" />
    </Button>
  </Flex>
  <Menu class="bg-base-100 rounded-box mt-1 max-h-96 w-md overflow-y-auto">
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
    <MenuItem v-for="(show, i) in recentShows.shows" :key="i">
      <button type="button" class="w-full text-left" @click="app.loadShow(show.path)">
        <FileIcon class="mt-1 h-4 w-4" />
        <Flex col items-start overflow-hidden w-full>
          <Text truncate w-full bold>{{ show.displayName }}</Text>
          <Text xs class="w-full truncate italic opacity-70" :title="show.path">
            {{ show.path }}
          </Text>
          <Text xs class="opacity-50">
            {{ formatDate(show.updatedAt) }}
          </Text>
        </Flex>
      </button>
    </MenuItem>
  </Menu>
</template>
