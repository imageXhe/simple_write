<template>
  <div class="graph-window">
    <!-- 标题栏 -->
    <div class="graph-header" @mousedown="startDrag">
      <span class="graph-header__title">{{ t('common.relationGraph') }}</span>
      <div class="graph-header__actions">
        <!-- 复位按钮 -->
        <a-tooltip :title="t('graph.resetView')" placement="bottom">
          <button class="graph-icon-btn" @click="resetView">
            <AimOutlined />
          </button>
        </a-tooltip>

        <!-- 图谱设置 popover -->
        <a-popover
          v-model:open="showSettingsPanel"
          trigger="click"
          placement="bottom"
          :overlayStyle="{ width: '250px' }"
        >
          <button
            class="graph-toolbar-btn"
            :class="{ 'graph-toolbar-btn--active': showSettingsPanel }"
          >
            <SettingOutlined />
            <span>{{ t('graph.settings') }}</span>
          </button>
          <template #content>
            <div class="graph-popover-content">
              <a-collapse v-model:activeKey="activePanelKeys" :bordered="false" size="small">
                <!-- 筛选 -->
                <a-collapse-panel key="filter">
                  <template #header>
                    <FilterOutlined style="margin-right:6px" />
                    <span>{{ t('graph.filter') }}</span>
                  </template>
                  <div class="settings-row">
                    <span>{{ t('graph.isolatedFiles') }}</span>
                    <a-switch v-model:checked="graphSettings.showIsolated" size="small" @change="onFilterChange" />
                  </div>
                  <div class="settings-row">
                    <span>{{ t('graph.folders') }}</span>
                    <a-switch v-model:checked="graphSettings.showFolders" size="small" @change="onFilterChange" />
                  </div>
                </a-collapse-panel>

                <!-- 颜色组 -->
                <a-collapse-panel key="colorGroup">
                  <template #header>
                    <BgColorsOutlined style="margin-right:6px" />
                    <span>{{ t('graph.colorGroup') }}</span>
                  </template>
                  <div class="color-group-list">
                    <div
                      v-for="group in colorGroups"
                      :key="group.id"
                      class="color-group-row"
                    >
                      <a-space :size="6">
                        <span class="color-dot" :style="{ background: group.color }"></span>
                        <span class="color-group-name">{{ group.name }}</span>
                        <span class="color-group-count">({{ group.fileCount }})</span>
                      </a-space>
                      <CloseOutlined class="color-group-delete" @click="handleDeleteColorGroup(group.id)" />
                    </div>
                    <div v-if="colorGroups.length === 0" class="color-group-empty">
                      {{ t('graph.noColorGroups') }}
                    </div>
                  </div>
                  <div class="color-group-create">
                    <a-space :size="6" direction="vertical" style="width:100%">
                      <a-space :size="6">
                        <div class="native-color-picker-wrapper">
                          <input type="color" v-model="newGroupColor" class="native-color-input" />
                          <span class="color-hex-text">{{ newGroupColor }}</span>
                        </div>
                        <a-input
                          v-model:value="newGroupName"
                          size="small"
                          :placeholder="t('graph.groupName')"
                          style="width:100%"
                          @pressEnter="handleCreateColorGroup"
                        />
                      </a-space>
                      <a-button size="small" type="primary" ghost block @click="handleCreateColorGroup">
                        {{ t('graph.newColorGroup') }}
                      </a-button>
                    </a-space>
                  </div>
                </a-collapse-panel>

                <!-- 外观 -->
                <a-collapse-panel key="appearance">
                  <template #header>
                    <EyeOutlined style="margin-right:6px" />
                    <span>{{ t('graph.appearance') }}</span>
                  </template>
                  <div class="settings-row">
                    <span>{{ t('graph.showArrows') }}</span>
                    <a-switch v-model:checked="graphSettings.showArrows" size="small" @change="onAppearanceChange" />
                  </div>
                  <div class="settings-slider">
                    <div class="slider-label">{{ t('graph.nodeSize') }}: {{ graphSettings.nodeSize }}</div>
                    <a-slider v-model:value="graphSettings.nodeSize" :min="6" :max="30" :step="1" @change="onAppearanceChange" />
                  </div>
                  <div class="settings-slider">
                    <div class="slider-label">{{ t('graph.linkWidth') }}: {{ graphSettings.linkWidth }}</div>
                    <a-slider v-model:value="graphSettings.linkWidth" :min="1" :max="6" :step="0.5" @change="onAppearanceChange" />
                  </div>
                </a-collapse-panel>

                <!-- 力度 -->
                <a-collapse-panel key="force">
                  <template #header>
                    <ThunderboltOutlined style="margin-right:6px" />
                    <span>{{ t('graph.force') }}</span>
                  </template>
                  <div class="settings-slider">
                    <div class="slider-label">{{ t('graph.linkDistance') }}: {{ graphSettings.linkDistance }}</div>
                    <a-slider v-model:value="graphSettings.linkDistance" :min="30" :max="300" :step="10" @change="onForceChange" />
                  </div>
                  <div class="settings-slider">
                    <div class="slider-label">{{ t('graph.centerForce') }}: {{ graphSettings.centerForce.toFixed(2) }}</div>
                    <a-slider v-model:value="graphSettings.centerForce" :min="0.1" :max="1" :step="0.05" @change="onForceChange" />
                  </div>
                  <div class="settings-slider">
                    <div class="slider-label">{{ t('graph.repulsionForce') }}: {{ graphSettings.repulsionForce }}</div>
                    <a-slider v-model:value="graphSettings.repulsionForce" :min="50" :max="500" :step="10" @change="onForceChange" />
                  </div>
                </a-collapse-panel>
              </a-collapse>
            </div>
          </template>
        </a-popover>

        <!-- 最小化 -->
        <button class="graph-icon-btn" @click="minimizeWindow" :title="t('graph.minimize')">
          <MinusOutlined />
        </button>

        <!-- 最大化/还原 -->
        <button class="graph-icon-btn" @click="toggleMaximize" :title="t('graph.maximize')">
          <SwitcherOutlined />
        </button>

        <!-- 关闭 -->
        <button class="graph-close-btn" @click="closeWindow">✕</button>
      </div>
    </div>

    <!-- 图谱画布（右键菜单包裹） -->
    <a-dropdown
      :trigger="['contextmenu']"
      placement="bottomLeft"
      overlayClassName="graph-context-overlay"
    >
      <div class="graph-canvas" ref="canvasRef">
        <svg ref="svgRef"></svg>
      </div>
      <template #overlay>
        <a-menu class="dropdown-menu-bordered" @click="handleContextMenuClick">
          <a-menu-item key="rename" :icon="h(EditOutlined)">
            {{ t('file.rename') }}
          </a-menu-item>
          <a-menu-item key="move" :icon="h(SvgIcon, { raw: svgIcons.move })">
            {{ t('file.move') }}
          </a-menu-item>
          <a-menu-item
            key="favorite"
            :icon="h(isCurrentNodeFavorited ? StarFilled : StarOutlined)"
            :class="{ 'graph-menu-item--favorited': isCurrentNodeFavorited }"
          >
            {{ isCurrentNodeFavorited ? t('file.unfavorite') : t('file.favorite') }}
          </a-menu-item>
          <a-menu-divider />
          <a-sub-menu key="colorGroup" :icon="h(BgColorsOutlined)" :title="t('graph.addToColorGroup')">
            <a-menu-item
              v-for="group in colorGroups"
              :key="group.id"
              @click="handleAddToColorGroup(group.id)"
            >
              <span class="color-dot" :style="{ background: group.color }"></span>
              {{ group.name }}
            </a-menu-item>
            <a-menu-item v-if="colorGroups.length === 0" disabled>
              {{ t('graph.noColorGroups') }}
            </a-menu-item>
          </a-sub-menu>
          <a-menu-item
            v-if="contextNodeId && nodeColorMap[contextNodeId]"
            key="removeColorGroup"
            :icon="h(CloseOutlined)"
            @click="handleRemoveFromColorGroup"
          >
            {{ t('graph.removeFromColorGroup') }}
          </a-menu-item>
          <a-menu-divider />
          <a-menu-item key="delete" danger :icon="h(DeleteOutlined)">
            {{ t('file.delete') }}
          </a-menu-item>
        </a-menu>
      </template>
    </a-dropdown>

    <!-- 重命名弹窗 -->
    <a-modal
      v-model:open="renameModal.open"
      :title="t('file.rename')"
      :ok-text="t('file.confirm')"
      :cancel-text="t('file.cancel')"
      @ok="confirmRename"
    >
      <a-input v-model:value="renameModal.newName" :placeholder="t('file.newName')" />
    </a-modal>

    <!-- 移动弹窗 -->
    <a-modal
      v-model:open="moveModal.open"
      :title="t('file.move')"
      :ok-text="t('file.confirm')"
      :cancel-text="t('file.cancel')"
      @ok="confirmMove"
    >
      <div class="move-path-display">
        <span>{{ t('file.currentPath') }}: {{ moveModal.currentPath }}</span>
      </div>
      <a-input v-model:value="moveModal.newPath" :placeholder="t('file.newPath')" style="margin-top:8px" />
    </a-modal>

    <!-- 删除确认弹窗 -->
    <a-modal
      v-model:open="deleteModal.open"
      :title="t('file.delete')"
      :ok-text="t('file.confirm')"
      :cancel-text="t('file.cancel')"
      ok-type="danger"
      @ok="confirmDelete"
    >
      <p>{{ t('file.confirmDelete') }}</p>
      <p style="font-weight:600;color:#ff4d4f">{{ deleteModal.nodeName }}</p>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, h, onMounted, onUnmounted, nextTick } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';
