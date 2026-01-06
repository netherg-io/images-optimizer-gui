<script setup>
import { useFilesStore } from '@/stores/files';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';

const filesStore = useFilesStore();
const isDragging = ref(false);

const triggerFileSelect = async () => {
  try {
    const selected = await open({
      multiple: true,
      directory: false,
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      await filesStore.addItemsFromPaths(paths);
    }
  } catch (err) {
    console.error('Failed to open file dialog', err);
  }
};

const triggerFolderSelect = async () => {
  try {
    const selected = await open({
      multiple: true,
      directory: true,
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      await filesStore.addItemsFromPaths(paths);
    }
  } catch (err) {
    console.error('Failed to open folder dialog', err);
  }
};

let unlistenDrop = null;
let unlistenHover = null;
let unlistenLeave = null;

onMounted(async () => {
  unlistenDrop = await listen('tauri://drag-drop', (event) => {
    isDragging.value = false;
    if (event.payload.paths && event.payload.paths.length) {
      filesStore.addItemsFromPaths(event.payload.paths);
    }
  });

  unlistenHover = await listen(
    'tauri://drag-enter',
    () => (isDragging.value = true),
  );
  unlistenLeave = await listen(
    'tauri://drag-leave',
    () => (isDragging.value = false),
  );
});

onUnmounted(() => {
  if (unlistenDrop) unlistenDrop();
  if (unlistenHover) unlistenHover();
  if (unlistenLeave) unlistenLeave();
});
</script>

<template>
  <div class="add-files-block">
    <div class="add-files-block__container container">
      <UiFileInput class="add-files-block__input" :is-active="isDragging" />

      <div class="add-files-block__buttons">
        <UiButton
          class="add-files-block__button"
          title="sections.add-files.buttons.0"
          icon="file-add"
          @click="triggerFileSelect"
        />

        <UiButton
          class="add-files-block__button"
          title="sections.add-files.buttons.1"
          icon="folder-add"
          @click="triggerFolderSelect"
        />
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.add-files-block {
  &__container {
    display: flex;
    flex-direction: column;
    gap: em(34);
  }

  &__input {
    height: em(288);
  }

  &__buttons {
    display: flex;
    gap: em(16);
  }

  &__button {
    flex: 1;
  }
}
</style>
