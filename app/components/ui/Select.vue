<script setup>
const props = defineProps({
  title: {
    type: String,
    default: '',
  },
  options: {
    type: Array,
    default: () => [],
  },
  multi: {
    type: Boolean,
    default: false,
  },
});

const model = defineModel({
  type: Array,
  default: () => [],
});

const getValue = (option) => {
  return option.value !== undefined ? option.value : option;
};

const isSelected = (option) => {
  if (!model.value) return false;
  return model.value.includes(getValue(option));
};

const toggleOption = (option) => {
  const value = getValue(option);
  const index = model.value.indexOf(value);
  const isAlreadySelected = index !== -1;

  if (props.multi) {
    const newArray = [...model.value];
    if (isAlreadySelected) {
      if (newArray.length > 1) {
        newArray.splice(index, 1);
      }
    } else {
      newArray.push(value);
    }
    model.value = newArray;
  } else {
    if (!isAlreadySelected) {
      model.value = [value];
    }
  }
};
</script>

<template>
  <div class="ui-select">
    <div v-if="title" class="ui-select__title">{{ title }}</div>

    <div
      v-if="options?.length"
      class="ui-select__content"
      role="group"
      :aria-label="title"
    >
      <CardSelect
        v-for="(option, index) in options"
        :key="index"
        v-bind="option"
        :is-active="isSelected(option)"
        class="ui-select__card"
        @click="toggleOption(option)"
      />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.ui-select {
  display: flex;
  flex-direction: column;
  gap: em(16);

  &__content {
    display: flex;
    gap: em(12);
  }

  &__card {
    flex: 1;
  }
}
</style>
