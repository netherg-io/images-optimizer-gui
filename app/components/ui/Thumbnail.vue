<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  src: { type: String, required: true },
  alt: { type: String, default: '' },
  className: { type: String, default: '' },
});

const thumbnailSrc = ref('');
const root = ref(null);
const isLoaded = ref(false);
let observer = null;

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
  observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          generate();
          if (root.value) observer.unobserve(root.value);
        }
      });
    },
    {
      rootMargin: '100px',
    },
  );

  if (root.value) observer.observe(root.value);
});

onBeforeUnmount(() => {
  if (observer) observer.disconnect();
});

watch(
  () => props.src,
  () => {
    isLoaded.value = false;
    thumbnailSrc.value = '';
    if (root.value && observer) observer.observe(root.value);
  },
);
</script>

<template>
  <div ref="root" class="thumb-container" :class="className">
    <AFade>
      <img
        v-if="thumbnailSrc"
        :src="thumbnailSrc"
        :alt="alt"
        class="thumb-container__img"
      />
      <div v-else class="thumb-container__skeleton"></div>
    </AFade>
  </div>
</template>

<style lang="scss" scoped>
.thumb-container {
  position: relative;
  width: 64px;
  height: 64px;
  overflow: hidden;
  border-radius: 4px;

  &__img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    background-color: $background-color-secondary;
    transition: background-color $time-normal $ease;
  }

  &__skeleton {
    width: 100%;
    height: 100%;
    background-color: $background-color-tertiary;
    transition: background-color $time-normal $ease;
    animation: pulse 1.5s infinite;
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
