import { I18N_LOCALES, I18N_DEFAULT_LOCALE } from './app/constants/i18n';
import { resolveAvailableI18nLocales } from './app/i18n/utils/resolve-locales';

const vitePlugins = [];

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: [
    '@pinia/nuxt',
    '@nuxtjs/i18n',
    '@vueuse/nuxt',
    'nuxt-svgo',
    '@formkit/auto-animate/nuxt',
    '@nuxt/eslint',
    '@nuxtjs/color-mode',
  ],
  ssr: false,
  components: [
    '@/components',
    { path: '@/components/common', prefix: 'C' },
    { path: '@/components/animation', prefix: 'A' },
    { path: '@/components/ui', prefix: 'Ui' },
    { path: '@/components/cards', prefix: 'Card' },
    { path: '@/components/sections', prefix: 'Section' },
    { path: '@/components/layout', prefix: 'L' },
  ],
  devtools: { enabled: false },
  css: ['reset-css', '@/assets/styles/base/index.scss'],
  colorMode: {
    preference: 'system',
    fallback: 'light',
    storageKey: 'theme-preference',
  },
  ignore: ['**/src-tauri/**'],
  devServer: {
    host: '0',
  },
  features: { inlineStyles: false },
  compatibilityDate: '2025-07-15',
  vite: {
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_'],
    server: {
      strictPort: true,
    },
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: '@use "@/assets/styles/utils" as *;',
          silenceDeprecations: ['global-builtin', 'import'],
        },
      },
    },
    plugins: vitePlugins,
  },
  eslint: {
    config: {
      stylistic: true,
    },
  },
  i18n: {
    langDir: '../app/i18n/locales',
    locales: resolveAvailableI18nLocales(I18N_LOCALES),
    defaultLocale: I18N_DEFAULT_LOCALE,
    vueI18n: '../i18n.config.ts',
  },
  svgo: { defaultImport: 'component', explicitImportsOnly: true },
});
