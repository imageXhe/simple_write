<template>
    <div class="graph-window">
        <!-- 标题栏 -->
        <div class="graph-header" @mousedown="startDrag">
        <span class="graph-header__title">{{ t('common.relationGraph') }}</span>
        <div class="graph-header__actions">
            <!-- 布局方向切换 -->
            <a-tooltip :title="directionLabel" placement="bottom">
            <button class="graph-icon-btn" @click="toggleDirection">
                <SwapOutlined />
            </button>
            </a-tooltip>

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
            :overlayStyle="{ width: '220px' }"
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
                <div class="settings-row">
                    <span>文件夹颜色</span>
                    <input type="color" v-model="graphStyle.folderNodeColor" class="native-color-input" @change="saveStyle" />
                </div>
                <div class="settings-row">
                    <span>文件颜色</span>
                    <input type="color" v-model="graphStyle.fileNodeColor" class="native-color-input" @change="saveStyle" />
                </div>
                <div class="settings-row">
                    <span>连线颜色</span>
                    <input type="color" v-model="graphStyle.edgeColor" class="native-color-input" @change="saveStyle" />
                </div>
                <div class="settings-row">
                    <span>显示箭头</span>
                    <a-switch v-model:checked="graphStyle.showArrows" size="small" @change="saveStyle" />
                </div>
                <div class="settings-slider">
                    <div class="slider-label">层级间距: {{ graphStyle.levelGap }}</div>
                    <a-slider v-model:value="graphStyle.levelGap" :min="40" :max="200" :step="10" @change="saveStyle" />
                </div>
                <div class="settings-slider">
                    <div class="slider-label">节点间距: {{ graphStyle.nodeGap }}</div>
                    <a-slider v-model:value="graphStyle.nodeGap" :min="20" :max="120" :step="5" @change="saveStyle" />
                </div>
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

        <!-- 图谱画布 -->
        <div class="graph-canvas" ref="canvasRef" @click="closeNodeDetail">
        <svg ref="svgRef"></svg>

        <!-- 侧边详情面板 -->
        <div v-if="nodeDetail.visible" class="graph-node-detail" :style="nodeDetailStyle">
            <div class="graph-node-detail__header">
            <span>关联文件</span>
            <button class="graph-icon-btn" @click="nodeDetail.visible = false">✕</button>
            </div>
            <div v-if="nodeDetail.links.length === 0" class="graph-node-detail__empty">无关联文件</div>
            <div
            v-for="link in nodeDetail.links"
            :key="link"
            class="graph-node-detail__item"
            @click="openLinkedFile(link)"
            >
            {{ link.split('/').pop() || link }}
            </div>
        </div>
        </div>
    </div>
</template>

<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, nextTick, h } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emit } from "@tauri-apps/api/event";
import { PhysicalPosition } from "@tauri-apps/api/dpi";
import { message } from "ant-design-vue";
import {
    SettingOutlined, AimOutlined, MinusOutlined, SwitcherOutlined, SwapOutlined,
} from "@ant-design/icons-vue";
import { select } from "d3-selection";
import { zoom, zoomIdentity } from "d3-zoom";
import { useI18n } from "../locales/index.js";
import { flattenTree, fileData, warehousePath } from "../stores/fileStore";
import { scanLinks, getOutgoingLinks, getIncomingLinks } from "../menu/linkActions";
import { getWarehouseRootPath } from "../menu/fileActions";
import { readNovelConfig, writeNovelConfig } from "../menu/novelActions";

const { t } = useI18n();

// DOM 引用
const canvasRef = ref(null);
const svgRef = ref(null);

// 设置
const showSettingsPanel = ref(false);
const graphDirection = ref("top-to-bottom");

// 图谱样式
const graphStyle = reactive({
    folderNodeColor: "#69c0ff",
    folderNodeBorder: "#1890ff",
    folderNodeFontSize: 14,
    fileNodeColor: "#ffffff",
    fileNodeBorder: "#d9d9d9",
    fileNodeFontSize: 12,
    edgeColor: "#bfbfbf",
    edgeWidth: 1,
    showArrows: true,
    nodeGap: 40,
    levelGap: 80,
});

let svg;
let zoomInstance;
let expandedFolders = new Set();
let currentTransform = zoomIdentity;
let shouldResetView = false;

// 节点详情面板
const nodeDetail = reactive({
    visible: false,
    x: 0,
    y: 0,
    fileName: "",
    filePath: "",
    links: [],
});