import { PhysicalPosition } from '@tauri-apps/api/dpi';
import { message } from 'ant-design-vue';
import {
  SettingOutlined, FilterOutlined, BgColorsOutlined,
  EyeOutlined, ThunderboltOutlined, CloseOutlined,
  AimOutlined, MinusOutlined, SwitcherOutlined,
  EditOutlined, StarOutlined, StarFilled, DeleteOutlined,
} from '@ant-design/icons-vue';
import * as d3Force from 'd3-force';
import { select } from 'd3-selection';
import { drag } from 'd3-drag';
import { zoom, zoomIdentity } from 'd3-zoom';
import svgIcons from '../assets/icons';
import SvgIcon from '../components/SvgIcon.vue';
import { useI18n } from '../locales/index.js';
import {
  graphNodes, graphLinks, colorGroups, nodeColorMap, graphSettings,
  visibleNodes, visibleLinks, getNodeColor,
  createColorGroup, deleteColorGroup,
  addNodeToColorGroup, removeNodeFromColorGroup,
  refreshAllGroupCounts,
} from '../stores/graphStore.js';
import { fetchGraphData, loadGraphConfig, saveGraphConfig } from '../menu/graphActions.js';
import { renameFileEntry, moveFileEntry, deleteFileEntry, getWarehouseRootPath } from '../menu/fileActions.js';
import { createFavorite, removeFavorite, fetchFavorites } from '../menu/favoriteActions.js';

