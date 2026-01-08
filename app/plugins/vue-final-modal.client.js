/**
 * Vue Final Modal integration
 * @example
 * const modal = useModal({
 *   component: resolveComponent('LazyModalComponent')
 * });
 * onMounted(() => modal.open());
 * @see https://vue-final-modal.org/
 */
export default defineNuxtPlugin({
  parallel: true,
  setup: async (nuxtApp) => {
    await import('vue-final-modal/style.css');
    const { createVfm } = await import('vue-final-modal');

    const vfm = createVfm();
    nuxtApp.vueApp.use(vfm);
  },
});
