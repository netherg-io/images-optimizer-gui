<script setup>
import { VueFinalModal } from 'vue-final-modal';
import { useOptimizationStore } from '@/stores/optimization';
import { formatSize, formatTime } from '@/utils/helpers';

const props = defineProps({
  total: {
    type: Number,
    default: 0,
  },
  done: {
    type: Number,
    default: 0,
  },
  currentFile: {
    type: String,
    default: '',
  },
  result: {
    type: Object,
    default: null,
  },
  error: {
    type: [String, Object],
    default: null,
  },
});

const { t } = useI18n();

const emit = defineEmits(['confirm', 'cancel', 'update:modelValue']);
const optStore = useOptimizationStore();

const percentage = computed(() => {
  if (!props.total || props.total === 0) return 0;
  return Math.round((props.done / props.total) * 100);
});

const isFinished = computed(() => !!props.result);
const hasError = computed(() => !!props.error);

const isCanceled = computed(() => props.result && props.result.is_canceled);

const savingsPercent = computed(() => {
  if (!props.result || props.result.total_size_original === 0) return 0;
  const saved = Math.min(
    props.result.total_size_optimized || props.result.total_size_original,
    props.result.total_size_avif || props.result.total_size_original,
    props.result.total_size_webp || props.result.total_size_original,
  );

  console.log(
    props.result.total_size_optimized || props.result.total_size_original,
    props.result.total_size_avif || props.result.total_size_original,
    props.result.total_size_webp || props.result.total_size_original,
  );

  return 100 - ((saved / props.result.total_size_original) * 100).toFixed(1);
});

const filesDisplay = computed(() => {
  if (!props.result) return '';
  if (isCanceled.value) {
    return `${props.result.processed_files} / ${props.result.total_files}`;
  }
  return props.result.total_files;
});

function handleCancel() {
  optStore.cancelOptimization();
}

const header = computed(() => [
  {
    title: t('modals.status.title.done'),
    modifier: 'success',
    condition: isFinished.value && !hasError.value && !isCanceled.value,
  },
  {
    title: t('modals.status.title.canceled'),
    modifier: 'canceled',
    condition: isCanceled.value,
  },
  {
    title: t('modals.status.title.error'),
    modifier: 'error',
    condition: hasError.value,
  },
  {
    title: t('modals.status.title.optimizing'),
    condition: !isFinished.value && !hasError.value && !isCanceled.value,
  },
]);

const table = computed(() => [
  {
    label: t('modals.status.table.label.original'),
    rawValue: props.result.total_size_original,
    value: formatSize(props.result.total_size_original),
    condition: true,
  },
  {
    label: t('modals.status.table.label.optimized'),
    rawValue: props.result.total_size_optimized,
    value: formatSize(props.result.total_size_optimized),
    condition: props.result.total_size_optimized > 0,
  },
  {
    label: t('modals.status.table.label.webp'),
    rawValue: props.result.total_size_webp,
    value: formatSize(props.result.total_size_webp),
    condition: props.result.total_size_webp > 0,
  },
  {
    label: t('modals.status.table.label.avif'),
    rawValue: props.result.total_size_avif,
    value: formatSize(props.result.total_size_avif),
    condition: props.result.total_size_avif > 0,
  },
]);