const { t } = useI18n();

// DOM 引用
const canvasRef = ref(null);
const svgRef = ref(null);

// 设置面板
const showSettingsPanel = ref(false);
const activePanelKeys = ref(['filter']);

// 收藏列表
const favoritesList = ref([]);

// 右键菜单：当前点击的节点信息
const contextNodeId = ref(null);
const contextNodePath = ref('');
const contextNodeName = ref('');
const contextNodeIsFolder = ref(false);

// 当前右键节点是否已收藏
const isCurrentNodeFavorited = computed(() => {
  if (!contextNodePath.value) return false;
  return favoritesList.value.some(f => f.path === contextNodePath.value);
});

// 随机颜色
const randomHexColor = () => {
  const h = Math.floor(Math.random() * 360);
  const s = 60 + Math.floor(Math.random() * 30);
  const l = 45 + Math.floor(Math.random() * 15);
  const hslToHex = (h, s, l) => {
    s /= 100; l /= 100;
    const a = s * Math.min(l, 1 - l);
    const f = n => {
      const k = (n + h / 30) % 12;
      return Math.round((l - a * Math.max(Math.min(k - 3, 9 - k, 1), -1)) * 255).toString(16).padStart(2, '0');
    };
    return `#${f(0)}${f(8)}${f(4)}`;
  };
  return hslToHex(h, s, l);
};

