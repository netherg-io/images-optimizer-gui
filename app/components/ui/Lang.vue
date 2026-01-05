<script setup>
const { locale, setLocale } = useI18n();

const options = [
  { code: 'en', label: 'EN' },
  { code: 'uk', label: 'УК' },
];

function switchLocale(code) {
  setLocale(code);
}

const activeIndex = computed(() => {
  const index = options.findIndex((item) => item.code === locale.value);
  return index === -1 ? 0 : index;
});
</script>

<template>
  <div
    class="ui-lang-switch"
    :style="{
      '--active-index': activeIndex,
      '--items-count': options.length,
    }"
  >
    <div class="ui-lang-switch__shutter" />

    <button
      v-for="item in options"
      :key="item.code"
      type="button"
      class="ui-lang-switch__option"
      :class="{ 'ui-lang-switch__option--active': locale === item.code }"
      @click="switchLocale(item.code)"
    >
      <span class="s2-r">{{ item.label }}</span>
    </button>
  </div>
</template>

<style lang="scss" scoped>
.ui-lang-switch {
  --toggle-gap: #{em(8)};
  --toggle-padding: #{em(4)};

  position: relative;
  display: flex;
  gap: var(--toggle-gap);
  width: fit-content;
  padding-inline: var(--toggle-padding);
  overflow: hidden;
  background-color: $background-color-secondary;
  border-radius: em(10);
  transition: background-color $time-normal $ease;

  &__shutter {
    position: absolute;
    top: em(6);
    bottom: em(6);
    left: var(--toggle-padding);
    z-index: 1;
    width: calc(
      (
          100% - (var(--toggle-padding) * 2) -
            (var(--toggle-gap) * (var(--items-count) - 1))
        ) /
        var(--items-count)
    );
    background-color: $background-color-primary;
    border-radius: em(8);
    box-shadow:
      0 1px 3px 0 rgb(0 0 0 / 10%),
      0 1px 2px -1px rgb(0 0 0 / 10%);
    translate: calc(var(--active-index) * (100% + var(--toggle-gap)));
    transition: $time-normal $ease;
    transition-property: background-color, translate;
    will-change: translate;
  }

  &__option {
    position: relative;
    z-index: 2;
    flex: 1;
    flex-basis: 0;
    min-width: em(40);
    padding: em(6);
    text-align: center;
    cursor: pointer;
    background: none;
    border: none;
    transition: color $time-normal $ease;

    &--active {
      color: $text-color-primary;
    }
  }
}
</style>
