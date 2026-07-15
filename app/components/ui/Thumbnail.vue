<script setup>
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  src: { type: String, required: true },
  alt: { type: String, default: '' },
  className: { type: String, default: '' },
});

const thumbnailSrc = ref('');
const isLoaded = ref(false);

const generate = async () => {
  if (isLoaded.value || !props.src) return;

  try {
    const b64 = await invoke('generate_thumbnail', { path: props.src });
    thumbnailSrc.value = b64;
    isLoaded.value = true;
  } catch (e) {
    console.error(`Failed to load ${props.src}`, e);
  }
};

onMounted(() => {
  generate();
});

watch(
  () => props.src,
  () => {
    isLoaded.value = false;
    thumbnailSrc.value = '';
    generate();
  },
);

const openOriginal = async () => {
  if (!props.src) return;
  try {
    await invoke('open_local_path', { path: props.src });
  } catch (err) {
    console.error('Failed to open image:', err);
  }
};
</script>

<template>
  <button
    type="button"
    class="thumb-container"
    :class="className"
    :aria-label="alt || $t('common.open-image')"
    @click.stop="openOriginal"
  >
    <ACrossFade>
      <img
        v-if="thumbnailSrc"
        :src="thumbnailSrc"
        :alt="alt"
        class="thumb-container__img"
      />
      <div v-else>
        <div class="thumb-container__skeleton"></div>
      </div>
    </ACrossFade>

    <div class="thumb-container__overlay">
      <CIcon name="eye" class="thumb-container__icon" aria-hidden="true" />
    </div>
  </button>
</template>

<style lang="scss" scoped>
.thumb-container {
  position: relative;
  width: 64px;
  height: 64px;
  overflow: hidden;
  border-radius: 4px;

  &__icon {
    width: em(30);
    height: em(30);
    color: $color-white;
  }

  &__img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    background-color: $background-color-secondary;
    transition: $time-normal $ease;
    transition-property: background-color, opacity;
  }

  &__skeleton {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    background-color: $background-color-tertiary;
    transition: $time-normal $ease;
    transition-property: background-color, opacity;
    animation: pulse 1.5s infinite;
  }

  &__overlay {
    position: absolute;
    inset: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    background: rgba($color-black, 0.5);
    opacity: 0;
    transition: opacity $time-normal $ease;

    @include hover {
      opacity: 1;
    }
  }
}

@keyframes pulse {
  0% {
    opacity: 0.5;
  }

  50% {
    opacity: 1;
  }

  100% {
    opacity: 0.5;
  }
}
</style>
