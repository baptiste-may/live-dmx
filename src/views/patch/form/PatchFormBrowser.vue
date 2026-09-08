<script setup lang="ts">
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useDebounceFn } from "@vueuse/core";
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
  Skeleton,
  SkeletonText,
  Text,
  Tooltip,
  TooltipContent,
  TooltipTarget,
} from "daisy-ui-kit";
import { GlobeIcon, RotateCwIcon, SearchIcon, UploadIcon, XIcon } from "lucide-vue-next";
import { onMounted, ref, watch } from "vue";

import { FixtureIcon } from "@/components/icons";
import { useFixtureBrowser } from "@/stores/fixture-browser";
import { usePatchForm } from "@/stores/patch/patch-form";
import { useSettings } from "@/stores/settings";

import PatchFormTooltip from "./PatchFormTooltip.vue";

const browser = useFixtureBrowser();
const patchForm = usePatchForm();
const settings = useSettings();

const openManufacturerId = ref<string | null>(null);
const openFixtureKeys = ref<Record<string, boolean>>({});
const searchInput = ref(browser.searchQuery);

const debouncedSearch = useDebounceFn((val: string) => {
  browser.search(val);
}, 150);

function onSearchInput(val: string) {
  searchInput.value = val;
  debouncedSearch(val);
}

function handleCleanButton() {
  searchInput.value = "";
  browser.clearSearch();
  patchForm.reset();
}

async function handleImportGdtf() {
  try {
    const selected = await openDialog({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "GDTF Fixture",
          extensions: ["gdtf", "zip"],
        },
      ],
    });
    if (selected && typeof selected === "string") {
      await browser.importGdtf(selected);
    }
  } catch {
    // Ignored in non-Tauri / test environments
  }
}

function toggleFixture(key: string, event: Event) {
  const target = event.target as HTMLDetailsElement;
  openFixtureKeys.value[key] = target.open;
}

watch(
  () => browser.searchQuery,
  (val) => {
    if (searchInput.value !== val) {
      searchInput.value = val;
    }
  },
);

watch(
  () => browser.filteredManufacturers,
  (filtered) => {
    const keys = Object.keys(filtered);
    if (keys.length === 1) {
      openManufacturerId.value = keys[0];
    } else if (openManufacturerId.value && !filtered[openManufacturerId.value]) {
      openManufacturerId.value = null;
    }
  },
  { immediate: true },
);

onMounted(browser.init);
</script>

