import { defineStore } from 'pinia';
import { processPaths } from '@/utils/fileScanner';

export const useFilesStore = defineStore('files', () => {
  const items = ref([]);
  const scanError = ref('');
  const isScanning = ref(false);
  const scanCount = ref(0);

  const totalSize = computed(() => {
    return items.value.reduce((acc, item) => acc + item.size, 0);
  });

  const totalItems = computed(() => {
    return items.value.reduce((acc, item) => acc + (item.fileCount || 0), 0);
  });

  const fileTasks = computed(() => {
    const tasks = [];

    const traverse = (node, rootPath) => {
      const currentRoot = rootPath || node.path;

      if (node.type === 'file') {
        tasks.push({
          path: node.path,
          root: currentRoot,
        });
      } else if (node.children && node.children.length > 0) {
        node.children.forEach((child) => traverse(child, currentRoot));
      }
    };

    items.value.forEach((item) => traverse(item, null));
    return tasks;
  });

  const addItemsFromPaths = async (paths) => {
    if (isScanning.value) return;
    scanError.value = '';
    scanCount.value = 0;
    isScanning.value = true;
    try {
      const newTrees = await processPaths(
        paths,
        (count) => (scanCount.value = count),
      );
      newTrees.forEach((newRoot) => {
        if (!items.value.some((i) => i.path === newRoot.path)) {
          items.value.push(newRoot);
        }
      });
    } catch (error) {
      scanError.value =
        typeof error === 'string' ? error : error?.message || String(error);
    } finally {
      isScanning.value = false;
    }
  };

  const recalculateFolder = (folder) => {
    let newSize = 0;
    let newCount = 0;

    folder.children.forEach((child) => {
      newSize += child.size;
      newCount += child.type === 'folder' ? child.fileCount || 0 : 1;
    });

    folder.size = newSize;
    folder.fileCount = newCount;
  };

  const findAndRemove = (list, targetId) => {
    const index = list.findIndex((i) => i.id === targetId);

    if (index !== -1) {
      list.splice(index, 1);
      return true;
    }

    for (let i = 0; i < list.length; i++) {
      const item = list[i];

      if (item.type === 'folder' && item.children) {
        const wasRemoved = findAndRemove(item.children, targetId);

        if (wasRemoved) {
          if (item.children.length === 0) {
            list.splice(i, 1);
          } else {
            recalculateFolder(item);
          }
          return true;
        }
      }
    }

    return false;
  };

  const removeById = (id) => {
    findAndRemove(items.value, id);
  };

  const clearAll = () => {
    items.value = [];
  };

  return {
    items,
    scanError,
    isScanning,
    scanCount,
    totalSize,
    totalItems,
    fileTasks,
    addItemsFromPaths,
    removeById,
    clearAll,
  };
});
