<script setup lang="ts">
import { Button, Input, Join } from "daisy-ui-kit";
import { CheckIcon, XIcon } from "lucide-vue-next";
import { ref, watch } from "vue";

const modelValue = defineModel<string>({ default: "" });

const props = withDefaults(
  defineProps<{
    placeholder?: string;
    disabled?: boolean;
  }>(),
  {
    placeholder: "",
    disabled: false,
  },
);

const emit = defineEmits<{
  (e: "save", value: string): void;
  (e: "submit", value: string): void;
  (e: "cancel"): void;
}>();

const localValue = ref(modelValue.value);

watch(modelValue, (val) => {
  localValue.value = val;
});

function handleSave() {
  if (props.disabled) return;
  modelValue.value = localValue.value;
  emit("save", localValue.value);
  emit("submit", localValue.value);
}

function handleCancel() {
  localValue.value = modelValue.value;
  emit("cancel");
}
</script>

<template>
  <Join>
    <Input
      v-model="localValue"
      join
      type="text"
      class="w-32"
      :placeholder="placeholder"
      :disabled="disabled"
      @keydown.enter.prevent="handleSave"
      @keydown.escape.prevent="handleCancel"
    />
    <Button
      type="button"
      success
      soft
      join
      :disabled="disabled"
      aria-label="Save"
      @click.stop="handleSave"
    >
      <CheckIcon class="h-4 w-4" />
    </Button>
    <Button
      type="button"
      error
      soft
      join
      :disabled="disabled"
      aria-label="Cancel"
      @click.stop="handleCancel"
    >
      <XIcon class="h-4 w-4" />
    </Button>
  </Join>
</template>
