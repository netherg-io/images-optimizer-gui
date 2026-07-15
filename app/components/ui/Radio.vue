<script setup>
import { watch } from 'vue';

const props = defineProps({
  title: {
    type: String,
    default: '',
  },
  options: {
    type: Array,
    default: () => [],
  },
});

const model = defineModel({
  type: String,
  default: '',
});

const getValue = (option) => {
  return option.value !== undefined ? option.value : option;
};

const isSelected = (option) => {
  return model.value === getValue(option);
};

const selectOption = (option) => {
  model.value = getValue(option);
};

watch(
  () => props.options,
  (newOptions) => {
    if (newOptions?.length && !model.value) {
      model.value = getValue(newOptions[0]);
    }
  },
  { immediate: true },
);
</script>

<template>
  <div class="ui-radio">
    <div v-if="title" class="ui-radio__title">{{ title }}</div>

    <div
      v-if="options?.length"
      class="ui-radio__content"
      role="radiogroup"
      :aria-label="title"
    >
      <CardRadio
        v-for="(option, index) in options"
        :key="index"
        v-bind="option"
        :is-active="isSelected(option)"
        class="ui-radio__card"
        @click="selectOption(option)"
      />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.ui-radio {
  display: flex;
  flex-direction: column;
  gap: em(16);

  &__title {
    flex-shrink: 0;
  }

  &__content {
    display: flex;
    flex-shrink: 0;
    flex-direction: column;
    gap: em(8);
  }

  &__card {
    flex-shrink: 0;
  }
}
</style>
