import gsap from 'gsap';
import { Flip } from 'gsap/all';

export default defineNuxtPlugin({
  parallel: true,
  setup() {
    gsap.registerPlugin(Flip);

    return {
      provide: { gsap, flip: Flip },
    };
  },
});
