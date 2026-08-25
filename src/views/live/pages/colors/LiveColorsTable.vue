<script setup lang="ts">
import { Card, CardBody, CardTitle, Table } from "daisy-ui-kit";
import { storeToRefs } from "pinia";

import { useShowGroups } from "@/stores/show/show-groups.ts";
import LiveColorsTableHeader from "@/views/live/pages/colors/table/LiveColorsTableHeader.vue";
import LiveColorsTableRow from "@/views/live/pages/colors/table/LiveColorsTableRow.vue";

const { groups } = storeToRefs(useShowGroups());
</script>

<template>
  <Card class="bg-base-200 h-full overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <CardTitle is="h2" bold size="2xl">Colors</CardTitle>
      <div class="min-h-0 flex-1 overflow-auto">
        <Table sm class="w-auto">
          <thead>
            <LiveColorsTableHeader />
          </thead>
          <tbody>
            <LiveColorsTableRow
              v-for="group in groups.filter((g) => g.id !== 0)"
              :key="group.id"
              :group="group"
            />
          </tbody>
        </Table>
      </div>
    </CardBody>
  </Card>
</template>
