import { existsSync } from 'fs';
import { join, resolve } from 'path';

const LOCALES_DIR = resolve(__dirname, '../locales');

export function resolveAvailableI18nLocales(locales) {
  if (!Array.isArray(locales)) return [];

  return locales
    .map((config) => ({
      code: config.code,
      language: config.language,
      file: `${config.code}.json`,
      dir: config.dir,

      // custom properties
      region: config.region,
      name: config.name,
      localizedName: config.localizedName,
    }))
    .filter((locale) => existsSync(join(LOCALES_DIR, locale.file)));
}
