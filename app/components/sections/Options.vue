<script setup>
import { storeToRefs } from 'pinia';

const filesStore = useFilesStore();
const { totalItems } = storeToRefs(filesStore);

const optStore = useOptimizationStore();
const { quality, optimization, path, saveMethod, savePath, isProcessing } =
  storeToRefs(optStore);

function handleStart() {
  optStore.startOptimization();
}
</script>

<template>
  <div class="options-block">
    <div class="container">
      <ASmoothList tag="div" class="options-block__container">
        <UiSelect
          v-model="optimization"
          multi
          :title="$t('sections.options.select.0.title')"
          :options="[
            {
              value: 'compress',
              icon: 'lightning',
              title: $t('sections.options.select.0.options.0.title'),
              description: $t(
                'sections.options.select.0.options.0.description',
              ),
            },
            {
              value: 'webp',
              icon: 'image',
              title: $t('sections.options.select.0.options.1.title'),
              description: $t(
                'sections.options.select.0.options.1.description',
              ),
            },
            {
              value: 'avif',
              icon: 'sparkles',
              title: $t('sections.options.select.0.options.2.title'),
              description: $t(
                'sections.options.select.0.options.2.description',
              ),
            },
          ]"
        />

        <UiButton
          :title="`${$t('sections.options.button')} ${$t('common.plurals.images', { count: totalItems })}`"
          theme="accent"
          :disabled="isProcessing"
          @click="handleStart"
        />

        <div class="options-block__divider"></div>

        <UiRange
          v-model="quality"
          :title="$t('sections.options.range.0.title')"
        />

        <div class="options-block__divider"></div>

        <UiSelect
          v-model="path"
          :title="$t('sections.options.select.1.title')"
          :options="[
            {
              value: 'same',
              icon: 'folder-opened',
              title: $t('sections.options.select.1.options.0.title'),
              description: $t(
                'sections.options.select.1.options.0.description',
              ),
            },
            {
              value: 'downloads',
              icon: 'download',
              title: $t('sections.options.select.1.options.1.title'),
              description: $t(
                'sections.options.select.1.options.1.description',
              ),
            },
            {
              value: 'custom',
              icon: 'folder-custom',
              title: $t('sections.options.select.1.options.2.title'),
              description: $t(
                'sections.options.select.1.options.2.description',
              ),
            },
          ]"
        />

        <UiRadio
          v-if="optimization.includes('compress') && path.includes('same')"
          v-model="saveMethod"
          :title="$t('sections.options.radio.0.title')"
          :options="[
            {
              value: 'rename',
              title: $t('sections.options.radio.0.options.0.title'),
              description: $t('sections.options.radio.0.options.0.description'),
            },
            {
              value: 'overwrite',
              title: $t('sections.options.radio.0.options.1.title'),
              description: $t('sections.options.radio.0.options.1.description'),
            },
          ]"
        />

        <UiFolderPicker
          v-if="path.includes('custom')"
          v-model="savePath"
          :title="$t('sections.options.folder-picker.title')"
        />
      </ASmoothList>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.options-block {
  &__container {
    display: flex;
    flex-direction: column;
    gap: em(24);
    padding: em(24);
    background-color: $background-color-primary;
    border: 1px solid $border-color-secondary;
    border-radius: em(16);
    transition: $time-normal $ease;
    transition-property: background-color, border-color;
  }

  &__divider {
    border-top: 1px solid $border-color-secondary;
    transition: border-color $time-normal $ease;
  }
}
</style>
