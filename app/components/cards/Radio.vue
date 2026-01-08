<script setup>
defineProps({
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
  <div class="radio-card" :class="{ ['radio-card--active']: isActive }">
    <div class="radio-card__icon-wrapper">
      <div class="radio-card__icon" />
    </div>

    <div class="radio-card__content">
      <div class="radio-card__title">
        <p class="s1-r">{{ title }}</p>
      </div>

      <div class="radio-card__description">
        <p class="s1-r">{{ description }}</p>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.radio-card {
  $parent: &;

  display: flex;
  gap: em(12);
  align-items: baseline;
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
    position: relative;
    align-self: flex-start;
    width: em(20);
    height: em(20);
    background-color: transparent;
    border: 2px solid $border-color-secondary;
    border-radius: 50%;
    transition: $time-normal $ease;
    transition-property: background-color, border-color;
  }

  &__icon {
    position: absolute;
    inset: em(4);
    background-color: $color-white;
    border-radius: 50%;
    scale: 0;
    transition: scale $time-normal $ease;
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
        scale: 1;
      }

      &__icon-wrapper {
        background-color: $accent-color-primary;
        border-color: $accent-color-primary;
      }
    }
  }
}
</style>
