<script setup>
const { $gsap: gsap } = useNuxtApp();
const { theme, toggleTheme } = useTheme();
const isDark = computed(() => theme.value === 'dark');
const reducedMotion = usePreferredReducedMotion();
const svgContainer = ref(null);
const sunRays = ref(null);

const sunPath = 'M12,7c-2.76,0-5,2.24-5,5s2.24,5,5,5s5-2.24,5-5S14.76,7,12,7z';
const moonPath =
  'M12,3c-4.97,0-9,4.03-9,9s4.03,9,9,9s9-4.03,9-9c0-0.46-0.04-0.92-0.1-1.36c-0.98,1.37-2.58,2.27-4.4,2.27c-3.03,0-5.5-2.47-5.5-5.5c0-1.82,0.89-3.42,2.27-4.4C12.92,3.04,12.46,3,12,3z';

const animateIcon = (dark) => {
  if (!svgContainer.value || !sunRays.value) return;
  const duration = reducedMotion.value === 'reduce' ? 0 : 0.2;
  gsap.killTweensOf([svgContainer.value, sunRays.value]);
  gsap.fromTo(
    svgContainer.value,
    {
      rotation: dark ? -90 : 90,
      scale: 0.75,
      transformOrigin: 'center center',
    },
    { rotation: 0, scale: 1, duration, ease: 'power4.out' },
  );
  gsap.to(sunRays.value, {
    opacity: dark ? 0 : 1,
    scale: dark ? 0.5 : 1,
    transformOrigin: 'center center',
    duration,
    ease: 'power4.out',
  });
};

onMounted(() => {
  gsap.set(sunRays.value, {
    opacity: isDark.value ? 0 : 1,
    scale: isDark.value ? 0.5 : 1,
    transformOrigin: 'center center',
  });
});

watch(isDark, animateIcon, { flush: 'post' });

onUnmounted(() => {
  gsap.killTweensOf([svgContainer.value, sunRays.value]);
});
</script>

<template>
  <button
    class="ui-theme-switch"
    :aria-label="$t('common.toggle-theme')"
    @click="toggleTheme"
  >
    <svg
      ref="svgContainer"
      class="ui-theme-switch__svg"
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <g ref="sunRays" class="ui-theme-switch__rays">
        <path
          d="M12,1V3 M12,21V23 M4.22,4.22L5.64,5.64 M18.36,18.36L19.78,19.78 M1,12H3 M21,12H23 M4.22,19.78L5.64,18.36 M18.36,5.64L19.78,4.22"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
        />
      </g>
      <path
        class="ui-theme-switch__body"
        :d="isDark ? moonPath : sunPath"
        fill="currentColor"
      />
    </svg>
  </button>
</template>

<style lang="scss" scoped>
.ui-theme-switch {
  display: flex;
  align-items: center;
  justify-content: center;
  width: em(45);
  height: em(45);
  padding: 0;
  color: $text-color-primary;
  cursor: pointer;
  background-color: $background-color-primary;
  border: 1px solid $border-color-secondary;
  border-radius: em(14);
  transition: $time-normal $ease;
  transition-property: background-color, border-color;

  @include hover {
    background-color: $background-color-tertiary;
  }

  &__svg {
    width: em(20);
    height: em(20);
    overflow: visible;
  }

  &__body {
    fill: currentcolor;
  }

  &__rays {
    stroke: currentcolor;
  }
}
</style>
