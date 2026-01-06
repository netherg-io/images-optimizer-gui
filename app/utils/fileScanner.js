import { stat, readDir } from '@tauri-apps/plugin-fs';
import { join } from '@tauri-apps/api/path';

import allowedExtensions from '@/configs/supportedExt';

const isSupported = (fileName) => {
  if (!fileName || !fileName.includes('.')) return false;
  const ext = fileName.split('.').pop().toLowerCase();
  return allowedExtensions.includes(ext);
};

const scanDirectory = async (dirPath) => {
  let totalSize = 0;
  let collectedFiles = [];

  try {
    const entries = await readDir(dirPath);

    for (const entry of entries) {
      const fullPath = await join(dirPath, entry.name);

      if (entry.isDirectory) {
        const subResult = await scanDirectory(fullPath);

        totalSize += subResult.totalSize;
        collectedFiles.push(...subResult.collectedFiles);
      } else {
        if (isSupported(entry.name)) {
          const fileStat = await stat(fullPath);
          totalSize += fileStat.size;

          collectedFiles.push({
            path: fullPath,
            name: entry.name,
            size: fileStat.size,
          });
        }
      }
    }
  } catch (err) {
    console.error(`Error scanning ${dirPath}:`, err);
  }

  return { totalSize, collectedFiles };
};

export const processPaths = async (paths) => {
  const items = [];

  for (const path of paths) {
    if (!path) continue;

    try {
      const metadata = await stat(path);
      const name = path.split(/[/\\]/).pop();

      if (metadata.isDirectory) {
        const { totalSize, collectedFiles } = await scanDirectory(path);

        if (collectedFiles.length === 0) {
          console.error('empty');
          continue;
        }

        items.push({
          id: crypto.randomUUID(),
          name: name,
          path: path,
          type: 'folder',
          size: totalSize,
          fileCount: collectedFiles.length,
          files: collectedFiles,
        });
      } else {
        if (isSupported(name)) {
          items.push({
            id: crypto.randomUUID(),
            name: name,
            path: path,
            type: 'file',
            size: metadata.size,
            fileCount: 1,
            files: [{ path: path, name: name, size: metadata.size }],
          });
        }
      }
    } catch (err) {
      console.error(`Failed to process path ${path}:`, err);
    }
  }

  return items;
};