// 颜色组
const newGroupColor = ref(randomHexColor());
const newGroupName = ref('');

const handleCreateColorGroup = () => {
  const color = newGroupColor.value;
  const name = newGroupName.value.trim() || (typeof color === 'string' ? color : '');
  createColorGroup(name, color);
  saveCurrentConfig();
  newGroupColor.value = randomHexColor();
  newGroupName.value = '';
  message.success(t('message.success'));
};

const handleDeleteColorGroup = (groupId) => {
  deleteColorGroup(groupId);
  saveCurrentConfig();
};

async function saveCurrentConfig() {
  await saveGraphConfig({
    colorGroups: colorGroups.value.map(g => ({ id: g.id, name: g.name, color: g.color })),
    nodeColors: { ...nodeColorMap },
  });
}

// 将完整路径转换为相对路径段数组
const pathToKey = async (fullPath) => {
  const root = await getWarehouseRootPath();
  const normalized = fullPath.replace(/\\/g, '/');
  const rootNormalized = root.replace(/\\/g, '/').replace(/\/$/, '');
  let relative = normalized;
  if (normalized.startsWith(rootNormalized + '/')) {
    relative = normalized.slice(rootNormalized.length + 1);
  } else if (normalized === rootNormalized) {
    relative = '';
  }
  return relative ? relative.split('/') : [];
};

// 拖拽窗口
let dragStart = { x: 0, y: 0 };
const startDrag = (e) => {
  dragStart = { x: e.screenX, y: e.screenY };
  const onMove = async (ev) => {
    const dx = ev.screenX - dragStart.x;
    const dy = ev.screenY - dragStart.y;
    dragStart = { x: ev.screenX, y: ev.screenY };
    const win = getCurrentWindow();
    const pos = await win.outerPosition();
    await win.setPosition(new PhysicalPosition(pos.x + dx, pos.y + dy));
  };
  const onUp = () => {
    document.removeEventListener('mousemove', onMove);
    document.removeEventListener('mouseup', onUp);
    document.body.style.cursor = '';
  };
  document.body.style.cursor = 'move';
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
};

const closeWindow = async () => {
  try { await getCurrentWindow().close(); } catch {}
};

const minimizeWindow = async () => {
  try { await getCurrentWindow().minimize(); } catch {}
};

const toggleMaximize = async () => {
  try { await getCurrentWindow().toggleMaximize(); } catch {}
};

// 弹窗状态
const renameModal = reactive({ open: false, newName: '', nodePath: '' });
const moveModal = reactive({ open: false, newPath: '', currentPath: '', nodePath: '' });
const deleteModal = reactive({ open: false, nodePath: '', nodeName: '' });

// 筛选变化 → 重建仿真
const onFilterChange = () => {
  rebuildSimulation();
};

// 外观变化 → 增量更新外观
const onAppearanceChange = () => {
  updateVisuals();
  if (simulation) {
    simulation.force('collision', d3Force.forceCollide(graphSettings.nodeSize + 4));
    simulation.alpha(0.1).restart();
  }
};

// 力度变化 → 增量更新力参数
const onForceChange = () => {
  if (simulation) {
    simulation.force('link').distance(graphSettings.linkDistance);
    simulation.force('center').strength(graphSettings.centerForce);
    simulation.force('charge').strength(-graphSettings.repulsionForce);
    simulation.alpha(0.3).restart();
  }
};

