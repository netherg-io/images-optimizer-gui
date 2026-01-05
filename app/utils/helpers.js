/*
 * import {
 *   disableBodyScroll,
 *   enableBodyScroll,
 *   clearAllBodyScrollLocks,
 * } from 'body-scroll-lock';
 */

// common
/**
 * Get value with boundaries
 * @param {number} value desired value
 * @param {number} min min boundary
 * @param {number} max max boundary
 * @returns {number} calculated value
 * @example
 * minMax(15, 5, 10); // returns 10 (max boundary)
 */
export function minMax(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

/**
 * Simple object clone
 * @param {object} object object to clone
 * @returns {any} cloned object
 * @example
 * const clonedObject = cloneObject(originalObject);
 */
export function cloneObject(object) {
  return JSON.parse(JSON.stringify(object));
}

// promise-based utils
/**
 * Await timeout
 * @param {number} [duration] timeout duration
 * @returns {Promise<void>} timeout promise
 * @example
 * await awaitTimeout(100);
 */
export function awaitTimeout(duration = 0) {
  return new Promise((resolve) => setTimeout(resolve, duration));
}

/**
 * Await request animation frame
 * @returns {Promise<void>} request animation frame promise
 * @example
 * await awaitRAF();
 */
export function awaitRAF() {
  return new Promise((resolve) => requestAnimationFrame(resolve));
}

// em font-sizing
/**
 * Get element font size
 * @param {HTMLElement|string} [element] reference element or selector
 * @returns {number} element font-size
 */
export function getElementFz(element = document.body) {
  if (typeof element === 'string') {
    element = document.querySelector(element);
  }

  if (!element) return 0;

  const elementStyle = window.getComputedStyle(element);
  return parseFloat(elementStyle.fontSize);
}

/**
 * Get px value regarding element font-size
 * @param {number} pxValue desired px value
 * @returns {number} calculated px value
 */
export function toResizedPx(pxValue) {
  const contextElement = document.querySelector('.resize') || document.body;
  const pxContext = getElementFz(contextElement);
  return (pxValue / 16) * pxContext;
}

/**
 * Get em value regarding element font-size
 * @param {number} pxValue desired px value
 * @returns {number} calculated em value
 */
export function toResizedEm(pxValue) {
  const contextElement = document.querySelector('.resize') || document.body;
  const pxContext = getElementFz(contextElement);
  return pxValue / pxContext;
}

// preload images
/**
 * Preload image
 * @param {string} url image url
 * @returns {Promise<void>} upload promise
 */
export function preloadImage(url) {
  return new Promise((resolve) => {
    const image = new Image();
    image.src = url;

    image.addEventListener('load', resolve);
    image.addEventListener('error', resolve);
  });
}

/**
 * Preload images
 * @param {string[]} imageUrls image urls
 * @returns {Promise<Awaited<void>[]>} upload promises
 */
export function preloadImages(imageUrls = []) {
  const preloads = Array.from(imageUrls).map(preloadImage);

  return Promise.all(preloads);
}

// preload fonts
/**
 * Get font preload list
 * @param {string} path font path
 * @param {number[]} weights font weights
 * @param {string} baseUrl site base url
 * @returns {{rel: string, href: string, as: string, type: string, crossorigin: true}[]} font preload list
 */
export function getFontPreloadList({ path, weights }, baseUrl = '/') {
  return weights.map((weight) => ({
    rel: 'preload',
    href: `${baseUrl}fonts/${path}${weight}.woff2`,
    as: 'font',
    type: 'font/woff2',
    crossorigin: true,
  }));
}

/**
 * Get fonts preload list
 * @param {object[]} fontsList font params list
 * @param {string} fontsList.path font path
 * @param {number[]} fontsList.weights font weights
 * @param {string} [baseUrl] site base url
 * @returns {{rel: string, href: string, as: string, type: string, crossorigin: true}[]} fonts preload list
 */
export function getFontsPreloadList(fontsList, baseUrl = '/') {
  return fontsList.reduce(
    (result, { path, weights }) => [
      ...result,
      ...getFontPreloadList({ path, weights }, baseUrl),
    ],
    [],
  );
}

export function round(value, digits = 0) {
  const pow = Math.pow(10, digits);
  return Math.round(value * pow) / pow;
}

/**
 * Преобразует число в сокращённый строковый формат:
 * - >=1_000_000_000 → X.YB
 * - >=1_000_000     → X.YM
 * - >=1_000         → X.YK
 * - иначе           → как есть
 *
 * @param {number|string} input — число или строка с цифрами (можно с пробелами)
 * @param {number} [decimals=2] — количество знаков после запятой
 * @returns {string} Например: "1.3M", "850", "4K"
 */
export function formatNumberAbbr(input, decimals = 2) {
  // Убираем пробелы и приводим к числу
  const num =
    typeof input === 'string' ? parseFloat(input.replace(/\s+/g, '')) : input;

  const abs = Math.abs(num);
  const sign = num < 0 ? '-' : '';

  // Вспомогательная функция форматирования
  function fmt(divisor, suffix) {
    const n = (num / divisor).toFixed(decimals);
    // Убираем лишний ".0" в конце
    const clean = parseFloat(n.replace(/\.0+$/, '')).toFixed(2);
    return sign + clean + suffix;
  }

  if (abs >= 1e9) {
    return fmt(1e9, 'B');
  }
  if (abs >= 1e6) {
    return fmt(1e6, 'M');
  }
  if (abs >= 1e3) {
    return fmt(1e3, 'K');
  }
  // Меньше тысячи — просто возвращаем число без изменений
  return num ? String(num) : num;
}

export function scrollHidden(value) {
  /*
   * const html = document?.documentElement;
   * const body = document?.body
   * TODO закоментил потому что на IOS сайт зависал
   */
  if (value) {
    /*
     * clearAllBodyScrollLocks();
     * disableBodyScroll(html);
     * disableBodyScroll(body);
     */
  } else {
    /*
     * enableBodyScroll(html);
     * enableBodyScroll(body);
     * clearAllBodyScrollLocks();
     */
  }
}

/**
 * Переводит дату из формата YYYY-MM-DD в формат Q{n} YYYY
 * @param {string} isoDate — строка даты, например "2025-07-23"
 * @returns {string} — строка вида "Q3 2025"
 */
export function formatToQuarter(isoDate = '') {
  // Разбиваем по дефисам и получаем год и месяц
  const [year, month] = isoDate.split('-').map(Number);
  // Квартал: (0–2)→1, (3–5)→2, (6–8)→3, (9–11)→4
  const quarter = Math.floor((month - 1) / 3) + 1;
  return `Q${quarter} ${year}`;
}

/**
 * Форматирует число или строку с цифрами, вставляя пробелы как разделители тысяч.
 * Примеры:
 *   formatPrice(1200000)      // "1 200 000"
 *   formatPrice("1200000")    // "1 200 000"
 *   formatPrice("1002003.45") // "1 002 003.45"
 *
 * @param {number|string} value — исходное значение (число или строка с цифрами и, возможно, пробелами)
 * @returns {string} — строка с разделением пробелами по тысячам
 */
export function formatPrice(value) {
  // Приводим к строке без пробелов
  let str =
    typeof value === 'number'
      ? String(value)
      : value.toString().replace(/\s+/g, '');

  // Сохраняем знак, если есть
  const sign = str.startsWith('-') ? '-' : '';
  if (sign) str = str.slice(1);

  // Разделяем целую и дробную части
  const [intPart, fracPart] = str.split('.');

  // Вставляем пробел перед каждой группой из трёх цифр с конца
  const intFormatted = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ',');

  // Собираем обратно, добавляем дробную часть, если она была
  return sign + intFormatted + (fracPart !== undefined ? '.' + fracPart : '');
}

