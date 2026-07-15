<script setup>
defineProps({
  icon: {
    type: String,
    default: '',
  },
  title: {
    type: String,
    default: '',
  },
  description: {
    type: String,
    default: '',
  },
  isActive: {
    type: Boolean,
    default: false,
  },
});
</script>

<template>
  <button
    type="button"
    class="select-card"
    :aria-pressed="isActive"
    :class="{ ['select-card--active']: isActive }"
  >
    <div class="select-card__icon-wrapper">
      <CIcon class="select-card__icon" :name="icon" />
    </div>

    <div class="select-card__content">
      <div class="select-card__title">
        <p class="s1-r">{{ title }}</p>
      </div>

      <div class="select-card__description">
        <p class="s1-r">{{ description }}</p>
      </div>
    </div>
  </button>
</template>

<style lang="scss" scoped>
.select-card {
  $parent: &;

  display: flex;
  flex-direction: column;
  gap: em(12);
  padding: em(18);
  cursor: pointer;
  user-select: none;
  background-color: $background-color-primary;
  border: 1px solid $border-color-secondary;
  border-radius: em(16);
  transition: $time-normal $ease;
  transition-property: background-color, border-color;

  @include hover {
    background-color: $background-color-tertiary;
  }

  &__icon-wrapper {
    align-self: flex-start;
    padding: em(10);
    background-color: $background-color-secondary;
    border-radius: em(10);
    transition: background-color $time-normal $ease;
  }

  &__icon {
    width: em(20);
    height: em(20);
    transition: color $time-normal $ease;
  }

  &__content {
    display: flex;
    flex-direction: column;
    gap: em(4);
  }

  &__description {
    color: $text-color-secondary;
    transition: color $time-normal $ease;
  }

  &--active {
    background-color: $background-color-accent !important;
    border-color: $accent-color-secondary;

    #{$parent} {
      &__icon {
        color: $icon-color-additional;
      }

      &__icon-wrapper {
        background-color: $background-color-additional;
      }
    }
  }
}
</style>