// 复位视图
const resetView = () => {
  if (!svgRef.value || !zoomBehavior) return;
  const svg = select(svgRef.value);
  svg.transition().duration(300).call(zoomBehavior.transform, zoomIdentity);
};

// 设置右键节点上下文
const setContextNode = (node) => {
  contextNodeId.value = node.id;
  contextNodePath.value = node.path;
  contextNodeName.value = node.name;
  contextNodeIsFolder.value = node.isFolder || false;
};

const clearContextNode = () => {
  contextNodeId.value = null;
  contextNodePath.value = '';
};

// 右键菜单点击分发
const handleContextMenuClick = async ({ key }) => {
  if (!key) return;

  if (key === 'rename') {
    renameModal.nodePath = contextNodePath.value;
    renameModal.newName = contextNodeName.value;
    renameModal.open = true;
  } else if (key === 'move') {
    const pkey = await pathToKey(contextNodePath.value);
    moveModal.nodePath = contextNodePath.value;
    moveModal.currentPath = pkey.join('/');
    moveModal.newPath = moveModal.currentPath;
    moveModal.open = true;
  } else if (key === 'favorite') {
    if (isCurrentNodeFavorited.value) {
      try {
        await removeFavorite({ filePath: contextNodePath.value });
        await loadFavorites();
        message.success(t('file.unfavorite') + ' ' + t('message.success'));
      } catch { message.error(t('message.error')); }
    } else {
      try {
        await createFavorite({ filePath: contextNodePath.value, itemType: contextNodeIsFolder.value ? 'folder' : 'file' });
        await loadFavorites();
        message.success(t('file.favorite') + ' ' + t('message.success'));
      } catch { message.error(t('message.error')); }
    }
  } else if (key === 'delete') {
    deleteModal.nodePath = contextNodePath.value;
    deleteModal.nodeName = contextNodeName.value;
    deleteModal.open = true;
  }
};

const confirmRename = async () => {
  if (!renameModal.newName || renameModal.newName === renameModal.nodePath.split('/').pop()) {
    renameModal.open = false;
    return;
  }
  try {
    const key = await pathToKey(renameModal.nodePath);
    await renameFileEntry({ key, newName: renameModal.newName });
    renameModal.open = false;
    message.success(t('message.success'));
    await loadGraphData();
  } catch { message.error(t('message.error')); }
};

const confirmMove = async () => {
  if (!moveModal.newPath || moveModal.newPath === moveModal.currentPath) {
    moveModal.open = false;
    return;
  }
  try {
    const key = await pathToKey(moveModal.nodePath);
    const newParentPath = moveModal.newPath.includes('/')
      ? moveModal.newPath.substring(0, moveModal.newPath.lastIndexOf('/'))
      : '';
    await moveFileEntry({ key, newParentPath });
    moveModal.open = false;
    message.success(t('message.success'));
    await loadGraphData();
  } catch { message.error(t('message.error')); }
};

const confirmDelete = async () => {
  try {
    const key = await pathToKey(deleteModal.nodePath);
    await deleteFileEntry({ key });
    deleteModal.open = false;
    message.success(t('message.success'));
    await loadGraphData();
  } catch { message.error(t('message.error')); }
};

const handleAddToColorGroup = async (groupId) => {
  if (contextNodeId.value) {
    addNodeToColorGroup(contextNodeId.value, groupId);
    updateNodeColors();
    await saveCurrentConfig();
    message.success(t('message.success'));
  }
};

const handleRemoveFromColorGroup = async () => {
  if (contextNodeId.value) {
    removeNodeFromColorGroup(contextNodeId.value);
    updateNodeColors();
    await saveCurrentConfig();
    message.success(t('message.success'));
  }
};

// 加载收藏列表
async function loadFavorites() {
  try {
    const favs = await fetchFavorites();
    favoritesList.value = favs || [];
  } catch {
    favoritesList.value = [];
  }
}

