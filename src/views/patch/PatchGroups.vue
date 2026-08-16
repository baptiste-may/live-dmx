<script setup lang="ts">
import { Card, CardBody, Text } from "daisy-ui-kit";

import PatchGroupsList from "@/views/patch/groups/PatchGroupsList.vue";
import PatchGroupsTable from "@/views/patch/groups/PatchGroupsTable.vue";
</script>

<template>
  <Card class="bg-base-200 h-full w-full overflow-hidden">
    <CardBody class="flex h-full min-h-0 flex-col gap-4">
      <Text is="h2" bold size="2xl">Groups</Text>
      <PatchGroupsList />
      <PatchGroupsTable />
    </CardBody>
  </Card>
</template>
