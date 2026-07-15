<script setup>
const { $gsap: gsap } = useNuxtApp();
const reducedMotion = usePreferredReducedMotion();
const root = ref(null);

const props = defineProps({
  tag: {
    type: String,
    default: 'ul',
  },
  duration: {
    type: Number,
    default: 0.25,
  },
});

const onBeforeEnter = (el) => {
  if (reducedMotion.value === 'reduce') return;
  gsap.set(el, {
    autoAlpha: 0,
    height: 0,
    marginBottom: 0,
    marginTop: 0,
    paddingTop: 0,
    paddingBottom: 0,
    overflow: 'hidden',
    z: 0,
    backfaceVisibility: 'hidden',
  });
};

const onEnter = (el, done) => {
  if (reducedMotion.value === 'reduce') return done();
  gsap.to(el, {
    autoAlpha: 1,
    height: 'auto',
    marginTop: '',
    marginBottom: '',
    paddingTop: '',
    paddingBottom: '',
    onComplete: () => {
      gsap.set(el, { clearProps: 'all' });
      done();
    },
    duration: props.duration,
    ease: 'power1.out',
  });
};

const onLeave = (el, done) => {
  if (reducedMotion.value === 'reduce') return done();
  gsap.set(el, {
    overflow: 'hidden',
    z: 0,
    backfaceVisibility: 'hidden',
  });

  gsap.to(el, {
    autoAlpha: 0,
    height: 0,
    marginTop: 0,
    marginBottom: 0,
    paddingTop: 0,
    paddingBottom: 0,
    onComplete: done,
    duration: props.duration,
    ease: 'power1.in',
  });
};

onUnmounted(() => {
  if (root.value) gsap.killTweensOf(root.value.children);
});
</script>

<template>
  <TransitionGroup
    ref="root"
    :tag="props.tag"
    :css="false"
    class="smooth-list"
    @before-enter="onBeforeEnter"
    @enter="onEnter"
    @leave="onLeave"
  >
    <slot />
  </TransitionGroup>
</template>

<style lang="scss" scoped>
.smooth-list {
  position: relative;
  list-style: none;
  transition: height $time-normal $ease;
}
</style>