const nodeDetailStyle = ref({});

// 加载样式配置
async function loadGraphStyle() {
    try {
        const config = await readNovelConfig();
        if (config?.graphStyle) {
            Object.assign(graphStyle, config.graphStyle);
            graphDirection.value = config.graphStyle.direction || "top-to-bottom";
        }
    } catch { /* 使用默认值 */ }
}

async function saveStyle() {
    try {
        const config = (await readNovelConfig()) || {};
        config.graphStyle = { ...graphStyle, direction: graphDirection.value };
        await writeNovelConfig(config);
        renderGraph();
    } catch { /* 静默失败 */ }
}

const directionLabel = computed(() => {
    return graphDirection.value === "top-to-bottom" ? "切换：左右布局" : "切换：上下布局";
});

function toggleDirection() {
    graphDirection.value = graphDirection.value === "top-to-bottom" ? "left-to-right" : "top-to-bottom";
    shouldResetView = true;
    saveStyle();
}

// 路径归一化辅助函数：统一为正斜杠，去除末尾斜杠
function normPath(p) {
    return (p || "").replace(/\\/g, "/").replace(/\/+$/, "") || "/";
}

// 构建树结构
function buildTreeData() {
    const all = flattenTree(fileData.value, warehousePath.value);
    const whPathNorm = normPath(warehousePath.value);
    const wsLen = whPathNorm.split("/").filter(Boolean).length;

    const root = { name: "仓库", children: [], path: warehousePath.value, isFolder: true, depth: 0 };
    const nodeMap = new Map();
    // 使用归一化路径作为键，确保查找一致
    nodeMap.set(whPathNorm, root);

    for (const entry of all) {
        const entryPathNorm = normPath(entry.path);
        const segs = entryPathNorm.split("/").filter(Boolean);
        const depth = segs.length - wsLen - 1;
        if (depth < 0) continue;

        // 查找父节点：使用归一化路径直接查找
        const parentPathNorm = segs.slice(0, -1).join("/") || "/";
        let parent = nodeMap.get(parentPathNorm);

        const node = {
            name: entry.name,
            children: [],
            path: entry.path,
            isFolder: entry.isFolder,
            depth: depth + 1,
            collapsed: false,
        };

        if (parent) {
            parent.children.push(node);
        } else {
            root.children.push(node);
        }
        if (entry.isFolder) {
            // 使用归一化路径作为键存储
            nodeMap.set(entryPathNorm, node);
        }
    }

    return root;
}

// 获取展开后的可见节点
function getVisibleNodes(root) {
    const nodes = [];
    const links = [];
    function walk(node, parent) {
        nodes.push(node);
        if (parent) {
            links.push({ source: parent, target: node });
        }
        if (node.isFolder && !expandedFolders.has(normPath(node.path)) && node !== root) return;
        if (node.children) {
            for (const child of node.children) {
                walk(child, node);
            }
        }
    }
    walk(root, null);
    return { nodes, links };
}

// 手动布局：为每个节点计算 (x, y) 坐标
function computeNodePositions(root) {
    const nodeH = 32;
    const nodeW = 120;
    const levelGap = graphStyle.levelGap;
    const nodeGap = graphStyle.nodeGap;
    const visibleNodes = [];
    const collectVisible = (node) => {
        visibleNodes.push(node);
        if (node.isFolder && node !== root && !expandedFolders.has(normPath(node.path))) return;
        for (const child of node.children || []) collectVisible(child);
    };
    collectVisible(root);

    const visibleChildrenOf = (node) => {
        if (node.isFolder && node !== root && !expandedFolders.has(normPath(node.path))) return [];
        return node.children || [];
    };

    let laneIndex = 0;
    const assignTopToBottom = (node, depth) => {
        const children = visibleChildrenOf(node);
        if (children.length === 0) {
            node.layoutX = laneIndex * (nodeW + nodeGap);
            node.layoutY = depth * (nodeH + levelGap);
            laneIndex += 1;
            return;
        }
        for (const child of children) {
            assignTopToBottom(child, depth + 1);
        }
        const first = children[0];
        const last = children[children.length - 1];
        node.layoutX = (first.layoutX + last.layoutX) / 2;
        node.layoutY = depth * (nodeH + levelGap);
    };

    const assignLeftToRight = (node, depth) => {
        const children = visibleChildrenOf(node);
        if (children.length === 0) {
            node.layoutX = depth * (nodeW + levelGap);
            node.layoutY = laneIndex * (nodeH + nodeGap);
            laneIndex += 1;
            return;
        }
        for (const child of children) {
            assignLeftToRight(child, depth + 1);
        }
        const first = children[0];
        const last = children[children.length - 1];
        node.layoutX = depth * (nodeW + levelGap);
        node.layoutY = (first.layoutY + last.layoutY) / 2;
    };

    if (graphDirection.value === "top-to-bottom") assignTopToBottom(root, 0);
    else assignLeftToRight(root, 0);

    // 防御性检查：确保所有可见节点都有有效坐标
    const xValues = visibleNodes.map((node) => node.layoutX).filter((v) => typeof v === "number" && !isNaN(v));
    const yValues = visibleNodes.map((node) => node.layoutY).filter((v) => typeof v === "number" && !isNaN(v));
    if (xValues.length === 0 || yValues.length === 0) return;
    const minX = Math.min(...xValues);
    const maxX = Math.max(...xValues);
    const centerShiftX = (minX + maxX) / 2;
    const minY = Math.min(...yValues);
    const maxY = Math.max(...yValues);
    const centerShiftY = (minY + maxY) / 2;
    for (const node of visibleNodes) {
        if (typeof node.layoutX === "number" && !isNaN(node.layoutX)) {
            node.layoutX -= centerShiftX;
        }
        if (typeof node.layoutY === "number" && !isNaN(node.layoutY)) {
            node.layoutY -= centerShiftY;
        }
    }
}

