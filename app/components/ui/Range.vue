<script setup>
defineProps({
  title: {
    type: String,
    default: '',
  },
});

const model = defineModel({
  type: String,
  default: '0',
});
</script>

<template>
  <div class="ui-range">
    <div class="ui-range__content">
      <div v-if="title" class="ui-range__title">
        <p class="s1-r">{{ title }}</p>
      </div>

      <div class="ui-range__model">
        <p class="s1-r">{{ model }}%</p>
      </div>
    </div>

    <input v-model="model" type="range" class="ui-range__input" />

    <div class="ui-range__content">
      <div class="ui-range__boundary">
        <p class="s1-r">{{ $t('sections.options.range.0.min') }}</p>
      </div>

      <div class="ui-range__boundary">
        <p class="s1-r">{{ $t('sections.options.range.0.max') }}</p>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
$c-track: #e2e8f0;

.ui-range {
  display: flex;
  flex-direction: column;
  gap: em(24);

  &__model {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: em(60);
    min-height: em(32);
    color: $icon-color-additional;
    background-color: $background-color-accent;
    border-radius: em(10);
    transition: $time-normal $ease;
    transition-property: color, background-color;
  }

  &__content {
    display: flex;
    gap: em(32);
    align-items: flex-start;
    justify-content: space-between;
  }

  &__input {
    width: 100%;
    height: em(6);
    font-size: inherit;
    appearance: none;
    cursor: pointer;
    outline: none;
    background: transparent;

    &::-webkit-slider-runnable-track {
      width: 100%;
      height: em(8);
      background: $c-track;
      border-radius: em(4);
    }

    &::-webkit-slider-thumb {
      width: em(20);
      height: em(20);
      margin-top: em(-6);
      appearance: none;
      background: $accent-color-primary;
      border: 3px solid $color-white;
      border-radius: 50%;
      box-shadow: 0 2px 6px rgb(0 0 0 / 30%);
      transition: transform $time-fast $ease;

      &:hover {
        transform: scale(1.1);
      }
    }

    &::-moz-range-track {
      width: 100%;
      height: em(8);
      background: $c-track;
      border-radius: em(4);
    }

    &::-moz-range-thumb {
      width: em(20);
      height: em(20);
      background: $accent-color-primary;
      border: 3px solid $color-white;
      border-radius: 50%;
      box-shadow: 0 2px 6px rgb(0 0 0 / 30%);
      transition: transform $time-fast $ease;

      &:hover {
        transform: scale(1.1);
      }
    }

    &:focus-visible {
      &::-webkit-slider-thumb {
        box-shadow: 0 0 0 4px rgba($accent-color-primary, 0.3);
      }

      &::-moz-range-thumb {
        box-shadow: 0 0 0 4px rgba($accent-color-primary, 0.3);
      }
    }
  }

  &__boundary {
    color: $text-color-secondary;
    transition: color $time-normal $ease;
  }
}
</style>