const tags = computed(() => [
  {
    value: `${t('modals.status.tags.compress')} ${formatTime(props.result.duration_opt)}`,
    condition: props.result.duration_opt > 0,
  },
  {
    value: `WebP: ${formatTime(props.result.duration_webp)}`,
    condition: props.result.duration_webp > 0,
  },
  {
    value: `AVIF: ${formatTime(props.result.duration_avif)}`,
    condition: props.result.duration_avif > 0,
  },
]);
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
    <div class="status-modal__header">
      <template v-for="({ title, condition, modifier }, index) in header">
        <h3
          v-if="condition"
          :key="index"
          class="status-modal__title"
          :class="{ [`status-modal__title--${modifier}`]: !!modifier }"
        >
          <p class="h1-r">{{ title }}</p>
        </h3>
      </template>
    </div>

    <div class="status-modal__body">
      <div v-if="hasError" class="status-modal__state">
        <div class="status-modal__icon status-modal__icon--error">!</div>

        <p class="status-modal__error-msg">{{ error }}</p>
      </div>

      <div v-else-if="!isFinished" class="status-modal__state">
        <div
          class="status-modal__progress-circle"
          :style="{ '--p': percentage }"
        >
          <span class="status-modal__progress-text">{{ percentage }}%</span>
        </div>

        <div class="status-modal__progress-details">
          <div class="status-modal__filename">
            <p class="s2-r">{{ currentFile }}</p>
          </div>

          <div class="status-modal__counter">
            <p class="s2-r">{{ done }} / {{ total }}</p>
          </div>
        </div>

        <UiButton
          size="sm"
          :title="$t('modals.status.button.cancel')"
          class="status-modal__button"
          theme="warn"
          @click="handleCancel"
        />
      </div>

      <div v-else class="status-modal__state">
        <div
          v-if="!isCanceled"
          class="status-modal__icon status-modal__icon--success"
        >
          <span>✓</span>
        </div>

        <div v-else class="status-modal__icon status-modal__icon--canceled">
          <span>■</span>
        </div>

        <div class="status-modal__grid">
          <div class="status-modal__stat">
            <span class="status-modal__stat-label">
              {{ $t('modals.status.stats.0') }}
            </span>

            <span class="status-modal__stat-value">{{ filesDisplay }}</span>
          </div>

          <div class="status-modal__stat">
            <span class="status-modal__stat-label">
              {{ $t('modals.status.stats.1') }}
            </span>

            <span class="status-modal__stat-value">
              {{ formatTime(result.duration_total) }}
            </span>
          </div>

          <div class="status-modal__stat status-modal__stat--highlight">
            <span class="status-modal__stat-label">
              {{ $t('modals.status.stats.2') }}
            </span>

            <span class="status-modal__stat-value">{{ savingsPercent }}%</span>
          </div>
        </div>

        <div class="status-modal__table">
          <div class="status-modal__row status-modal__row--header">
            <span class="status-modal__cell">
              {{ $t('modals.status.table.head.0') }}
            </span>

            <span class="status-modal__cell">
              {{ $t('modals.status.table.head.1') }}
            </span>
          </div>

          <template
            v-for="({ label, value, rawValue, condition }, index) in table"
          >
            <div
              v-if="condition"
              :key="index"
              class="status-modal__row"
              :class="{
                ['status-modal__row--highlight']:
                  rawValue ===
                  Math.min(
                    ...table.map(({ rawValue }) => rawValue || Infinity),
                  ),
              }"
            >
              <span class="status-modal__cell status-modal__cell--label">
                {{ label }}
              </span>

              <span class="status-modal__cell">{{ value }}</span>
            </div>
          </template>
        </div>

        <div
          v-if="tags.filter(({ condition }) => condition).length > 1"
          class="status-modal__time-block"
        >
          <div class="status-modal__time-subtitle">
            <p class="s1-r">{{ $t('modals.status.tags.title') }}</p>
          </div>

          <div class="status-modal__tags">
            <div
              v-for="({ value }, index) in tags.filter(
                ({ condition }) => condition,
              )"
              :key="index"
              class="status-modal__tag"
            >
              <p class="s2-r">{{ value }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <AFade>
      <div v-if="!optStore.isProcessing" class="status-modal__footer">
        <UiButton
          :title="$t('modals.status.button.close')"
          theme="accent"
          class="status-modal__button"
          :disabled="optStore.isProcessing"
          @click="emit('confirm')"
        />
      </div>
    </AFade>
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
    display: flex;
    align-items: center;
    justify-content: center;
    width: em(56, 20);
    height: em(56, 20);
    margin-bottom: em(16, 20);
    font-size: em(20);
    border-radius: 50%;

    &--success {
      color: $accent-color-success;
      background-color: rgb(76 175 80 / 10%);
      border: 1px solid $accent-color-success;
    }

    &--error {
      color: $text-color-warn;
      background-color: rgb(244 67 54 / 10%);
      border: 1px solid $text-color-warn;
    }

    &--canceled {
      width: em(56, 24);
      height: em(56, 24);
      margin-bottom: em(16, 24);
      font-size: em(24);
      color: $accent-color-warn;
      background-color: rgb(255 152 0 / 10%);
      border: 1px solid $accent-color-warn;

      & > span {
        translate: 0 -10%;
      }
    }
  }

  &__content {
    position: relative;
    display: flex;
    flex-direction: column;
    width: 90%;
    max-width: em(350);
    max-height: 90vh;
    padding: em(24);
    background-color: $background-color-primary;
    border-radius: em(10);
    box-shadow: 0 em(10) em(40) rgb(0 0 0 / 20%);
  }

  &__header {
    margin-bottom: em(8);
    text-align: center;
  }

  &__title {
    margin: 0;

    &--success {
      color: $accent-color-success;
    }

    &--error {
      color: $text-color-warn;
    }

    &--canceled {
      color: $accent-color-warn;
    }
  }

  &__body {
    display: flex;
    flex-grow: 1;
    flex-direction: column;
    align-items: center;
    overflow-y: auto;
  }

  &__button {
    min-width: em(200);
  }

  &__state {
    display: flex;
    flex-direction: column;
    gap: em(16);
    align-items: center;
    width: 100%;
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
    align-items: center;
    padding: em(10) em(4);
    background-color: $background-color-secondary;
    border-radius: em(8);

    &--highlight {
      color: $accent-color-success;
    }
  }

  &__stat-label {
    margin-bottom: em(4);
    font-size: 0.75em;
    color: $text-color-secondary;
  }

  &__stat-value {
    font-weight: 700;
  }

  &__table {
    width: 100%;
    margin-bottom: 16px;
    overflow: hidden;
    border: 1px solid #eeeeee;
    border-radius: 8px;
  }

  &__row {
    display: flex;
    justify-content: space-between;
    padding: em(8) em(12);
    font-size: 0.9em;
    border-bottom: 1px solid #eeeeee;

    &:last-child {
      border-bottom: none;
    }

    &--header {
      font-size: 0.8em;
      color: $text-color-secondary;
      text-transform: uppercase;
      background-color: #fafafa;
    }

    &--highlight {
      color: $accent-color-success;
    }
  }

  &__time-block {
    width: 100%;
    text-align: center;
  }

  &__time-subtitle {
    margin-bottom: em(6);
    color: $text-color-secondary;
    text-transform: uppercase;
  }

  &__tags {
    display: flex;
    flex-wrap: wrap;
    gap: em(8);
    justify-content: center;
  }

  &__tag {
    padding: em(4) em(8);
    font-size: 0.9em;
    background: $background-color-additional;
    border-radius: em(4);
  }

  &__error-msg {
    width: 100%;
    padding: em(12);
    color: $text-color-warn;
    text-align: center;
    background: $background-color-secondary;
    border-radius: em(8);
  }

  &__progress-circle {
    --percentage: calc(var(--p) * 1%);

    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: em(80);
    height: em(80);
    background: conic-gradient(
      $accent-color-secondary var(--percentage),
      #eeeeee 0
    );
    border-radius: 50%;
    transition: --percentage $time-fast $ease;

    &::before {
      position: absolute;
      inset: em(8);
      content: '';
      background: $color-white;
      border-radius: 50%;
    }
  }

  &__progress-text {
    position: relative;
    font-size: 1.1rem;
    font-weight: bold;
    color: $accent-color-secondary;
  }

  &__progress-details {
    width: em(300);
    text-align: center;
  }

  &__filename {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  &__counter {
    margin: 0;
  }

  &__footer {
    display: flex;
    justify-content: center;
    margin-top: em(24);
  }
}
</style>