// 渲染图谱
async function renderGraph() {
    if (!svgRef.value || !canvasRef.value) return;
    // 数据未加载时不渲染，避免使用空数据构建错误的树结构
    if (!warehousePath.value || !fileData.value || fileData.value.length === 0) return;
    const w = canvasRef.value.clientWidth;
    const h = canvasRef.value.clientHeight;

    svg = select(svgRef.value);
    svg.selectAll("*").remove();
    svg.attr("width", w).attr("height", h);

    const treeData = buildTreeData();
    computeNodePositions(treeData);

    // 收集所有可见节点和连线
    const visibleNodes = [];
    const visibleLinks = [];
    const visited = new Set();

    function collect(node, parent) {
        const np = normPath(node.path);
        if (visited.has(np)) return;
        visited.add(np);
        visibleNodes.push(node);
        if (parent) {
            visibleLinks.push({ source: parent, target: node });
        }
        if (node.isFolder && node !== treeData && !expandedFolders.has(np)) return;
        for (const child of node.children || []) {
            collect(child, node);
        }
    }
    collect(treeData, null);

    if (visibleNodes.length === 0) return;

    const nodeH = 32;
    const nodeW = 120;

    const g = svg.append("g");
    const nodeX = (node) => node.layoutX ?? 0;
    const nodeY = (node) => node.layoutY ?? 0;

    // 连线
    const linkGroup = g.append("g").attr("class", "links");
    for (const link of visibleLinks) {
        const s = link.source; const t = link.target;
        let x1, y1, x2, y2;
        if (graphDirection.value === "left-to-right") {
            x1 = s.layoutX + nodeW; y1 = s.layoutY + nodeH / 2;
            x2 = t.layoutX; y2 = t.layoutY + nodeH / 2;
        } else {
            x1 = s.layoutX + nodeW / 2; y1 = s.layoutY + nodeH;
            x2 = t.layoutX + nodeW / 2; y2 = t.layoutY;
        }
        linkGroup.append("line")
            .attr("x1", x1).attr("y1", y1).attr("x2", x2).attr("y2", y2)
            .attr("stroke", graphStyle.edgeColor)
            .attr("stroke-width", graphStyle.edgeWidth)
            .attr("marker-end", graphStyle.showArrows ? "url(#arrowhead)" : null);
    }

    // 箭头标记
    if (graphStyle.showArrows) {
        svg.append("defs").append("marker")
            .attr("id", "arrowhead")
            .attr("viewBox", "0 0 10 7")
            .attr("refX", 10).attr("refY", 3.5)
            .attr("markerWidth", 8).attr("markerHeight", 6)
            .attr("orient", "auto")
            .append("polygon")
            .attr("points", "0 0, 10 3.5, 0 7")
            .attr("fill", graphStyle.edgeColor);
    }

    // 节点
    const nodeGroup = g.append("g").attr("class", "nodes");
    for (const node of visibleNodes) {
        const x = nodeX(node);
        const y = nodeY(node);

        const ng = nodeGroup.append("g").attr("transform", `translate(${x},${y})`).style("cursor", "pointer");

        ng.append("rect")
            .attr("width", nodeW).attr("height", nodeH)
            .attr("rx", 6).attr("ry", 6)
            .attr("fill", node.isFolder ? graphStyle.folderNodeColor : graphStyle.fileNodeColor)
            .attr("stroke", node.isFolder ? graphStyle.folderNodeBorder : graphStyle.fileNodeBorder)
            .attr("stroke-width", 1.5);

        ng.append("text")
            .attr("x", nodeW / 2).attr("y", nodeH / 2 + 5)
            .attr("text-anchor", "middle")
            .attr("fill", "#262626")
            .attr("font-size", node.isFolder ? graphStyle.folderNodeFontSize : graphStyle.fileNodeFontSize)
            .text(node.name.length > 12 ? node.name.slice(0, 11) + "…" : node.name);

        ng.on("click", (e) => {
            e.stopPropagation();
            if (node.isFolder && node !== treeData) {
                const np = normPath(node.path);
                if (expandedFolders.has(np)) {
                    expandedFolders.delete(np);
                } else {
                    expandedFolders.add(np);
                }
                renderGraph();
            } else if (!node.isFolder) {
                showNodeDetail(node, e);
            }
        });

        ng.on("dblclick", (e) => {
            e.stopPropagation();
            if (!node.isFolder) {
                emit("open-file-from-graph", { path: node.path, name: node.name }).catch(() => {});
            }
        });
    }

    // 缩放
    zoomInstance = zoom()
        .scaleExtent([0.2, 3])
        .on("zoom", (event) => {
            currentTransform = event.transform;
            g.attr("transform", event.transform);
        });
    svg.call(zoomInstance);

    // 初始居中 / 保留当前视图位置
    try {
        const shouldFit = shouldResetView || (currentTransform.k === 1 && currentTransform.x === 0 && currentTransform.y === 0);
        if (shouldFit) {
            const bounds = g.node().getBBox();
            const scale = Math.min(w / (bounds.width + 100), h / (bounds.height + 100), 1);
            const tx = w / 2 - bounds.x * scale - bounds.width / 2 * scale;
            const ty = h / 2 - bounds.y * scale - bounds.height / 2 * scale;
            currentTransform = zoomIdentity.translate(tx, ty).scale(scale);
            shouldResetView = false;
        }
        svg.call(zoomInstance.transform, currentTransform);
    } catch { /* 忽略空图 */ }
}

