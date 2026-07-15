import { joinURL, hasProtocol } from 'ufo';
import { convertFileSrc } from '@tauri-apps/api/core';

export const formatSize = (bytes) => {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.min(
    sizes.length - 1,
    Math.floor(Math.log(bytes) / Math.log(k)),
  );
  return `${Number((bytes / k ** i).toFixed(2)).toLocaleString()} ${sizes[i]}`;
};

export function formatTime(seconds) {
  if (!seconds) return '0s';
  if (seconds < 1) return '< 1s';
  return seconds.toFixed(1) + 's';
}

/**
 * Helper to resolve a single URL.
 * 1. Checks if it is a local absolute path (Windows/Unix) and converts it for Tauri.
 * 2. If it has a protocol (http/https), returns as is.
 * 3. Otherwise, joins with baseURL.
 */
export const resolveUrl = (path) => {
  const {
    app: { baseURL },
  } = useRuntimeConfig();

  if (!path) return '';

  const isLocalAbsolutePath = /^(?:[a-zA-Z]:[\\/]|\\\\|\/)/.test(path);

  if (isLocalAbsolutePath) {
    try {
      return convertFileSrc(path);
    } catch (error) {
      console.error('Tauri convertFileSrc failed:', error);
      return path;
    }
  }

  if (hasProtocol(path, { acceptRelative: true })) {
    return path;
  }
  return joinURL(baseURL, path);
};

export const getParentPath = (path) => {
  if (!path) return '';
  const clean = path.replace(/[\\/]+$/, '');
  const index = Math.max(clean.lastIndexOf('\\'), clean.lastIndexOf('/'));
  if (index < 0) return '';
  if (index === 0) return clean[0];
  if (index === 2 && /^[a-zA-Z]:/.test(clean)) return clean.slice(0, 3);
  return clean.slice(0, index);
};
