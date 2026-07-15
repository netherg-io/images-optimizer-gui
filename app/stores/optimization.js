import { defineStore, storeToRefs } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { downloadDir } from '@tauri-apps/api/path';
import { confirm } from '@tauri-apps/plugin-dialog';
import { useFilesStore } from './files';

export const useOptimizationStore = defineStore('optimization', () => {
  const filesStore = useFilesStore();
  const { t } = useI18n();
  const { fileTasks } = storeToRefs(filesStore);

  const quality = ref(80);
  const optimization = ref(['compress']);
  const path = ref(['same']);
  const saveMethod = ref('rename');
  const existingFilePolicy = ref('rename');
  const savePath = ref('');
  const isProcessing = ref(false);
  const isCancelRequested = ref(false);
  const progress = ref(emptyProgress());
  const result = ref(null);
  const error = ref(null);
  const unlisteners = [];
  let listenersInitialized = false;

  const canStart = computed(
    () =>
      !isProcessing.value &&
      fileTasks.value.length > 0 &&
      optimization.value.length > 0 &&
      (!path.value.includes('custom') || !!savePath.value.trim()),
  );

  async function initListeners() {
    if (listenersInitialized) return;

    isProcessing.value = await invoke('get_processing_state');
    if (!isProcessing.value) {
      const hasUnviewed =
        localStorage.getItem('has_unviewed_result') === 'true';
      if (hasUnviewed) await fetchLastResult();
    }

    unlisteners.push(
      await listen('processing_state_change', async ({ payload }) => {
        isProcessing.value = payload;
        if (payload) {
          result.value = null;
          error.value = null;
          isCancelRequested.value = false;
          progress.value = { ...emptyProgress(), currentFile: 'Starting...' };
        } else {
          if (!result.value && !error.value) await fetchLastResult();
          if (result.value) {
            localStorage.setItem('has_unviewed_result', 'true');
          }
        }
      }),
    );
    unlisteners.push(
      await listen('progress', ({ payload }) => {
        const { total, done, current_file: currentFile } = payload;
        const percentage = total > 0 ? Math.round((done / total) * 100) : 0;
        progress.value = {
          total,
          done,
          currentFile,
          percentage: Math.min(isCancelRequested.value ? 99 : 100, percentage),
        };
      }),
    );
    listenersInitialized = true;
  }

  function disposeListeners() {
    unlisteners.splice(0).forEach((unlisten) => unlisten());
    listenersInitialized = false;
  }

  async function fetchLastResult() {
    if (isProcessing.value) return;
    const lastResult = await invoke('get_last_result');
    if (!lastResult) return;

    result.value = lastResult;
    const canceledOperations = lastResult.operations.filter(
      ({ status }) => status === 'canceled',
    ).length;
    const done = lastResult.total_operations - canceledOperations;
    progress.value = {
      total: lastResult.total_operations,
      done,
      currentFile: lastResult.is_canceled ? 'Canceled' : 'Completed',
      percentage: lastResult.total_operations
        ? Math.min(100, Math.round((done / lastResult.total_operations) * 100))
        : 0,
    };
  }

  async function startOptimization() {
    if (!canStart.value) return;
    resetState();

    const replacesFiles =
      (path.value.includes('same') && saveMethod.value === 'overwrite') ||
      existingFilePolicy.value === 'overwrite';
    const overwriteConfirmed =
      !replacesFiles ||
      (await confirm(t('sections.options.overwrite-confirm.message'), {
        title: t('sections.options.overwrite-confirm.title'),
        kind: 'warning',
      }));
    if (!overwriteConfirmed) return;

    isProcessing.value = true;
    try {
      let outputDir = null;
      if (path.value.includes('downloads')) outputDir = await downloadDir();
      if (path.value.includes('custom')) outputDir = savePath.value;

      const numericQuality = Number(quality.value);
      const config = {
        tasks: fileTasks.value,
        jpg_q: numericQuality,
        png_min: Math.max(0, numericQuality - 15),
        png_max: numericQuality,
        webp: optimization.value.includes('webp'),
        avif: optimization.value.includes('avif'),
        optimize_original: optimization.value.includes('compress'),
        replace:
          path.value.includes('same') && saveMethod.value === 'overwrite',
        output_dir: outputDir,
        existing_file_policy: existingFilePolicy.value,
        overwrite_confirmed: overwriteConfirmed,
        webp_quality: numericQuality,
        avif_quality: numericQuality,
        avif_speed: 4,
        skip_if_larger: true,
      };

      result.value = await invoke('run_optimization', { config });
      if (
        result.value.succeeded_files === result.value.total_files &&
        result.value.skipped_files === 0
      ) {
        filesStore.clearAll();
      }
      localStorage.setItem('has_unviewed_result', 'true');
    } catch (reason) {
      error.value =
        typeof reason === 'string' ? reason : reason?.message || String(reason);
    } finally {
      isProcessing.value = false;
    }
  }

  async function cancelOptimization() {
    if (!isProcessing.value || isCancelRequested.value) return;
    isCancelRequested.value = true;
    await invoke('cancel_optimization');
  }

  function resetState() {
    localStorage.setItem('has_unviewed_result', 'false');
    result.value = null;
    error.value = null;
    isCancelRequested.value = false;
    progress.value = emptyProgress();
  }

  return {
    quality,
    optimization,
    path,
    saveMethod,
    existingFilePolicy,
    savePath,
    isProcessing,
    isCancelRequested,
    canStart,
    progress,
    result,
    error,
    initListeners,
    disposeListeners,
    startOptimization,
    cancelOptimization,
    resetState,
  };
});

function emptyProgress() {
  return { total: 0, done: 0, currentFile: '', percentage: 0 };
}
