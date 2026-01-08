import { defineStore, storeToRefs } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { downloadDir } from '@tauri-apps/api/path';
import { useFilesStore } from './files';

export const useOptimizationStore = defineStore('optimization', () => {
  const filesStore = useFilesStore();
  const { fileTasks } = storeToRefs(filesStore);

  const quality = ref('80');
  const optimization = ref(['compress']);
  const path = ref(['same']);
  const saveMethod = ref('rename');
  const savePath = ref('');

  const isProcessing = ref(false);
  const progress = ref({
    total: 0,
    done: 0,
    currentFile: '',
    percentage: 0,
  });
  const result = ref(null);
  const error = ref(null);

  async function initListeners() {
    const processingState = await invoke('get_processing_state');
    isProcessing.value = processingState;

    if (!processingState) {
      const hasUnviewed =
        localStorage.getItem('has_unviewed_result') === 'true';
      if (hasUnviewed) {
        await fetchLastResult();
      }
    }

    await listen('processing_state_change', async (event) => {
      isProcessing.value = event.payload;

      if (event.payload === true) {
        result.value = null;
        error.value = null;
        progress.value = {
          total: 0,
          done: 0,
          currentFile: 'Starting...',
          percentage: 0,
        };
      } else {
        if (!result.value && !error.value) {
          await fetchLastResult();
        }
        if (result.value) {
          localStorage.setItem('has_unviewed_result', 'true');
        }
      }
    });

    await listen('progress', (event) => {
      const { total, done, current_file } = event.payload;
      progress.value = {
        total,
        done,
        currentFile: current_file,
        percentage: total > 0 ? Math.round((done / total) * 100) : 0,
      };
    });
  }

  async function fetchLastResult() {
    if (isProcessing.value) return;

    try {
      const lastResult = await invoke('get_last_result');
      if (lastResult) {
        result.value = lastResult;
        progress.value = {
          total: lastResult.total_files,
          done: lastResult.total_files,
          currentFile: 'Completed',
          percentage: 100,
        };
      }
    } catch (e) {
      console.error('Failed to fetch last result', e);
    }
  }

  async function startOptimization() {
    resetState();

    isProcessing.value = true;

    try {
      let outputDir = null;
      if (path.value.includes('downloads')) {
        outputDir = await downloadDir();
      } else if (path.value.includes('custom')) {
        outputDir = savePath.value;
      }

      const config = {
        tasks: fileTasks.value,
        jpg_q: Math.max(10, parseInt(quality.value)),
        png_max: Math.max(10, parseInt(quality.value)),
        png_min: Math.max(10, parseInt(quality.value) - 15),
        webp: optimization.value.includes('webp'),
        avif: optimization.value.includes('avif'),
        optimize_original: optimization.value.includes('compress'),
        replace:
          path.value.includes('same') && saveMethod.value === 'overwrite',
        output_dir: outputDir,
      };

      const res = await invoke('run_optimization', { config });
      result.value = res;
      filesStore.clearAll();
      localStorage.setItem('has_unviewed_result', 'true');
    } catch (err) {
      console.error('Optimization failed:', err);
      error.value = err;
    } finally {
      isProcessing.value = false;
    }
  }

  async function cancelOptimization() {
    try {
      await invoke('cancel_optimization');
    } catch (e) {
      console.error('Failed to send cancel command', e);
    }
  }

  function resetState() {
    localStorage.setItem('has_unviewed_result', 'false');

    result.value = null;
    error.value = null;
    progress.value = {
      total: 0,
      done: 0,
      currentFile: '',
      percentage: 0,
    };
  }

  return {
    quality,
    optimization,
    path,
    saveMethod,
    savePath,
    isProcessing,
    progress,
    result,
    error,
    initListeners,
    startOptimization,
    cancelOptimization,
    resetState,
  };
});
