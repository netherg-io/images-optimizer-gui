<script setup>
defineProps({
  description: {
    type: String,
    default: '',
  },
});
</script>

<template>
  <div class="head-block">
    <div class="head-block__container container">
      <div class="head-block__content">
        <div class="head-block__title">
          <h1 class="h1-r">Image Optimizer</h1>
        </div>

        <div class="head-block__description">
          <p class="s1-r">{{ $t(description) }}</p>
        </div>
      </div>

      <div class="head-block__controls">
        <UiTheme />

        <UiLang />
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.head-block {
  &__container {
    display: flex;
    gap: em(20);
    align-items: flex-start;
    justify-content: space-between;
  }

  &__content {
    display: flex;
    flex-direction: column;
    gap: em(8);
  }

  &__description {
    color: $text-color-secondary;
    transition: color $time-normal $ease;
  }

  &__controls {
    display: flex;
    gap: em(16);
  }
}
</style>