// ---- D3 力导向图 ----
let simulation = null;
let svgRoot = null;
let zoomBehavior = null;

// 定义箭头 marker
function defineMarkers(svg) {
  svg.select('defs').remove();
  const defs = svg.append('defs');
  defs
    .append('marker')
    .attr('id', 'arrowhead')
    .attr('viewBox', '0 -5 10 10')
    .attr('refX', 16)
    .attr('refY', 0)
    .attr('markerWidth', 6)
    .attr('markerHeight', 6)
    .attr('markerUnits', 'userSpaceOnUse')
    .attr('orient', 'auto')
    .append('path')
    .attr('d', 'M0,-5L10,0L0,5')
    .attr('fill', '#b0b0b0');
}

// 更新节点颜色
function updateNodeColors() {
  if (!svgRoot) return;
  svgRoot.selectAll('circle.node-circle').attr('fill', (d) => {
    if (d.isFolder) return 'none';
    const color = getNodeColor(d.id);
    return color || '#8c8c8c';
  });
}

// 更新外观
function updateVisuals() {
  if (!svgRoot) return;

  svgRoot.selectAll('circle.node-circle')
    .attr('r', graphSettings.nodeSize);

  svgRoot.selectAll('line.graph-link')
    .attr('stroke-width', graphSettings.linkWidth)
    .attr('marker-end', graphSettings.showArrows ? 'url(#arrowhead)' : null);

  // 更新标签偏移
  svgRoot.selectAll('text.node-label')
    .attr('dy', graphSettings.nodeSize + 14);
}