// 节点详情
async function showNodeDetail(node, event) {
    const canvasRect = canvasRef.value.getBoundingClientRect();
    nodeDetail.x = event.clientX - canvasRect.left + 10;
    nodeDetail.y = event.clientY - canvasRect.top + 10;
    nodeDetail.fileName = node.name;
    nodeDetail.filePath = node.path;
    nodeDetail.visible = true;
    nodeDetail.links = [];
    nodeDetailStyle.value = { left: nodeDetail.x + "px", top: nodeDetail.y + "px" };

    try {
        await scanLinks();
        const [outgoing, incoming] = await Promise.all([
            getOutgoingLinks(node.path),
            getIncomingLinks(node.path),
        ]);
        nodeDetail.links = [...new Set([...outgoing, ...incoming])];
    } catch { /* 静默失败 */ }
}

function closeNodeDetail() {
    nodeDetail.visible = false;
}

function openLinkedFile(filePath) {
    const name = filePath.split("/").filter(Boolean).pop() || filePath;
    emit("open-file-from-graph", { path: filePath, name }).catch(() => {});
}

function resetView() {
    currentTransform = zoomIdentity;
    shouldResetView = true;
    renderGraph();
}

// 窗口操作
async function closeWindow() {
    try { await getCurrentWindow().close(); } catch {}
}
async function minimizeWindow() {
    try { await getCurrentWindow().minimize(); } catch {}
}
async function toggleMaximize() {
    try {
        const win = getCurrentWindow();
        const isMax = await win.isMaximized();
        if (isMax) await win.unmaximize();
        else await win.maximize();
    } catch {}
}
let dragging = false;
let refreshTimer = null;
function startDrag(e) {
    if (e.target.closest("button") || e.target.closest(".ant-popover")) return;
    dragging = true;
    const startPos = { x: e.screenX, y: e.screenY };
    const onMove = (ev) => {
        if (!dragging) return;
        const dx = ev.screenX - startPos.x;
        const dy = ev.screenY - startPos.y;
        startPos.x = ev.screenX;
        startPos.y = ev.screenY;
        getCurrentWindow().outerPosition().then((pos) => {
            getCurrentWindow().setPosition(new PhysicalPosition(pos.x + dx, pos.y + dy));
        });
    };
    const onUp = () => { dragging = false; window.removeEventListener("mousemove", onMove); window.removeEventListener("mouseup", onUp); };
    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
}

