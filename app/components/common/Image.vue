<script setup>
import { resolveUrl } from '@/utils/helpers';

const props = defineProps({
  src: {
    type: String,
    required: true,
  },
  srcset: {
    type: String,
    default: undefined,
  },
  alt: {
    type: String,
    default: '',
  },
  loading: {
    type: String,
    default: 'lazy',
    validator: (value) => ['lazy', 'eager'].includes(value),
  },
  fetchPriority: {
    type: String,
    default: undefined,
    validator: (value) => ['high', 'low', 'auto'].includes(value),
  },
});

const imageSrc = computed(() => {
  return resolveUrl(props.src);
});

const srcSet = computed(() => {
  if (!props.srcset) return undefined;

  return props.srcset
    .split(',')
    .map((entry) => {
      const trimmed = entry.trim();
      const spaceIndex = trimmed.lastIndexOf(' ');

      if (spaceIndex === -1) {
        return resolveUrl(trimmed);
      }

      const url = trimmed.slice(0, spaceIndex);
      const descriptor = trimmed.slice(spaceIndex + 1);

      return `${resolveUrl(url)} ${descriptor}`;
    })
    .join(', ');
});
</script>

<template>
  <img
    :src="imageSrc"
    :srcset="srcSet"
    :alt="alt"
    :loading="loading"
    :fetchpriority="fetchPriority"
  />
</template>
