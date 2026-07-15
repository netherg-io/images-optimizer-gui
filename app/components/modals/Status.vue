<script setup>
import { invoke } from '@tauri-apps/api/core';
import { VueFinalModal } from 'vue-final-modal';
import { useOptimizationStore } from '@/stores/optimization';
import { formatSize, formatTime, getParentPath } from '@/utils/helpers';

const props = defineProps({
  total: { type: Number, default: 0 },
  done: { type: Number, default: 0 },
  currentFile: { type: String, default: '' },
  result: { type: Object, default: null },
  error: { type: [String, Object], default: null },
});

const emit = defineEmits(['confirm', 'cancel', 'update:modelValue']);
const optStore = useOptimizationStore();
const { t } = useI18n();

const percentage = computed(() =>
  props.total > 0
    ? Math.max(0, Math.min(100, Math.round((props.done / props.total) * 100)))
    : 0,
);
const isFinished = computed(() => !!props.result);
const isCanceled = computed(() => !!props.result?.is_canceled);
const isPartial = computed(() => (props.result?.failed_files || 0) > 0);
const normalizedError = computed(() =>
  typeof props.error === 'string'
    ? props.error
    : props.error?.message || String(props.error || ''),
);
const failures = computed(
  () =>
    props.result?.operations?.filter(({ status }) => status === 'failed') || [],
);
const successfulOperations = computed(
  () =>
    props.result?.operations?.filter(
      ({ status, output_size: outputSize }) =>
        status === 'succeeded' && Number.isFinite(outputSize),
    ) || [],
);
const destinationRoot = computed(
  () =>
    props.result?.destination_root ||
    getParentPath(
      successfulOperations.value.find((operation) => operation.destination)
        ?.destination,
    ),
);

const savingsPercent = computed(() => {
  const original = successfulOperations.value.reduce(
    (sum, operation) => sum + operation.original_size,
    0,
  );
  const output = successfulOperations.value.reduce(
    (sum, operation) => sum + operation.output_size,
    0,
  );
  if (!original) return 0;
  return Math.max(
    0,
    Math.min(100, ((original - output) / original) * 100),
  ).toFixed(1);
});

const formatRows = computed(() => {
  const labels = {
    optimize_original: t('modals.status.table.label.optimized'),
    webp: t('modals.status.table.label.webp'),
    avif: t('modals.status.table.label.avif'),
  };
  return Object.entries(labels).flatMap(([operation, label]) => {
    const results = successfulOperations.value.filter(
      (result) => result.operation === operation,
    );
    if (!results.length) return [];
    const original = results.reduce(
      (sum, result) => sum + result.original_size,
      0,
    );
    const output = results.reduce((sum, result) => sum + result.output_size, 0);
    const saved = Math.max(
      0,
      Math.min(100, ((original - output) / original) * 100),
    );
    return [{ operation, label, output, saved: saved.toFixed(1) }];
  });
});

const title = computed(() => {
  if (props.error) return t('modals.status.title.error');
  if (isCanceled.value) return t('modals.status.title.canceled');
  if (isPartial.value) return t('modals.status.title.partial');
  if (isFinished.value) return t('modals.status.title.done');
  if (optStore.isCancelRequested) return t('modals.status.title.canceling');
  return t('modals.status.title.optimizing');
});

async function openDestination() {
  if (destinationRoot.value) {
    await invoke('open_local_path', { path: destinationRoot.value });
  }
}

async function copyDiagnostics() {
  const diagnostics = {
    operationId: props.result?.operation_id,
    totalFiles: props.result?.total_files,
    succeeded: props.result?.succeeded_files,
    failed: props.result?.failed_files,
    skipped: props.result?.skipped_files,
    canceled: props.result?.canceled_files,
    failures: failures.value.map((failure) => ({
      file: failure.source.split(/[\\/]/).pop(),
      operation: failure.operation,
      code: failure.error_code,
      message: failure.error_message,
    })),
  };
  await navigator.clipboard.writeText(JSON.stringify(diagnostics, null, 2));
}
</script>

