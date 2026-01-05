import { I18N_DEFAULT_LOCALE } from './app/constants/i18n';

export default defineI18nConfig(() => ({
  legacy: false,
  locale: I18N_DEFAULT_LOCALE,
  fallbackLocale: I18N_DEFAULT_LOCALE,
}));
