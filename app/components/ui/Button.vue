<script setup>
defineProps({
  title: {
    type: String,
    default: '',
  },
  icon: {
    type: String,
    default: '',
  },
  iconPos: {
    type: String,
    default: 'left',
    validator: (pos) => ['left', 'right'].includes(pos),
  },
  theme: {
    type: String,
    default: 'primary',
    validator: (v) => ['primary', 'secondary'].includes(v),
  },
});
</script>

<template>
  <button
    class="ui-button"
    :class="{ [`ui-button--theme--${theme}`]: !!theme }"
  >
    <CIcon
      v-if="icon && iconPos === 'left'"
      class="ui-button__icon"
      :name="icon"
    />

    <slot>
      <span v-if="!$slots.default && title" class="ui-button__font">
        {{ $t(title) }}
      </span>
    </slot>

    <CIcon
      v-if="icon && iconPos === 'right'"
      class="ui-button__icon"
      :name="icon"
    />
  </button>
</template>

<style lang="scss" scoped>
.ui-button {
  $parent: &;

  display: flex;
  gap: em(12);
  align-items: center;
  justify-content: center;
  padding: em(16);
  border-radius: em(14);
  transition: $time-normal $ease;
  transition-property: border-color, background-color;

  &__font {
    @include s1;
  }

  &__icon {
    width: em(20);
    height: em(20);
    color: $icon-color-secondary;
    object-fit: contain;
    transition: color $time-normal $ease;
  }

  &--theme {
    &--primary {
      background-color: $background-color-primary;
      border: 1px solid $border-color-secondary;

      @include hover {
        background-color: $background-color-tertiary;
      }

      #{$parent} {
        &__icon {
          color: $icon-color-secondary;
        }
      }
    }
  }
}
</style>
