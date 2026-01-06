<script setup>
defineProps({
  isActive: {
    type: Boolean,
    default: false,
  },
});
</script>

<template>
  <div class="ui-file-input" :class="{ [`ui-file-input--active`]: isActive }">
    <div class="ui-file-input__content">
      <div class="ui-file-input__wrapper">
        <CIcon class="ui-file-input__icon" name="upload" />
      </div>

      <div class="ui-file-input__title">
        <p class="s1-r">{{ $t('sections.add-files.input.title') }}</p>
      </div>

      <div class="ui-file-input__description">
        <p class="s1-r">{{ $t('sections.add-files.input.description') }}</p>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.ui-file-input {
  background-color: $background-color-primary;
  border: 2px dashed $border-color-tertiary;
  border-radius: em(10);
  transition: $time-normal $ease;
  transition-property: border-color, background-color;

  &__icon {
    width: em(40);
    height: em(40);
    color: $icon-color-primary;
  }

  &__content {
    display: flex;
    flex-direction: column;
    gap: em(8);
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  &__wrapper {
    padding: em(20);
    margin-bottom: em(14);
    background-color: $background-color-secondary;
    border-radius: 50%;
    transition: background-color $time-normal $ease;
  }

  &__title {
  }

  &__description {
    color: $text-color-secondary;
    transition: color $time-normal $ease;
  }

  &--active {
    border-color: $border-color-accent;
  }
}
</style>
