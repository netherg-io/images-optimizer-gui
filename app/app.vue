<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useModal } from 'vue-final-modal';
import { useOptimizationStore } from './stores/optimization';

useTheme();

const optStore = useOptimizationStore();

const { open, close, patchOptions } = useModal({
  component: resolveComponent('LazyModalsStatus'),
  attrs: {
    clickToClose: false,
    escToClose: false,
    total: 0,
    done: 0,
    currentFile: '',
    result: null,
    error: null,

    onConfirm: () => {
      close();
      optStore.resetState();
    },
  },
});

onMounted(async () => {
  try {
    await optStore.initListeners();

    if (optStore.isProcessing || optStore.result) {
      open({
        attrs: {
          total: optStore.progress.total,
          done: optStore.progress.done,
          currentFile: optStore.progress.currentFile,
          result: optStore.result,
          error: optStore.error,
        },
      });
    }

    await getCurrentWindow().show();
    await getCurrentWindow().setFocus();
  } catch (e) {
    console.error(e);
    await getCurrentWindow().show();
  }
});

watch(
  () => optStore.isProcessing,
  (isProcessing) => {
    if (isProcessing) {
      open({
        attrs: {
          total: optStore.progress.total || 0,
          done: 0,
          currentFile: 'Starting...',
          result: null,
          error: null,
        },
      });
    }
  },
);

watch(
  () => optStore.progress,
  (newProgress) => {
    if (optStore.isProcessing || optStore.result) {
      patchOptions({
        attrs: {
          total: newProgress.total,
          done: newProgress.done,
          currentFile: newProgress.currentFile,
        },
      });
    }
  },
  { deep: true },
);

watch(
  [() => optStore.result, () => optStore.error],
  ([newResult, newError]) => {
    patchOptions({
      attrs: {
        result: newResult,
        error: newError,
      },
    });
  },
  { deep: true },
);
</script>

<template>
  <CResize id="app" class="app">
    <NuxtLayout v-slot="{ className }" class="app__layout">
      <div :class="className">
        <NuxtPage class="app__page" :keepalive="false" />
      </div>
    </NuxtLayout>
  </CResize>
</template>

<style scoped lang="scss">
.app {
  $parent: &;

  display: flex;
  flex-direction: column;

  &:deep(#{$parent}__layout) {
    flex-grow: 1;
  }

  &:deep(#{$parent}__page) {
    flex-grow: 1;
    width: 100%;
  }
}
</style>
