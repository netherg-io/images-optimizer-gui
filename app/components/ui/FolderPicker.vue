<script setup>
import { open } from '@tauri-apps/plugin-dialog';

defineProps({
  title: {
    type: String,
    default: '',
  },
});

const model = defineModel({
  type: String,
  default: '',
});

async function handleBrowse() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: model.value || undefined,
    });

    if (typeof selected === 'string') {
      model.value = selected;
    }
  } catch (err) {
    console.error('Failed to open folder browser:', err);
  }
}
</script>

<template>
  <div class="ui-folder-picker">
    <div v-if="title" class="ui-folder-picker__title">
      <p class="s1-r">{{ title }}</p>
    </div>

    <div class="ui-folder-picker__content">
      <input
        v-model="model"
        type="text"
        disabled
        :placeholder="$t('sections.options.folder-picker.placeholder')"
        class="ui-folder-picker__input"
      />

      <UiButton
        icon="folder-opened"
        theme="browse"
        :title="$t('sections.options.folder-picker.button')"
        @click="handleBrowse"
      />
    </div>
  </div>
</template>

<style lang="scss" scoped>
$c-track: #e2e8f0;

.ui-folder-picker {
  display: flex;
  flex-direction: column;
  gap: em(24);

  &__title {
    flex-shrink: 0;
  }

  &__content {
    display: flex;
    flex-shrink: 0;
    gap: em(8);
  }

  &__input {
    flex: 1;
    padding: em(12);
    background-color: $background-color-tertiary;
    border: 1px solid $border-color-accent;
    border-radius: em(10);
    transition: $time-normal $ease;
    transition-property: border-color background-color;

    &::placeholder {
      color: $text-color-placeholder;
      transition: color $time-normal $ease;
    }
  }
}
</style>
