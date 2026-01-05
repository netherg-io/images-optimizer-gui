import gsap from 'gsap';
import { Flip, MorphSVGPlugin } from 'gsap/all';

export default defineNuxtPlugin({
  parallel: true,
  setup() {
    gsap.registerPlugin(Flip);
    gsap.registerPlugin(MorphSVGPlugin);

    return {
      provide: { gsap, flip: Flip },
    };
  },
});
