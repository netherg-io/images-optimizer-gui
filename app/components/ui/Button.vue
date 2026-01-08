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
    validator: (v) => ['primary', 'accent', 'warn', 'browse'].includes(v),
  },
  size: {
    type: String,
    default: 'md',
    validator: (v) => ['md', 'sm', 'xs'].includes(v),
  },
});
</script>

<template>
  <button
    class="ui-button"
    :class="{
      [`ui-button--theme--${theme}`]: !!theme,
      [`ui-button--size--${size}`]: !!size,
    }"
  >
    <CIcon
      v-if="icon && iconPos === 'left'"
      class="ui-button__icon"
      :name="icon"
    />

    <slot>
      <span v-if="!$slots.default && title" class="ui-button__font">
        {{ title }}
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
  align-items: center;
  justify-content: center;
  background-color: transparent;
  border: 1px solid transparent;
  transition: $time-normal $ease;
  transition-property: color, border-color, background-color;

  &__font {
    @include s1;
  }

  &__icon {
    width: em(20);
    height: em(20);
    object-fit: contain;
    transition: color $time-normal $ease;
  }

  &--theme {
    &--primary {
      background-color: $background-color-primary;
      border-color: $border-color-secondary;

      @include hover {
        background-color: $background-color-tertiary;
      }

      #{$parent} {
        &__icon {
          color: $icon-color-secondary;
        }
      }
    }

    &--accent {
      color: $color-white;
      background-color: $accent-color-primary;
      border-color: $border-color-secondary;

      @include hover {
        background-color: $accent-color-tertiary;
      }
    }

    &--warn {
      color: $text-color-warn;

      @include hover {
        background-color: $background-color-warn;
        border-color: $background-color-warn;
      }

      #{$parent} {
        &__icon {
          color: $icon-color-warn;
        }
      }
    }

    &--browse {
      color: $color-white;
      background-color: $background-color-browse;
      border-color: $background-color-browse;

      @include hover {
        background-color: color-mix(
          in srgb,
          #{$background-color-browse},
          #{$color-white} 20%
        );
        border-color: color-mix(
          in srgb,
          #{$background-color-browse},
          #{$color-white} 20%
        );
      }
    }
  }

  &--size {
    &--md {
      gap: em(12);
      padding: em(16);
      border-radius: em(14);
    }

    &--sm {
      gap: em(8);
      padding: em(8) em(16);
      border-radius: em(10);
    }

    &--xs {
      gap: em(4);
      padding: em(8);
      border-radius: em(10);

      #{$parent} {
        &__icon {
          width: em(16);
          height: em(16);
        }
      }
    }
  }
}
</style>
