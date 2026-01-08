import { invoke } from '@tauri-apps/api/core';

export async function processPaths(paths) {
  try {
    const fileNodes = await invoke('scan_dropped_paths', { paths });
    return transformToUiFormat(fileNodes);
  } catch (error) {
    console.error('Scan failed:', error);
    return [];
  }
}

function transformToUiFormat(nodes) {
  return nodes.map((node) => ({
    path: node.path,
    name: node.name,
    type: node.is_dir ? 'folder' : 'file',
    size: node.size,
    fileCount: node.file_count,
    children: node.children ? transformToUiFormat(node.children) : [],
    id: node.path,
  }));
}
