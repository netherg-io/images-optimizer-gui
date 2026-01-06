import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { processPaths } from '@/utils/fileScanner';

export const useFilesStore = defineStore('files', () => {
  const items = ref([]);

  const totalSize = computed(() => {
    return items.value.reduce((acc, item) => acc + item.size, 0);
  });

  const addItemsFromPaths = async (paths) => {
    const processedItems = await processPaths(paths);

    processedItems.forEach((newItem) => {
      const exists = items.value.some(
        (existing) => existing.path === newItem.path,
      );
      if (!exists) {
        items.value.push(newItem);
      }
    });
  };

  const removeFile = (index) => {
    items.value.splice(index, 1);
  };

  watch(
    () => items.value,
    (f) => console.log(f),
    { deep: true },
  );

  return {
    items,
    totalSize,
    addItemsFromPaths,
    removeFile,
  };
});
