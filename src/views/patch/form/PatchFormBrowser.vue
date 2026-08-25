<script setup lang="ts">
import {
  Badge,
  Button,
  Collapse,
  CollapseContent,
  CollapseTitle,
  Flex,
  Input,
  Join,
  Label,
  LoadingSpinner,
  Menu,
  MenuItem,
  Progress,
  Skeleton,
  SkeletonText,
  Text,
  Tooltip,
  TooltipContent,
  TooltipTarget,
} from "daisy-ui-kit";
import { DownloadIcon, SearchIcon } from "lucide-vue-next";
import { onMounted, ref } from "vue";

import { FixtureIcon } from "@/components/icons";
import { useFixtureBrowser } from "@/stores/fixture-browser";
import { usePatchForm } from "@/stores/patch/patch-form";

import PatchFormTooltip from "./PatchFormTooltip.vue";

const browser = useFixtureBrowser();
const patchForm = usePatchForm();

const openManufacturerId = ref<string | null>(null);
const openFixtureKeys = ref<Record<string, boolean>>({});

function handleCleanButton() {
  browser.clearSearch();
  patchForm.reset();
}

function toggleFixture(key: string, event: Event) {
  const target = event.target as HTMLDetailsElement;
  openFixtureKeys.value[key] = target.open;
}

onMounted(browser.init);
</script>

<template>
  <Flex col class="min-h-0 flex-1">
    <Flex
      v-if="browser.isInstalled === false && !browser.isDownloading"
      col
      items-center
      justify-center
      class="text-base-content/70 h-full min-h-64 gap-4 p-4 text-center"
    >
      <DownloadIcon class="text-primary/70 h-12 w-12" />
      <div class="flex flex-col gap-1">
        <Text class="text-base font-semibold">Fixture Library Required</Text>
        <Text size="xs" class="max-w-xs opacity-60">
          Download the Open Fixture Library to search and patch fixtures.
        </Text>
      </div>
      <Button primary @click="browser.downloadFixtures">
        <DownloadIcon class="h-4 w-4" />
        Download Fixtures
      </Button>
    </Flex>
    <template v-else>
      <Join class="w-full">
        <Label input join class="flex-1">
          <LoadingSpinner v-if="browser.isLoading || browser.isDownloading" size="sm" primary />
          <SearchIcon v-else class="h-[1em] opacity-50" />
          <Input
            placeholder="Search fixtures..."
            :model-value="browser.searchQuery"
            :disabled="browser.isLoading || browser.isDownloading"
            @update:model-value="browser.search"
          />
        </Label>
        <Button
          error
          join
          :disabled="browser.isLoading || browser.isDownloading"
          @click="handleCleanButton"
        >
          Clear
        </Button>
      </Join>
      <Flex v-if="browser.isLoading || browser.isDownloading" col class="mt-2 mb-1 w-full">
        <Flex justify-between items-center class="px-0.5 text-xs font-semibold opacity-70">
          <span>{{ browser.loadingStatus }}</span>
          <span>{{ browser.progress }}%</span>
        </Flex>
        <Progress :value="browser.progress" :max="100" primary class="mt-1 h-2 w-full rounded" />
      </Flex>
      <div class="mt-2 grow overflow-y-auto pr-2">
        <Flex v-if="browser.isLoading || browser.isDownloading" col gap-2 h-full>
          <div
            v-for="i in 6"
            :key="i"
            class="bg-base-100 border-base-content/5 mb-2 flex flex-col gap-2 rounded-lg border p-3"
          >
            <div class="flex items-center justify-between">
              <SkeletonText class="h-4 w-1/3 rounded" />
              <Skeleton class="h-4 w-4 rounded-full" />
            </div>
            <div
              v-if="i === 1 || i === 3"
              class="border-base-content/5 flex flex-col gap-2 border-t pt-2 pl-4"
            >
              <div class="flex items-center gap-2">
                <Skeleton class="h-4 w-4 rounded-full" />
                <SkeletonText class="h-3 w-1/2 rounded" />
              </div>
              <div class="flex items-center gap-2 pl-4">
                <SkeletonText class="h-3 w-1/3 rounded" />
                <Skeleton class="h-4 w-12 rounded-full" />
              </div>
            </div>
          </div>
        </Flex>
        <Flex
          v-else-if="Object.keys(browser.filteredManufacturers).length === 0"
          col
          items-center
          justify-center
          class="text-base-content/50 h-48 gap-2"
        >
          <SearchIcon class="h-8 w-8 opacity-40" />
          <Text size="sm">No fixtures found matching "{{ browser.searchQuery }}"</Text>
        </Flex>
        <Collapse
          v-for="(
            { name: manufacturerName, fixtures }, manufacturerId
          ) in browser.filteredManufacturers"
          v-else
          :key="manufacturerId"
          arrow
          class="bg-base-100 mb-2"
        >
          <input
            v-model="openManufacturerId"
            type="radio"
            name="fixtures-accordion"
            :value="manufacturerId"
          />
          <CollapseTitle class="font-semibold">
            {{ manufacturerName }}
          </CollapseTitle>
          <CollapseContent
            v-if="openManufacturerId === manufacturerId || Boolean(browser.searchQuery.trim())"
            class="p-0 px-2 pb-2"
          >
            <Menu class="bg-base-100 w-full">
              <MenuItem v-for="(fixture, fixtureId) in fixtures" :key="fixtureId">
                <details @toggle="toggleFixture(manufacturerId + ':' + fixtureId, $event)">
                  <summary class="flex items-center gap-2">
                    <FixtureIcon
                      :category="fixture.categories"
                      :size="16"
                      class="text-primary opacity-80"
                    />
                    <span>{{ fixture.name }}</span>
                  </summary>
                  <ul v-if="openFixtureKeys[manufacturerId + ':' + fixtureId]">
                    <MenuItem v-for="({ name, channels }, modeId) in fixture.modes" :key="modeId">
                      <Tooltip position="right" class="block w-full">
                        <TooltipTarget class="block w-full">
                          <button
                            type="button"
                            :class="[
                              patchForm.selectedFixture?.manufacturerKey === manufacturerId &&
                                patchForm.selectedFixture?.fixtureKey === fixtureId &&
                                patchForm.selectedMode?.name === name &&
                                'menu-active',
                              'flex w-full items-center gap-2 text-left',
                            ]"
                            @click="patchForm.select(manufacturerId, fixtureId, modeId)"
                          >
                            <span>{{ name }}</span>
                            <Badge xs secondary soft> {{ channels?.length || 0 }} ch </Badge>
                          </button>
                        </TooltipTarget>
                        <TooltipContent>
                          <PatchFormTooltip :fixture="fixture" :mode="modeId" />
                        </TooltipContent>
                      </Tooltip>
                    </MenuItem>
                  </ul>
                </details>
              </MenuItem>
            </Menu>
          </CollapseContent>
        </Collapse>
      </div>
    </template>
  </Flex>
</template>
