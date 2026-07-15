import { createVfm } from 'vue-final-modal';
import 'vue-final-modal/style.css';

export default defineNuxtPlugin((nuxtApp) => {
  nuxtApp.vueApp.use(createVfm());
});