// 构建/重建力仿真
function rebuildSimulation() {
  if (!svgRef.value) return;

  const svg = select(svgRef.value);
  const width = canvasRef.value.clientWidth;
  const height = canvasRef.value.clientHeight;

  svg.selectAll('*').remove();
  defineMarkers(svg);

  svgRoot = svg.append('g').attr('class', 'zoom-group');

  zoomBehavior = zoom()
    .scaleExtent([0.1, 4])
    .on('zoom', (event) => {
      svgRoot.attr('transform', event.transform);
    });
  svg.call(zoomBehavior);

  const nodes = visibleNodes.value.map(n => ({ ...n }));
  const links = visibleLinks.value.map(l => ({ ...l }));

  if (nodes.length === 0) {
    svg.append('text')
      .attr('x', width / 2)
      .attr('y', height / 2)
      .attr('text-anchor', 'middle')
      .attr('fill', '#bfbfbf')
      .attr('font-size', '14')
      .text(t('graph.noNodesToDisplay'));
    return;
  }

  // 初始位置：均匀环形分布
  const cx = width / 2;
  const cy = height / 2;
  const radius = Math.min(width, height) / 5;
  nodes.forEach((n, i) => {
    const angle = (2 * Math.PI * i) / nodes.length;
    n.x = cx + radius * Math.cos(angle);
    n.y = cy + radius * Math.sin(angle);
  });

  if (simulation) {
    simulation.stop();
  }

  simulation = d3Force.forceSimulation(nodes)
    .alpha(0.5)
    .alphaDecay(0.01)
    .force('link', d3Force.forceLink(links).id(d => d.id).distance(graphSettings.linkDistance))
    .force('charge', d3Force.forceManyBody().strength(-graphSettings.repulsionForce))
    .force('center', d3Force.forceCenter(cx, cy).strength(graphSettings.centerForce))
    .force('collision', d3Force.forceCollide(graphSettings.nodeSize + 4));

  // 连线
  const linkGroup = svgRoot.append('g').attr('class', 'links');

  const linkElements = linkGroup.selectAll('line')
    .data(links)
    .enter()
    .append('line')
    .attr('class', 'graph-link')
    .attr('stroke', '#c0c0c0')
    .attr('stroke-width', graphSettings.linkWidth)
    .attr('stroke-opacity', 0.7)
    .attr('marker-end', graphSettings.showArrows ? 'url(#arrowhead)' : null);

  // 节点组
  const nodeGroup = svgRoot.append('g').attr('class', 'nodes');

  const nodeElements = nodeGroup.selectAll('g.node')
    .data(nodes, d => d.id)
    .enter()
    .append('g')
    .attr('class', 'node')
    .call(
      drag()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on('drag', (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        })
    );

  // 节点圆：文件实心，文件夹空心
  nodeElements
    .append('circle')
    .attr('class', 'node-circle')
    .attr('r', graphSettings.nodeSize)
    .attr('fill', d => {
      if (d.isFolder) return 'none';
      const color = getNodeColor(d.id);
      return color || '#8c8c8c';
    })
    .attr('stroke', d => d.isFolder ? '#8c8c8c' : '#fff')
    .attr('stroke-width', d => d.isFolder ? 2 : 2)
    .attr('cursor', 'pointer');

  // 节点标签
  nodeElements
    .append('text')
    .attr('class', 'node-label')
    .attr('dy', graphSettings.nodeSize + 14)
    .attr('text-anchor', 'middle')
    .attr('fill', '#595959')
    .attr('font-size', '11')
    .attr('pointer-events', 'none')
    .text(d => d.name.length > 16 ? d.name.slice(0, 15) + '...' : d.name);

  // 点击文件/文件夹节点 → 打开文件
  nodeElements
    .on('click', async (event, d) => {
      event.stopPropagation();
      if (!d.isFolder) {
        try {
          await emit('open-file-from-graph', { path: d.path, name: d.name });
        } catch {}
      }
    });

  // 右键节点 → 设置上下文，让事件冒泡到 a-dropdown
  nodeElements
    .on('contextmenu', (event, d) => {
      setContextNode(d);
      // 不阻止冒泡，让 a-dropdown 收到 contextmenu 事件
    });

  // 右键空白区域 → 阻止冒泡，不弹出菜单
  svg.on('contextmenu', (event) => {
    const clickedOnNode = event.target.closest('.node');
    if (!clickedOnNode) {
      clearContextNode();
      event.stopPropagation();
      event.preventDefault();
    }
  });

  // 仿真 tick
  simulation.on('tick', () => {
    linkElements
      .attr('x1', d => d.source.x)
      .attr('y1', d => d.source.y)
      .attr('x2', d => d.target.x)
      .attr('y2', d => d.target.y);

    // 更新所有节点位置（文件 + 文件夹）
    nodeElements
      .attr('transform', d => `translate(${d.x},${d.y})`);
  });
}

// 加载图谱数据
async function loadGraphData() {
  try {
    const { nodes, links } = await fetchGraphData();

    const oldPositions = {};
    for (const n of graphNodes.value) {
      if (n.x !== undefined) oldPositions[n.id] = { x: n.x, y: n.y };
    }

    for (const n of nodes) {
      if (oldPositions[n.id]) {
        n.x = oldPositions[n.id].x;
        n.y = oldPositions[n.id].y;
      }
    }

    graphNodes.value = nodes;
    graphLinks.value = links;

    const config = await loadGraphConfig();
    if (config) {
      colorGroups.value = (config.colorGroups || []).map(g => ({
        ...g,
        fileCount: 0,
      }));

      for (const key of Object.keys(nodeColorMap)) {
        delete nodeColorMap[key];
      }
      for (const [nodeId, groupId] of Object.entries(config.nodeColors || {})) {
        nodeColorMap[nodeId] = groupId;
      }

      refreshAllGroupCounts();
    }

    await nextTick();
    rebuildSimulation();
  } catch (e) {
    console.error('加载图谱数据失败:', e);
  }
}

let resizeObserver = null;

onMounted(async () => {
  await loadGraphData();
  await loadFavorites();

  if (canvasRef.value) {
    resizeObserver = new ResizeObserver(() => {
      if (svgRef.value && canvasRef.value) {
        select(svgRef.value)
          .attr('width', canvasRef.value.clientWidth)
          .attr('height', canvasRef.value.clientHeight);
        rebuildSimulation();
      }
    });
    resizeObserver.observe(canvasRef.value);
  }
});