/**
 * Переводит дату из формата ISO в формат 2025.02.01
 * @param {string} isoDate — строка даты, например "2025-07-11T12:57:48.426Z"
 * @returns {string} — строка вида "Q3 2025"
 */
export const formatDate = (dateString) => {
  if (!dateString) return '';

  const date = new Date(dateString);
  const day = date.getDate().toString().padStart(2, '0');
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const year = date.getFullYear();

  return `${day}.${month}.${year}`;
};

export const formatBytes = (a, b) => {
  if (0 === a) return '0 Bytes';

  const c = 1024,
    d = b || 2,
    e = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'],
    f = Math.floor(Math.log(a) / Math.log(c));

  return parseFloat((a / Math.pow(c, f)).toFixed(d)) + ' ' + e[f];
};

export function buildStrapiFilters(filterArray) {
  const filters = {};

  filterArray.forEach(({ name, value }) => {
    // Пропускаем пустые значения
    if (
      value == null ||
      (Array.isArray(value) && value.length === 0) ||
      (typeof value === 'object' &&
        !Array.isArray(value) &&
        value.min?.value == null &&
        value.max?.value == null)
    ) {
      return;
    }

    // 1) Диапазон цен
    if (
      name === 'price' &&
      typeof value === 'object' &&
      !Array.isArray(value)
    ) {
      filters.price_aed = {};
      if (value.min.value != null)
        filters.price_aed.$gte = Number(value.min?.value || 0);
      if (value.max.value != null)
        filters.price_aed.$lte = Number(value.max?.value || Infinity);
      return;
    }

    const filtersData = Array.isArray(value) ? value : [value];
    const valuesFromFiltersData = filtersData.map((data) => data.value);
    // 2) Вложенный фильтр по slug для связанной коллекции project(s)
    if (name === 'projects' || name === 'project') {
      // Если пришёл просто строкой, приводим к массиву

      // Здесь — любой оператор: $in (несколько), или $eq (одно значение)
      filters[name] = {
        slug: { $in: valuesFromFiltersData },
      };
      return;
    }

    // 3) Обычный фильтр по полю в массиве
    filters[name] = {
      $in: valuesFromFiltersData,
    };
  });

  return filters;
}