onMounted(async () => {
    await loadGraphStyle();

    // 加载文件数据（独立窗口中 fileStore 可能为空）
    await refreshGraphData();
    nextTick(() => renderGraph());
    window.addEventListener("focus", refreshGraphData);
    window.addEventListener("simple-write:file-updated", refreshGraphData);
    document.addEventListener("visibilitychange", refreshGraphData);
    refreshTimer = window.setInterval(refreshGraphData, 2500);
});

onUnmounted(() => {
    window.removeEventListener("focus", refreshGraphData);
    window.removeEventListener("simple-write:file-updated", refreshGraphData);
    document.removeEventListener("visibilitychange", refreshGraphData);
    if (refreshTimer) {
        window.clearInterval(refreshTimer);
        refreshTimer = null;
    }
});

async function refreshGraphData() {
    if (typeof document !== "undefined" && document.visibilityState === "hidden") return;
    if (typeof window !== "undefined" && window.__TAURI_INTERNALS__) {
        try {
            const { invoke } = await import("@tauri-apps/api/core");
            const { Store } = await import("@tauri-apps/plugin-store");
            const store = await Store.load("store.json");
            const warehouse = await store.get("warehouse_now");
            if (warehouse?.path && warehouse?.name) {
                const whPath = warehouse.path + "/" + warehouse.name;
                warehousePath.value = whPath;
                fileData.value = await invoke("get_file_json", { warehousePath: whPath });
            }
        } catch (e) {
            console.error("加载文件数据失败:", e);
        }
    }
    renderGraph();
}
</script>

<style scoped>
.graph-window {
    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
    background: var(--bg-base, #fff);
    overflow: hidden;
}

.graph-header {
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px;
    border-bottom: 1px solid var(--border-color, #e8e8e8);
    background: var(--bg-secondary, #fafafa);
    flex-shrink: 0;
    user-select: none;
}

.graph-header__title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary, #262626);
}

.graph-header__actions {
    display: flex;
    align-items: center;
    gap: 4px;
}

.graph-icon-btn,
.graph-close-btn {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary, #595959);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 14px;
}

.graph-icon-btn:hover {
    background: rgba(0, 0, 0, 0.06);
    color: var(--text-primary, #262626);
}

.graph-close-btn:hover {
    background: #ff4d4f;
    color: #fff;
}

.graph-toolbar-btn {
    height: 28px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary, #595959);
    display: inline-flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    padding: 0 8px;
    font-size: 12px;
}

.graph-toolbar-btn:hover,
.graph-toolbar-btn--active {
    background: rgba(0, 0, 0, 0.06);
    color: var(--text-primary, #262626);
}

.graph-canvas {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    position: relative;
}

.graph-canvas svg {
    width: 100%;
    height: 100%;
}

/* 节点详情面板 */
.graph-node-detail {
    position: absolute;
    background: var(--bg-elevated, #fff);
    border: 1px solid var(--border-color, #e8e8e8);
    border-radius: 8px;
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08);
    padding: 8px;
    min-width: 180px;
    max-height: 200px;
    overflow: auto;
    z-index: 100;
}

.graph-node-detail__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 13px;
    font-weight: 600;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--border-secondary, #f0f0f0);
    margin-bottom: 4px;
    color: var(--text-primary, #262626);
}

.graph-node-detail__empty {
    font-size: 12px;
    color: #bfbfbf;
    padding: 12px 0;
    text-align: center;
}

.graph-node-detail__item {
    padding: 4px 6px;
    cursor: pointer;
    font-size: 12px;
    color: #1890ff;
    border-radius: 4px;
}

.graph-node-detail__item:hover {
    background: #e6f7ff;
}

/* 图谱设置 */
.graph-popover-content {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.settings-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 13px;
    color: var(--text-primary, #262626);
}

.settings-slider {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.slider-label {
    font-size: 12px;
    color: var(--text-secondary, #595959);
}

.native-color-input {
    width: 28px;
    height: 22px;
    border: 1px solid var(--border-color, #d9d9d9);
    border-radius: 4px;
    cursor: pointer;
    padding: 1px;
}
</style>