<template>
  <Flex col class="min-h-0 flex-1">
    <Join class="w-full">
      <Label input join class="flex-1">
        <LoadingSpinner v-if="browser.isLoading" size="sm" primary />
        <SearchIcon v-else class="h-[1em] opacity-50" />
        <Input
          placeholder="Search GDTF & generic fixtures..."
          :model-value="searchInput"
          @update:model-value="onSearchInput"
          @keydown.esc="handleCleanButton"
          @keydown.enter="browser.search(searchInput)"
        />
        <button
          v-if="searchInput"
          type="button"
          class="cursor-pointer opacity-50 hover:opacity-100"
          title="Clear search"
          aria-label="Clear search"
          @click="handleCleanButton"
        >
          <XIcon class="h-3.5 w-3.5" />
        </button>
      </Label>
      <Button v-if="searchInput" error join @click="handleCleanButton"> Clear </Button>
      <Button
        secondary
        join
        square
        title="Import .gdtf fixture file"
        aria-label="Import GDTF"
        @click="handleImportGdtf"
      >
        <UploadIcon class="h-4 w-4" />
      </Button>
      <Button
        :class="[browser.isAuthenticated ? 'btn-success text-success-content' : 'btn-ghost']"
        join
        square
        :title="
          browser.isAuthenticated
            ? 'GDTF-Share Connected (Settings)'
            : 'Connect GDTF-Share account (Settings)'
        "
        aria-label="GDTF-Share Settings"
        @click="settings.openModal"
      >
        <GlobeIcon class="h-4 w-4" />
      </Button>
      <Button
        secondary
        join
        square
        title="Refresh fixtures catalog"
        aria-label="Refresh fixtures"
        :disabled="browser.isLoading"
        @click="browser.refresh"
      >
        <RotateCwIcon class="h-4 w-4" :class="[browser.isLoading ? 'animate-spin' : '']" />
      </Button>
    </Join>

    <div class="mt-2 grow overflow-y-auto pr-2">
      <Flex v-if="browser.isLoading" col gap-2 class="h-full">
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
        class="text-base-content/50 h-56 gap-3 p-4 text-center"
      >
        <SearchIcon class="h-8 w-8 opacity-40" />
        <div class="flex flex-col gap-1">
          <Text size="sm">
            {{
              browser.searchQuery
                ? 'No fixtures found matching "' + browser.searchQuery + '"'
                : "No fixtures available"
            }}
          </Text>
          <Text v-if="!browser.isAuthenticated" size="xs" class="max-w-xs opacity-60">
            Connect your free GDTF-Share account to search and download thousands of manufacturer
            fixtures.
          </Text>
        </div>
        <Button v-if="!browser.isAuthenticated" xs primary @click="settings.openModal">
          <GlobeIcon class="h-3.5 w-3.5" />
          Connect GDTF-Share
        </Button>
        <Button v-else xs secondary @click="browser.syncCatalog">
          <RotateCwIcon class="h-3.5 w-3.5" />
          Sync GDTF-Share Catalog
        </Button>
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
        <CollapseTitle class="flex items-center justify-between pr-8 font-semibold">
          <span>{{ manufacturerName }}</span>
          <Badge v-if="browser.searchQuery.trim()" xs ghost>
            {{ Object.keys(fixtures).length }}
          </Badge>
        </CollapseTitle>
        <CollapseContent v-if="openManufacturerId === manufacturerId" class="p-0 px-2 pb-2">
          <Menu class="bg-base-100 w-full">
            <MenuItem v-for="(fixture, fixtureId) in fixtures" :key="fixtureId">
              <details @toggle="toggleFixture(manufacturerId + ':' + fixtureId, $event)">
                <summary class="flex items-center justify-between gap-2">
                  <div class="flex min-w-0 items-center gap-2">
                    <FixtureIcon
                      :category="fixture.categories"
                      :size="16"
                      class="text-primary shrink-0 opacity-80"
                    />
                    <span class="truncate">{{ fixture.name }}</span>
                  </div>
                  <div class="flex shrink-0 items-center gap-1">
                    <Badge v-if="fixture.isGeneric" ghost xs>Generic</Badge>
                    <Badge v-else-if="fixture.isInstalled" success xs outline>Installed</Badge>
                    <Badge v-else-if="fixture.rid" warning xs outline>GDTF</Badge>
                  </div>
                </summary>
                <ul v-if="openFixtureKeys[manufacturerId + ':' + fixtureId]">
                  <li
                    v-for="({ name, channels, dmxfootprint }, modeId) in fixture.modes"
                    :key="modeId"
                    class="mode-item block w-full"
                  >
                    <Tooltip position="right" class="block w-full">
                      <TooltipTarget class="block w-full">
                        <button
                          type="button"
                          :class="[
                            patchForm.selectedFixture?.manufacturerKey === manufacturerId &&
                            patchForm.selectedFixture?.fixtureKey === fixtureId &&
                            patchForm.selectedMode?.name === name
                              ? 'bg-primary text-primary-content font-semibold shadow-xs'
                              : 'text-base-content/80 hover:bg-base-content/10 hover:text-base-content',
                            'flex w-full cursor-pointer items-center justify-between gap-2 rounded-lg px-3 py-1.5 text-left text-sm transition-colors',
                          ]"
                          @click="patchForm.select(manufacturerId, fixtureId, modeId)"
                        >
                          <span class="truncate">{{ name }}</span>
                          <Badge xs secondary soft class="shrink-0">
                            {{ channels?.length || dmxfootprint || 0 }} ch
                          </Badge>
                        </button>
                      </TooltipTarget>
                      <TooltipContent>
                        <PatchFormTooltip :fixture="fixture" :mode="modeId" />
                      </TooltipContent>
                    </Tooltip>
                  </li>
                </ul>
              </details>
            </MenuItem>
          </Menu>
        </CollapseContent>
      </Collapse>
    </div>
  </Flex>
</template>