export function buildQueryParams(filterArray) {
  const query = {};

  filterArray.forEach(({ name, value }) => {
    console.log(value);

    if (name === 'price' && typeof value === 'object') {
      if (value.min.value != null) query.price_min = value.min.value;
      if (value.max.value != null) query.price_max = value.max.value;
      return;
    }
    // массив → "a,b,c"; одиночное значение тоже оборачиваем в массив
    const vals = Array.isArray(value) ? value : [value];
    if (vals.length) {
      query[name] = vals.map((data) => data.value).join(',');
    }
  });

  return query;
}

export function parseQueryParams(query) {
  const filters = [];
  let priceMin = null,
    priceMax = null;

  Object.entries(query).forEach(([key, val]) => {
    if (key === 'price_min') {
      priceMin = val;
      return;
    }
    if (key === 'price_max') {
      priceMax = val;
      return;
    }
    // всё остальное: разбить по запятым
    const arr = ('' + val).split(',').filter(Boolean);
    filters.push({ name: key, value: arr });
  });

  if (priceMin != null || priceMax != null) {
    filters.push({
      name: 'price',
      value: {
        min: {
          label: priceMin != null ? Number(priceMin) : null,
          value: priceMin != null ? Number(priceMin) : null,
        },
        max: {
          label: priceMin != null ? Number(priceMax) : null,
          value: priceMin != null ? Number(priceMax) : null,
        },
      },
    });
  }

  return filters;
}

export function youtubeParser(url) {
  const regExp =
    /^.*((youtu.be\/)|(v\/)|(\/u\/\w\/)|(embed\/)|(watch\?))\??v?=?([^#&?]*).*/;
  const match = url.match(regExp);
  return match && match[7].length == 11 ? match[7] : false;
}

export function slugFromString(name) {
  return name.toLowerCase().split(' ').join('_');
}

export const cleanPhone = (phone) => phone.replace(/\s/g, '');

/**
 * Builds a valid srcset string for <img> or <source> tags based on the image object and its formats.
 *
 * @param {object} obj - The image object containing the main url, width, and formats (e.g., large, medium, small).
 * @param {string} [mainKey='url'] - The key for the main image url in the object (usually 'url').
 * @param {string[]|string} [formatKeys=[]] - Array of format keys (or a single key) to include from obj.formats (e.g., ['large', 'medium', 'small']).
 * @returns {string} A srcset string, e.g. "/uploads/image.png 800w, /uploads/large_image.png 1600w, /uploads/medium_image.png 750w".
 *
 * @example
 * const image = {
 *   url: '/uploads/image.png',
 *   width: 800,
 *   formats: {
 *     large: { url: '/uploads/large_image.png', width: 1600 },
 *     medium: { url: '/uploads/medium_image.png', width: 750 },
 *   },
 * };
 *
 * const srcset = buildSrcset(image, 'url', ['large', 'medium']);
 * // srcset: "/uploads/image.png 800w, /uploads/large_image.png 1600w, /uploads/medium_image.png 750w"
 */
export function buildSrcset(obj, mainKey = 'url', formatKeys = []) {
  if (!obj) return '';
  const srcs = [];

  if (obj[mainKey] && obj.width) srcs.push(`${obj[mainKey]} ${obj.width}w`);

  if (Array.isArray(formatKeys)) {
    formatKeys.forEach((key) => {
      if (obj.formats?.[key]?.url && obj.formats?.[key]?.width) {
        srcs.push(`${obj.formats[key].url} ${obj.formats[key].width}w`);
      }
    });
  } else if (
    typeof formatKeys === 'string' &&
    obj.formats?.[formatKeys]?.url &&
    obj.formats?.[formatKeys]?.width
  ) {
    srcs.push(
      `${obj.formats[formatKeys].url} ${obj.formats[formatKeys].width}w`,
    );
  }

  return srcs.join(', ');
}

/**
 * Creates a new URL by combining the specified URLs
 * @param {string} baseURL The base URL
 * @param {string} relativeURL The relative URL
 *
 * @returns {string} The combined URL
 */
export function combineURLs(baseURL, relativeURL) {
  return relativeURL
    ? baseURL.replace(/\/?\/$/, '') + '/' + relativeURL.replace(/^\/+/, '')
    : baseURL;
}
