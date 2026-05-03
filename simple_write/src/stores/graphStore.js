import { ref, reactive, computed } from 'vue';

// 图谱节点和连线原始数据
export const graphNodes = ref([]);
export const graphLinks = ref([]);

// 颜色组列表 { id, name, color, fileCount }
export const colorGroups = ref([]);

// 节点ID -> 颜色组ID 的映射
export const nodeColorMap = reactive({});

// 图谱设置
export const graphSettings = reactive({
  // 筛选
  showIsolated: true,    // 显示孤立文件
  showFolders: false,    // 显示文件夹大圆

  // 外观
  showArrows: true,      // 显示连线箭头
  nodeSize: 10,          // 文件节点圆形半径 (6~30)
  linkWidth: 2,          // 连线粗细 (1~6)

  // 力度
  linkDistance: 60,      // 连线长度 (30~300)
  centerForce: 0.3,      // 向心力 (0.1~1)
  repulsionForce: 60,   // 排斥力 (50~500)
});

// 根据筛选条件计算可见节点
export const visibleNodes = computed(() => {
  const nodes = graphNodes.value;
  const links = graphLinks.value;

  // 计算每个节点的度数（有连线数）
  const degreeMap = {};
  for (const link of links) {
    const srcId = typeof link.source === 'object' ? link.source.id : link.source;
    const tgtId = typeof link.target === 'object' ? link.target.id : link.target;
    degreeMap[srcId] = (degreeMap[srcId] || 0) + 1;
    degreeMap[tgtId] = (degreeMap[tgtId] || 0) + 1;
  }

  return nodes.filter(node => {
    // 文件夹节点：只在 showFolders 开启时显示
    if (node.isFolder) {
      return graphSettings.showFolders;
    }

    // 文件节点
    const degree = degreeMap[node.id] || 0;

    // 如果两个开关都关闭，只显示有链接的文件
    if (!graphSettings.showIsolated && !graphSettings.showFolders) {
      return degree > 0;
    }

    // 如果关闭了孤立文件开关，过滤掉无链接的文件
    if (!graphSettings.showIsolated && degree === 0) {
      return false;
    }

    return true;
  });
});

// 根据可见节点过滤连线
export const visibleLinks = computed(() => {
  const visibleIds = new Set(visibleNodes.value.map(n => n.id));
  return graphLinks.value.filter(link => {
    const srcId = typeof link.source === 'object' ? link.source.id : link.source;
    const tgtId = typeof link.target === 'object' ? link.target.id : link.target;
    return visibleIds.has(srcId) && visibleIds.has(tgtId);
  });
});

// 获取节点的颜色（根据颜色组）
export function getNodeColor(nodeId) {
  const groupId = nodeColorMap[nodeId];
  if (groupId) {
    const group = colorGroups.value.find(g => g.id === groupId);
    if (group) return group.color;
  }
  return null; // null 表示使用默认颜色
}

// 创建颜色组
export function createColorGroup(name, color) {
  const id = 'cg_' + Date.now() + '_' + Math.random().toString(36).slice(2, 8);
  colorGroups.value.push({ id, name, color, fileCount: 0 });
  return id;
}

// 删除颜色组（并将组内所有节点退出）
export function deleteColorGroup(groupId) {
  // 移除所有节点的颜色映射
  for (const nodeId of Object.keys(nodeColorMap)) {
    if (nodeColorMap[nodeId] === groupId) {
      delete nodeColorMap[nodeId];
    }
  }
  // 删除颜色组
  const idx = colorGroups.value.findIndex(g => g.id === groupId);
  if (idx !== -1) {
    colorGroups.value.splice(idx, 1);
  }
}

// 将节点添加到颜色组
export function addNodeToColorGroup(nodeId, groupId) {
  const oldGroupId = nodeColorMap[nodeId];
  if (oldGroupId) {
    updateGroupFileCount(oldGroupId);
  }
  nodeColorMap[nodeId] = groupId;
  updateGroupFileCount(groupId);
}

// 将节点退出颜色组
export function removeNodeFromColorGroup(nodeId) {
  const groupId = nodeColorMap[nodeId];
  if (groupId) {
    delete nodeColorMap[nodeId];
    updateGroupFileCount(groupId);
  }
}

// 更新颜色组的文件计数
function updateGroupFileCount(groupId) {
  const group = colorGroups.value.find(g => g.id === groupId);
  if (group) {
    group.fileCount = Object.values(nodeColorMap).filter(gid => gid === groupId).length;
  }
}

// 更新所有颜色组的文件计数
export function refreshAllGroupCounts() {
  for (const group of colorGroups.value) {
    group.fileCount = Object.values(nodeColorMap).filter(gid => gid === group.id).length;
  }
}

// 导出力度参数（用于持久化）
export function exportForceSettings() {
  return {
    linkDistance: graphSettings.linkDistance,
    centerForce: graphSettings.centerForce,
    repulsionForce: graphSettings.repulsionForce,
  };
}

// 导入力度参数（从持久化恢复）
export function importForceSettings(settings) {
  if (!settings) return;
  if (typeof settings.linkDistance === 'number') graphSettings.linkDistance = settings.linkDistance;
  if (typeof settings.centerForce === 'number') graphSettings.centerForce = settings.centerForce;
  if (typeof settings.repulsionForce === 'number') graphSettings.repulsionForce = settings.repulsionForce;
}
