<script setup>
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { downloadDir } from '@tauri-apps/api/path'; // Import needed for 'downloads' option

const filesStore = useFilesStore();
const { totalItems, sourcePaths } = storeToRefs(filesStore);

const quality = ref('80');
const optimization = ref(['compress']);
const path = ref(['same']);
const saveMethod = ref('rename');
const savePath = ref('');

await listen('progress', (event) => {
  const { total, done, current_file } = event.payload;
  const percentage = Math.round((done / total) * 100);
  console.log(`Processing: ${current_file} (${percentage}%)`);
});

async function startOptimization() {
  try {
    let outputDir = null;

    if (path.value.includes('downloads')) {
      outputDir = await downloadDir();
    } else if (path.value.includes('custom')) {
      outputDir = savePath.value;
    }

    const config = {
      paths: sourcePaths.value,
      jpg_q: Math.max(10, parseInt(quality.value)),
      png_max: Math.max(10, parseInt(quality.value)),
      png_min: Math.max(10, parseInt(quality.value) - 15),

      webp: optimization.value.includes('webp'),
      avif: optimization.value.includes('avif'),

      replace: path.value.includes('same') && saveMethod.value === 'overwrite',

      output_dir: outputDir,
    };

    console.log('Sending config:', config);

    const result = await invoke('run_optimization', { config });

    console.log('Finished!', result);
  } catch (error) {
    console.error('Optimization failed:', error);
  }
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
          @click="startOptimization"
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