onUnmounted(() => {
  if (simulation) {
    simulation.stop();
  }
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});
</script>

<style scoped>
.graph-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #fff;
  font-family: -apple-system, sans-serif;
  border-radius: 8px;
  overflow: hidden;
}

/* 标题栏 */
.graph-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: #fafafa;
  border-bottom: 1px solid #f0f0f0;
  cursor: move;
  user-select: none;
  flex-shrink: 0;
}

.graph-header__title {
  font-size: 13px;
  font-weight: 600;
  color: #1f1f1f;
}

.graph-header__actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.graph-toolbar-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 28px;
  padding: 0 10px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  font-size: 12px;
  color: #595959;
  transition: all 0.15s;
}

.graph-toolbar-btn:hover {
  border-color: #1890ff;
  color: #1890ff;
}

.graph-toolbar-btn--active {
  border-color: #1890ff;
  color: #1890ff;
  background: #e6f7ff;
}

.graph-icon-btn {
  width: 28px;
  height: 28px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  background: #fff;
  cursor: pointer;
  color: #595959;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  transition: all 0.15s;
}

.graph-icon-btn:hover {
  border-color: #1890ff;
  color: #1890ff;
}

.graph-close-btn {
  border: none;
  background: transparent;
  cursor: pointer;
  color: #8c8c8c;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 14px;
}

.graph-close-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: #1f1f1f;
}

/* Popover 内容 */
.graph-popover-content {
  max-height: 60vh;
  overflow-y: auto;
}

.graph-popover-content :deep(.ant-collapse) {
  background: transparent;
}

.graph-popover-content :deep(.ant-collapse-header) {
  font-size: 13px;
  padding: 8px 12px !important;
}

.graph-popover-content :deep(.ant-collapse-content-box) {
  padding: 8px 12px !important;
}

.settings-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
  font-size: 12px;
  color: #595959;
}

.settings-slider {
  padding: 4px 0;
}

.slider-label {
  font-size: 12px;
  color: #8c8c8c;
  margin-bottom: 2px;
}

/* 颜色组 */
.color-group-list {
  margin-bottom: 8px;
}

.color-group-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
  font-size: 12px;
}

.color-group-name {
  color: #262626;
}

.color-group-count {
  font-size: 11px;
  color: #bfbfbf;
}

.color-group-empty {
  font-size: 12px;
  color: #bfbfbf;
  padding: 4px 0;
}

.color-group-delete {
  font-size: 11px;
  color: #bfbfbf;
  cursor: pointer;
  transition: color 0.15s;
}

.color-group-delete:hover {
  color: #ff4d4f;
}

.color-group-create {
  padding: 4px 0;
}

.native-color-picker-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
}

.native-color-input {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  padding: 0;
  background: transparent;
}

.native-color-input::-webkit-color-swatch-wrapper {
  padding: 0;
}

.native-color-input::-webkit-color-swatch {
  border: 1px solid #d9d9d9;
  border-radius: 4px;
}

.color-hex-text {
  font-size: 12px;
  color: #8c8c8c;
  font-family: monospace;
}

.color-dot {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid rgba(0, 0, 0, 0.15);
  flex-shrink: 0;
}

/* 画布 */
.graph-canvas {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.graph-canvas svg {
  width: 100%;
  height: 100%;
  display: block;
}

.move-path-display {
  font-size: 12px;
  color: #8c8c8c;
  word-break: break-all;
}
</style>

<!-- 全局样式：dropdown overlay 渲染到 body -->
<style>
.graph-context-overlay .graph-menu-item--favorited {
  color: #faad14 !important;
}

/* 右键菜单中颜色组圆点 */
.graph-context-overlay .color-dot {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid rgba(0, 0, 0, 0.15);
  flex-shrink: 0;
  vertical-align: middle;
  margin-right: 4px;
}
</style>