<template>
  <VueFinalModal
    class="status-modal"
    content-class="status-modal__content"
    overlay-class="status-modal__overlay"
    overlay-transition="vfm-fade"
    content-transition="vfm-fade"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <h2 class="status-modal__title">{{ title }}</h2>

    <div v-if="error" class="status-modal__state" role="alert">
      <div
        class="status-modal__icon status-modal__icon--error"
        aria-hidden="true"
      >
        !
      </div>
      <p class="status-modal__message">{{ normalizedError }}</p>
    </div>

    <div v-else-if="!isFinished" class="status-modal__state">
      <div
        class="status-modal__progress-circle"
        :style="{ '--p': percentage }"
        role="progressbar"
        aria-valuemin="0"
        aria-valuemax="100"
        :aria-valuenow="percentage"
      >
        <span class="status-modal__progress-text">{{ percentage }}%</span>
      </div>
      <p class="status-modal__filename">{{ currentFile }}</p>
      <p>{{ done }} / {{ total }}</p>
      <UiButton
        size="sm"
        :title="$t('modals.status.button.cancel')"
        theme="warn"
        :disabled="optStore.isCancelRequested"
        @click="optStore.cancelOptimization()"
      />
    </div>

    <div v-else class="status-modal__state">
      <div
        class="status-modal__icon"
        :class="
          isPartial
            ? 'status-modal__icon--error'
            : 'status-modal__icon--success'
        "
        aria-hidden="true"
      >
        {{ isPartial ? '!' : isCanceled ? '■' : '✓' }}
      </div>

      <div class="status-modal__grid">
        <div class="status-modal__stat">
          <span>{{ $t('modals.status.stats.0') }}</span>
          <strong
            >{{ result.succeeded_files }} / {{ result.total_files }}</strong
          >
        </div>
        <div class="status-modal__stat">
          <span>{{ $t('modals.status.stats.1') }}</span>
          <strong>{{ formatTime(result.duration_total) }}</strong>
        </div>
        <div class="status-modal__stat status-modal__stat--highlight">
          <span>{{ $t('modals.status.stats.2') }}</span>
          <strong>{{ savingsPercent }}%</strong>
        </div>
      </div>

      <table v-if="formatRows.length" class="status-modal__table">
        <thead>
          <tr>
            <th>{{ $t('modals.status.table.head.0') }}</th>
            <th>{{ $t('modals.status.table.head.1') }}</th>
            <th>{{ $t('modals.status.table.head.2') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in formatRows" :key="row.operation">
            <th scope="row">{{ row.label }}</th>
            <td>{{ formatSize(row.output) }}</td>
            <td>{{ row.saved }}%</td>
          </tr>
        </tbody>
      </table>

      <details
        v-if="failures.length"
        class="status-modal__failures"
        role="alert"
      >
        <summary>
          {{ $t('modals.status.failures', { count: failures.length }) }}
        </summary>
        <ul>
          <li
            v-for="failure in failures"
            :key="`${failure.source}-${failure.operation}`"
          >
            <strong>{{ failure.source.split(/[\\/]/).pop() }}</strong>
            — {{ failure.error_message }}
          </li>
        </ul>
      </details>

      <div class="status-modal__actions">
        <UiButton
          :title="$t('modals.status.button.copy-diagnostics')"
          @click="copyDiagnostics"
        />
        <UiButton
          v-if="destinationRoot"
          :title="$t('modals.status.button.open-folder')"
          @click="openDestination"
        />
        <UiButton
          :title="$t('modals.status.button.close')"
          theme="accent"
          @click="emit('confirm')"
        />
      </div>
    </div>
  </VueFinalModal>
</template>

<style lang="scss">
@property --percentage {
  initial-value: 0%;
  inherits: true;
  syntax: '<percentage>';
}

.status-modal {
  display: flex;
  align-items: center;
  justify-content: center;

  &__icon {
    display: grid;
    place-items: center;
    width: em(56);
    height: em(56);
    border-radius: 50%;

    &--success {
      color: $accent-color-success;
      background: rgb(76 175 80 / 10%);
      border: 1px solid $accent-color-success;
    }

    &--error {
      color: $text-color-warn;
      background: rgb(244 67 54 / 10%);
      border: 1px solid $text-color-warn;
    }
  }

  &__content {
    display: flex;
    flex-direction: column;
    gap: em(20);
    width: min(90%, em(520));
    max-height: 90vh;
    padding: em(24);
    overflow-y: auto;
    color: $text-color-primary;
    background: $background-color-primary;
    border: 1px solid $border-color-secondary;
    border-radius: em(10);
    box-shadow: 0 em(10) em(40) rgb(0 0 0 / 20%);
  }

  &__title {
    margin: 0;
    text-align: center;
  }

  &__state {
    display: flex;
    flex-direction: column;
    gap: em(16);
    align-items: center;
    width: 100%;
  }

  &__message,
  &__failures {
    width: 100%;
    padding: em(12);
    background: $background-color-secondary;
    border-radius: em(8);
  }

  &__progress-circle {
    --percentage: calc(var(--p) * 1%);

    position: relative;
    display: grid;
    place-items: center;
    width: em(80);
    height: em(80);
    background: conic-gradient(
      $accent-color-secondary var(--percentage),
      $background-color-tertiary 0
    );
    border-radius: 50%;
    transition: --percentage $time-fast $ease;

    &::before {
      position: absolute;
      inset: em(8);
      content: '';
      background: $background-color-primary;
      border-radius: 50%;
    }
  }

  &__progress-text {
    position: relative;
    color: $accent-color-secondary;
  }

  &__filename {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: em(8);
    width: 100%;
  }

  &__stat {
    display: flex;
    flex-direction: column;
    gap: em(4);
    align-items: center;
    padding: em(10) em(4);
    text-align: center;
    background: $background-color-secondary;
    border-radius: em(8);

    &--highlight {
      color: $accent-color-success;
    }
  }

  &__table {
    width: 100%;
    overflow: hidden;
    border-collapse: collapse;
    border: 1px solid $border-color-secondary;

    th,
    td {
      padding: em(8) em(12);
      text-align: left;
      border-bottom: 1px solid $border-color-secondary;
    }
  }

  &__failures {
    max-height: em(180);
    overflow: auto;

    ul {
      padding-left: em(20);
    }
  }

  &__actions {
    display: flex;
    flex-wrap: wrap;
    gap: em(8);
    justify-content: center;
  }
}
</style>
