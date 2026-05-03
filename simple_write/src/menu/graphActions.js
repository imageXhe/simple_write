import { invoke } from "@tauri-apps/api/core";
import { getWarehouseRootPath, fetchFileTree } from "./fileActions";
import { scanLinks } from "./linkActions";
import { fileData, warehousePath, flattenTree } from "../stores/fileStore";
import { exportForceSettings, importForceSettings } from "../stores/graphStore";

// 从文件树和链接数据构建图谱节点和连线
export async function fetchGraphData() {
  // 图谱窗口是独立 webview，需先加载文件树
  const { warehousePath: root, fileData: tree } = await fetchFileTree();
  warehousePath.value = root;
  fileData.value = tree;

  // 获取链接数据
  const linkEntries = await scanLinks();

  // 展平文件树
  const flatEntries = flattenTree(fileData.value, root);

  // 构建节点
  const nodes = [];
  const nodeMap = {};

  for (const entry of flatEntries) {
    const id = entry.isFolder
      ? 'folder:' + entry.path
      : 'file:' + entry.path;

    // 计算父文件夹的完整路径（path 去除文件名后的部分）
    const normalized = entry.path.replace(/\\/g, '/');
    const lastSlash = normalized.lastIndexOf('/');
    const parentFolder = lastSlash > 0 ? normalized.substring(0, lastSlash) : null;

    const node = {
      id,
      name: entry.name,
      path: entry.path,
      isFolder: entry.isFolder,
      parentFolder,
    };

    nodes.push(node);
    nodeMap[entry.path] = id;
  }

  // 构建连线
  const links = [];
  const linkSet = new Set();

  for (const entry of linkEntries) {
    const sourcePath = entry.source;
    const sourceId = nodeMap[sourcePath];
    if (!sourceId) continue;

    for (const targetPath of entry.targets) {
      const targetId = nodeMap[targetPath];
      if (!targetId) continue;

      // 去重（双向视为同一条连线）
      const key = [sourceId, targetId].sort().join('|');
      if (linkSet.has(key)) continue;
      linkSet.add(key);

      links.push({
        source: sourceId,
        target: targetId,
      });
    }
  }

  return { nodes, links };
}

// 从仓库目录读取图谱配置（颜色组、力度参数等）
export async function loadGraphConfig() {
  try {
    const warehouseRoot = await getWarehouseRootPath();
    const config = await invoke("read_graph_config", { warehousePath: warehouseRoot });
    // 恢复力度参数
    if (config && config.forceSettings) {
      importForceSettings(config.forceSettings);
    }
    return config;
  } catch {
    return { colorGroups: [], nodeColors: {} };
  }
}

// 保存图谱配置到仓库目录
export async function saveGraphConfig(config) {
  const warehouseRoot = await getWarehouseRootPath();
  // 附加力度参数
  const fullConfig = {
    ...config,
    forceSettings: exportForceSettings(),
  };
  return invoke("write_graph_config", {
    warehousePath: warehouseRoot,
    config: JSON.stringify(fullConfig),
  });
}
