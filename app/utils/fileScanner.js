import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export async function processPaths(paths, onProgress) {
  const unlisten = await listen('scan_progress', (event) => {
    onProgress(Number(event.payload) || 0);
  });
  try {
    const fileNodes = await invoke('scan_dropped_paths', { paths });
    return transformToUiFormat(fileNodes);
  } finally {
    unlisten();
  }
}

function transformToUiFormat(nodes) {
  return nodes.map((node) => ({
    path: node.path,
    name: node.name,
    extension: node.extension,
    type: node.is_dir ? 'folder' : 'file',
    size: node.size,
    fileCount: node.file_count,
    children: node.children ? transformToUiFormat(node.children) : [],
    id: node.path,
  }));
}
