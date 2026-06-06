<script setup>
import { computed, provide, ref, onMounted, onUnmounted } from "vue";
import Content from "./template/Content.vue";
import FileList from "./menu/FileList.vue";
import SearchList from "./menu/SearchList.vue";
import BookMark from "./menu/BookMark.vue";
import Favorite from "./menu/Favorite.vue";
import Warehouse from "./menu/Warehouse.vue";
import Language from "./menu/Language.vue";
import Help from "./menu/Help.vue";
import Setting from "./menu/Setting.vue";
import Graph from "./views/Graph.vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n, loadLanguage } from "./locales";
import { message } from "ant-design-vue";
import { quickPaste } from "./menu/fileActions";
import {
  MenuUnfoldOutlined,
  MenuFoldOutlined,
  DeploymentUnitOutlined,
  AppstoreOutlined,
  FolderOpenOutlined,
  SearchOutlined,
  BookOutlined,
  StarOutlined,
  DeliveredProcedureOutlined,
  LinkOutlined,
  SnippetsOutlined,
} from "@ant-design/icons-vue";

// 初始化语言
const { t } = useI18n();

// 关系图独立窗口模式
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
const isRelationMode = ref(window.location.hash === '#relation');

if (isRelationMode.value) {
    document.title = t('common.relationGraph');
}

const closeRelation = async () => {
    try { await getCurrentWindow().close(); } catch {}
};

// 打开关系图谱窗口（已存在则聚焦）
const openRelationGraph = async () => {
  try {
    const existing = WebviewWindow.getByLabel('relation-graph');
    if (existing) {
      await existing.unminimize();
      await existing.setFocus();
      return;
    }
  } catch {}
  new WebviewWindow('relation-graph', {
    url: '/#relation',
    title: t('common.relationGraph'),
    width: 1000,
    height: 700,
    decorations: false,
  });
};

// tab 状态
const tabs = ref([]);
const activeTabId = ref("");
const navigationBackStack = ref([]);
const navigationForwardStack = ref([]);

const canGoBack = computed(() => navigationBackStack.value.length > 0);
const canGoForward = computed(() => navigationForwardStack.value.length > 0);

const normalizePath = (filePath) => String(filePath || "").replace(/\\/g, "/");

// 图片文件扩展名列表
const imageExtensions = ["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp", "ico"];

const getFileExtension = (filePath) => {
    if (!filePath) return "";
    const fileName = filePath.split("/").filter(Boolean).pop() || filePath;
    return fileName.split(".").pop()?.toLowerCase() || "";
};

const isImageFile = (filePath) => imageExtensions.includes(getFileExtension(filePath));

const getDisplayName = (filePath, fallback = "") => {
  const normalizedPath = normalizePath(filePath);
  const fileName = fallback || normalizedPath.split("/").filter(Boolean).pop() || normalizedPath;

  if (fileName.startsWith(".") && !fileName.slice(1).includes(".")) {
    return fileName;
  }

  const displayName = fileName.replace(/(\.[^.]+)+$/, "");
  return displayName || fileName;
};

const getTabIndex = (tabId) => tabs.value.findIndex((tab) => tab.id === tabId);

const removeTabFromHistory = (tabId) => {
  navigationBackStack.value = navigationBackStack.value.filter((historyTabId) => historyTabId !== tabId);
  navigationForwardStack.value = navigationForwardStack.value.filter((historyTabId) => historyTabId !== tabId);
};

const setActiveTab = (tabId, { trackHistory = true } = {}) => {
  if (!tabId || activeTabId.value === tabId) {
    activeTabId.value = tabId;
    return;
  }

  const targetExists = tabs.value.some((tab) => tab.id === tabId);
  if (!targetExists) {
    return;
  }

  if (trackHistory && activeTabId.value) {
    navigationBackStack.value.push(activeTabId.value);
    navigationForwardStack.value = [];
  }

  activeTabId.value = tabId;
};

const goBack = () => {
  if (!navigationBackStack.value.length) {
    return;
  }

  const targetTabId = navigationBackStack.value.pop();
  if (!targetTabId || targetTabId === activeTabId.value) {
    return;
  }

  const currentTabId = activeTabId.value;
  if (currentTabId) {
    navigationForwardStack.value.push(currentTabId);
  }

  activeTabId.value = targetTabId;
};

const goForward = () => {
  if (!navigationForwardStack.value.length) {
    return;
  }

  const targetTabId = navigationForwardStack.value.pop();
  if (!targetTabId || targetTabId === activeTabId.value) {
    return;
  }

  const currentTabId = activeTabId.value;
  if (currentTabId) {
    navigationBackStack.value.push(currentTabId);
  }

  activeTabId.value = targetTabId;
};

const updateTab = (tabId, patch) => {
  const exists = tabs.value.some((tab) => tab.id === tabId);

  if (!exists) {
    return null;
  }

  const nextTabs = tabs.value.map((tab) => {
    if (tab.id !== tabId) {
      return tab;
    }

    return {
      ...tab,
      ...patch,
    };
  });

  tabs.value = nextTabs;
  return nextTabs.find((tab) => tab.id === tabId) || null;
};

const activateTab = (tabId) => {
  setActiveTab(tabId);
};

const closeTab = (tabId) => {
  const closeIndex = getTabIndex(tabId);
  if (closeIndex === -1) {
    return;
  }

  const isActive = activeTabId.value === tabId;
  tabs.value.splice(closeIndex, 1);
  removeTabFromHistory(tabId);

  if (isActive) {
    const nextTab = tabs.value[closeIndex] || tabs.value[closeIndex - 1] || null;
    if (nextTab) {
      setActiveTab(nextTab.id, { trackHistory: false });
    } else {
      activeTabId.value = "";
    }
  }
};

const isPathWithinPrefix = (path, prefix) => {
  return path === prefix || path.startsWith(`${prefix}/`);
};

const replaceTabPathPrefix = (oldPrefix, newPrefix) => {
  const normalizedOldPrefix = normalizePath(oldPrefix);
  const normalizedNewPrefix = normalizePath(newPrefix);

  if (!normalizedOldPrefix || !normalizedNewPrefix || normalizedOldPrefix === normalizedNewPrefix) {
    return;
  }

  const tabIdMap = new Map();

  tabs.value = tabs.value.map((tab) => {
    if (!isPathWithinPrefix(tab.path, normalizedOldPrefix)) {
      return tab;
    }

    const suffix = tab.path === normalizedOldPrefix
      ? ""
      : tab.path.slice(normalizedOldPrefix.length);
    const nextPath = `${normalizedNewPrefix}${suffix}`;

    tabIdMap.set(tab.id, nextPath);

    return {
      ...tab,
      id: nextPath,
      path: nextPath,
      name: getDisplayName(nextPath),
    };
  });

  activeTabId.value = tabIdMap.get(activeTabId.value) || activeTabId.value;
  navigationBackStack.value = navigationBackStack.value.map((tabId) => tabIdMap.get(tabId) || tabId);
  navigationForwardStack.value = navigationForwardStack.value.map((tabId) => tabIdMap.get(tabId) || tabId);
};

const closeTabsByPathPrefix = (pathPrefix) => {
  const normalizedPrefix = normalizePath(pathPrefix);

  if (!normalizedPrefix) {
    return;
  }

  const currentTabs = [...tabs.value];
  const firstRemovedIndex = currentTabs.findIndex((tab) => isPathWithinPrefix(tab.path, normalizedPrefix));

  if (firstRemovedIndex === -1) {
    return;
  }

  const removedTabIds = new Set(
    currentTabs
      .filter((tab) => isPathWithinPrefix(tab.path, normalizedPrefix))
      .map((tab) => tab.id)
  );

  tabs.value = currentTabs.filter((tab) => !removedTabIds.has(tab.id));
  navigationBackStack.value = navigationBackStack.value.filter((tabId) => !removedTabIds.has(tabId));
  navigationForwardStack.value = navigationForwardStack.value.filter((tabId) => !removedTabIds.has(tabId));

  if (!removedTabIds.has(activeTabId.value)) {
    return;
  }

  const nextTab = tabs.value[firstRemovedIndex] || tabs.value[firstRemovedIndex - 1] || null;
  activeTabId.value = nextTab ? nextTab.id : "";
};

const loadTabContent = async (tabId) => {
  const currentTab = tabs.value.find((tab) => tab.id === tabId);

  if (!currentTab) {
    return;
  }

  // 图片文件无需加载文本内容
  if (isImageFile(currentTab.path)) {
    updateTab(tabId, { loading: false, error: "", isDirty: false });
    return;
  }

  updateTab(tabId, {
    loading: true,
    error: "",
    content: "",
  });

  try {
    const content = await invoke("get_file_content", { filePath: currentTab.path });
    const normalizedContent = typeof content === "string" ? content : String(content ?? "");

    updateTab(tabId, {
      content: normalizedContent,
      draftContent: normalizedContent,
      loading: false,
      error: "",
      isDirty: false,
    });
  } catch (error) {
    updateTab(tabId, {
      content: "",
      draftContent: "",
      loading: false,
      error: error?.message || t('content.readFileFailed'),
      isDirty: false,
    });
  }
};

// 点击文件时读取内容并刷新右侧内容区
const openFile = async ({ filePath, fileName }) => {
  if (!filePath) {
    return;
  }

  const path = normalizePath(filePath);
  const existingTab = tabs.value.find((tab) => tab.id === path);

  if (existingTab) {
    setActiveTab(existingTab.id);

    if (existingTab.loading || (!existingTab.content && !existingTab.error)) {
      await loadTabContent(existingTab.id);
    }

    return;
  }

  const tab = {
    id: path,
    name: getDisplayName(path, fileName),
    path,
    content: "",
    draftContent: "",
    viewMode: "read",
    isDirty: false,
    loading: false,
    error: "",
  };

  tabs.value.push(tab);
  setActiveTab(tab.id);
  // 图片文件无需加载文本内容
  if (!isImageFile(path)) {
    await loadTabContent(tab.id);
  }
};

provide("tabs", tabs);
provide("activeTabId", activeTabId);
provide("activateTab", activateTab);
provide("closeTab", closeTab);
provide("updateTab", updateTab);
provide("openFile", openFile);
provide("replaceTabPathPrefix", replaceTabPathPrefix);
provide("closeTabsByPathPrefix", closeTabsByPathPrefix);

onMounted(async () => {
  if (!isRelationMode.value) {
    await loadLanguage();
    // 监听来自图谱窗口的打开文件事件
    listen('open-file-from-graph', (event) => {
      const { path, name } = event.payload || {};
      if (path) {
        openFile({ filePath: path, fileName: name || path.split('/').pop() });
      }
    });
  }
  window.addEventListener('simple-write:switch-view', (e) => {
    const { view } = e.detail || {};
    if (view) switchView(view);
  });
});

onUnmounted(() => {
  // listener cleanup handled by window lifecycle
});

// 控制面板展开状态
const panelExpanded = ref(true);
// 控制当前显示的内容
const currentView = ref("filelist");
const MIN_PANEL_WIDTH = 200;
const MAX_PANEL_WIDTH = 600;
const DEFAULT_PANEL_WIDTH = 200;

// 控制面板宽度 - 默认展开时更窄一些
const panelWidth = ref(DEFAULT_PANEL_WIDTH);
// 控制是否正在拖动
const isResizing = ref(false);
// 记录拖动开始时的鼠标位置和面板宽度
const startX = ref(0);
const startWidth = ref(0);

const currentPanelWidth = computed(() => (panelExpanded.value ? panelWidth.value : 0));
const currentHandleWidth = computed(() => (panelExpanded.value ? 4 : 0));

const panelComponents = {
  filelist: FileList,
  search: SearchList,
  bookmark: BookMark,
  favorite: Favorite,
};

const currentPanelComponent = computed(() => panelComponents[currentView.value] || FileList);

const switchView = (view) => {
  currentView.value = view;
};

// 触发批量导出（由 FileList 监听）
const openBatchExport = () => {
  window.dispatchEvent(new CustomEvent("simple-write:open-batch-export"));
};

// 链接查看器
const linkViewerOpen = ref(false);
const customLinks = ref([]);

const openLinkViewer = async () => {
  linkViewerOpen.value = true;
  customLinks.value = [];
  try {
    if (window.__TAURI_INTERNALS__) {
      const { readCustomLinks } = await import("./menu/novelActions");
      customLinks.value = await readCustomLinks();
    }
  } catch { /* 忽略 */ }
};

// 快速粘贴：读取剪贴板内容并追加到仓库根目录文件
const handleQuickPaste = async () => {
  try {
    const fileName = t("quickPaste.fileName");
    const filePath = await quickPaste(fileName);
    message.success(t("quickPaste.success"));
  } catch (error) {
    // 后端返回错误类型码（格式: "error_type" 或 "error_type|detail"），前端做 i18n 翻译
    const errMsg = error?.message || error || "";
    const raw = typeof errMsg === "string" ? errMsg : String(errMsg);
    const sep = raw.indexOf("|");
    const code = sep > -1 ? raw.substring(0, sep) : raw;
    const detail = sep > -1 ? raw.substring(sep + 1) : "";

    const errorKeyMap = {
      clipboard_access: "quickPaste.errorClipboardAccess",
      clipboard_no_text: "quickPaste.errorClipboardNoText",
      clipboard_empty: "quickPaste.errorClipboardEmpty",
      duplicate_content: "quickPaste.errorDuplicateContent",
      file_open: "quickPaste.errorFileOpen",
      write_file: "quickPaste.errorWriteFile",
    };

    const i18nKey = errorKeyMap[code] || "quickPaste.error";
    const msg = detail ? `${t(i18nKey)}: ${detail}` : t(i18nKey);
    message.warning(msg);
  }
};

const jumpToFile = async (filePath, locateText = "") => {
  const name = filePath.split("/").filter(Boolean).pop() || filePath;
  await openFile({ filePath, fileName: name });
  if (locateText && locateText !== name) {
    window.dispatchEvent(new CustomEvent("simple-write:locate-text", {
      detail: { filePath, text: locateText },
    }));
  }
};

// 开始拖动
const startResize = (e) => {
  isResizing.value = true;
  startX.value = e.clientX;
  startWidth.value = panelWidth.value;
  document.addEventListener("mousemove", handleResize);
  document.addEventListener("mouseup", stopResize);
  // 添加临时样式，防止拖动时选中文本
  document.body.style.userSelect = "none";
  document.body.style.cursor = "col-resize";
};

// 处理拖动
const handleResize = (e) => {
  if (!isResizing.value) return;

  // 计算鼠标移动的距离
  const deltaX = e.clientX - startX.value;
  // 计算新的宽度
  const newWidth = startWidth.value + deltaX;

  // 限制宽度，并在超出范围时直接钳制到边界
  panelWidth.value = Math.min(Math.max(newWidth, MIN_PANEL_WIDTH), MAX_PANEL_WIDTH);
};

// 停止拖动
const stopResize = () => {
  isResizing.value = false;
  document.removeEventListener("mousemove", handleResize);
  document.removeEventListener("mouseup", stopResize);
  // 恢复默认样式
  document.body.style.userSelect = "";
  document.body.style.cursor = "";
};
</script>

<template>
  <!-- 关系图谱独立窗口 -->
  <Graph v-if="isRelationMode" />

  <a-layout v-else class="app-layout">
    <!-- 左侧边栏 -->
    <a-layout-sider class="left_sider" width="auto">
      <!-- 左侧边栏 展开 等 -->
      <a-flex vertical class="sider-buttons" justify="space-between">
        <div>
          <!-- 展开按钮 -->
          <a-tooltip :title="!panelExpanded ? t('common.expand') : t('common.collapse')" placement="right" :arrow="false">
            <a-button
              type="text"
              class="sider-button"
              @click="panelExpanded = !panelExpanded"
            >
              <MenuUnfoldOutlined v-if="!panelExpanded" />
              <MenuFoldOutlined v-else />
            </a-button>
          </a-tooltip>

          <!-- 关系图谱按钮 -->
          <a-tooltip :title="t('common.relationGraph')" placement="right" :arrow="false">
            <a-button type="text" class="sider-button" @click="openRelationGraph">
              <DeploymentUnitOutlined />
            </a-button>
          </a-tooltip>

          <!-- 批量导出按钮 -->
          <a-tooltip :title="t('common.batchExport')" placement="right" :arrow="false">
            <a-button type="text" class="sider-button" @click="openBatchExport">
              <DeliveredProcedureOutlined />
            </a-button>
          </a-tooltip>

          <!-- 查看链接按钮 -->
          <a-tooltip :title="t('common.viewLinks')" placement="right" :arrow="false">
            <a-button type="text" class="sider-button" @click="openLinkViewer">
              <LinkOutlined />
            </a-button>
          </a-tooltip>

          <!-- 快速粘贴按钮 -->
          <a-tooltip :title="t('quickPaste.tooltip')" placement="right" :arrow="false">
            <a-button type="text" class="sider-button" @click="handleQuickPaste">
              <SnippetsOutlined />
            </a-button>
          </a-tooltip>

          <!-- 其他按钮 待定-->
          <!-- <a-tooltip :title="t('common.otherFeatures')" placement="right" :arrow="false">
            <a-button type="text" class="sider-button">
              <AppstoreOutlined />
            </a-button>
          </a-tooltip> -->
        </div>

        <!-- 帮助 语言 设置 -->
        <a-flex vertical>
          <Help />

          <Language />

          <Setting />
        </a-flex>
      </a-flex>
    </a-layout-sider>

    <!-- 主内容区域 -->
    <a-layout class="main-layout">
      <!-- 展开的 panel：保留 DOM，通过宽度过渡实现平滑展开/收起 -->
      <a-layout
        class="expanded-layout"
        :class="{ collapsed: !panelExpanded, resizing: isResizing }"
        :style="{
          width: currentPanelWidth + 'px',
          minWidth: currentPanelWidth + 'px',
          maxWidth: currentPanelWidth + 'px',
          flex: '0 0 ' + currentPanelWidth + 'px',
        }"
      >
        <!-- 上侧边栏 -->
        <a-layout-header class="top-header panel-section">
          <div class="top-toolbar">
            <!-- 文件列表按钮 -->
            <a-tooltip :title="t('file.fileList')" placement="bottom" :arrow="false">
              <a-button
                class="sider-panel-siderbutton" ghost
                :type="currentView === 'filelist' ? 'primary' : 'text'"
                @click="switchView('filelist')"
              >
                <FolderOpenOutlined />
              </a-button>
            </a-tooltip>

            <!-- 搜索按钮 -->
            <a-tooltip :title="t('file.search')" placement="bottom" :arrow="false">
              <a-button
                class="sider-panel-siderbutton" ghost
                :type="currentView === 'search' ? 'primary' : 'text'"
                @click="switchView('search')"
              >
                <SearchOutlined />
              </a-button>
            </a-tooltip>

            <!-- 书签按钮 -->
            <a-tooltip :title="t('file.bookmark')" placement="bottom" :arrow="false">
              <a-button
                class="sider-panel-siderbutton" ghost
                :type="currentView === 'bookmark' ? 'primary' : 'text'"
                @click="switchView('bookmark')"
              >
                <BookOutlined />
              </a-button>
            </a-tooltip>

            <!-- 收藏按钮 -->
            <a-tooltip :title="t('file.favorite')" placement="bottom" :arrow="false">
              <a-button
                class="sider-panel-siderbutton" ghost
                :type="currentView === 'favorite' ? 'primary' : 'text'"
                @click="switchView('favorite')"
              >
                <StarOutlined />
              </a-button>
            </a-tooltip>
          </div>
        </a-layout-header>

        <!-- 内容区域 -->
        <a-layout-content class="content-area panel-section">
          <KeepAlive>
            <component :is="currentPanelComponent" />
          </KeepAlive>
        </a-layout-content>

        <!-- 下方设置栏 -->
        <a-layout-footer class="bottom-footer panel-section">
          <Warehouse />
        </a-layout-footer>
      </a-layout>

      <!-- 拖动条 -->
      <div
        class="resize-handle"
        :class="{ collapsed: !panelExpanded }"
        :style="{ width: currentHandleWidth + 'px' }"
        @mousedown="panelExpanded ? startResize($event) : null"
      ></div>

      <!-- 主内容 -->
      <a-layout-content class="content">
        <Content />
      </a-layout-content>
    </a-layout>

    <!-- 查看链接弹窗 -->
    <a-modal v-model:open="linkViewerOpen" :title="t('content.allLinks')" :footer="null" width="500px">
      <div v-if="customLinks.length === 0" style="color:var(--text-disabled,#bfbfbf);text-align:center;padding:24px;">{{ t('content.noLinks') }}</div>
      <div v-for="link in customLinks" :key="link.id" class="link-viewer-item">
        <span class="link-viewer-name" @click="jumpToFile(link.targetPath, link.targetText)">{{ link.name }}</span>
        <span class="link-viewer-target">{{ link.targetText || link.targetPath.split('/').pop() || link.targetPath }}</span>
      </div>
    </a-modal>
  </a-layout>
</template>

<style scoped>
.app-layout {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  display: flex;
}

.left_sider {
  background-color: var(--bg-base, #fff);
  border-right: 1px solid var(--border-color, #e8e8e8);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sider-buttons {
  padding-top: 6px;
  height: 100%;
}

.expanded-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: var(--bg-base, #fff);
  overflow: hidden;
  position: relative;
  will-change: width;
  transition: width 0.26s cubic-bezier(0.22, 1, 0.36, 1);
}

.expanded-layout.resizing {
  transition: none;
}

.panel-section {
  min-width: 0;
  opacity: 1;
  transition: opacity 0.16s ease;
}

.expanded-layout.collapsed .panel-section {
  opacity: 0;
  pointer-events: none;
}

.sider-button {
  width: 34px;
  height: 34px;
  min-width: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border-radius: 8px;
  margin: 0 8px 4px 8px;
}

.main-layout {
  display: flex;
  flex-direction: row;
  height: 100%;
  flex: 1;
}

.top-header {
  background-color: var(--bg-base, #fff);
  border-bottom: 1px solid var(--border-color, #e8e8e8);
  padding: 3px 8px 0;
  height: 40px;
  display: flex;
  align-items: center;
}

.top-toolbar {
  width: 100%;
  height: 100%;
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  align-items: center;
  justify-items: center;
}

.sider-panel-siderbutton {
  width: 34px;
  height: 34px;
  min-width: 34px;
  padding: 0;
  border-radius: 8px;
}

.content-area {
  flex: 1;
  overflow: hidden;
  background-color: var(--bg-tertiary, #f5f5f5);
  min-width: 0;
}

.bottom-footer {
  background-color: var(--bg-base, #fff);
  border-top: 1px solid var(--border-color, #e8e8e8);
  padding: 10px;
  height: 50px;
  flex-shrink: 0;
}

.content {
  flex: 1;
  overflow: hidden;
  background-color: var(--bg-base, #fff);
}

.resize-handle {
  position: relative;
  z-index: 10;
  flex-shrink: 0;
  cursor: col-resize;
  background-color: transparent;
  overflow: hidden;
  transition: width 0.26s cubic-bezier(0.22, 1, 0.36, 1);
}

.resize-handle::after {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 2px;
  transform: translateX(-50%);
  background-color: transparent;
  transition: background-color 0.2s, width 0.2s, opacity 0.2s ease;
  border-radius: 999px;
  opacity: 1;
}

.resize-handle.collapsed {
  cursor: default;
}

.resize-handle::before {
  content: "";
  position: absolute;
  top: 0;
  bottom: 0;
  left: 50%;
  width: 1px;
  transform: translateX(-50%);
  background-color: var(--border-color, #e8e8e8);
  transition: background-color 0.2s, width 0.2s, opacity 0.2s ease;
  border-radius: 999px;
  opacity: 1;
}

.resize-handle.collapsed::before {
  opacity: 0;
}

.resize-handle:hover::before {
  background-color: #1890ff;
  width: 4px;
}

.resize-handle:hover::after {
  background-color: rgba(24, 144, 255, 0.2);
}

.resize-handle:active::before {
  background-color: #096dd9;
  width: 4px;
}

.resize-handle:active::after {
  background-color: rgba(9, 109, 217, 0.25);
}

</style>

<!-- 全局 antd 组件主题覆盖和主题修复（非 scoped） -->
<style>
/* 文字按钮（左侧图标按钮、面板标签按钮） */
.ant-btn.ant-btn-text:not(:disabled) {
  color: var(--text-primary, rgba(0, 0, 0, 0.88)) !important;
}
.ant-btn.ant-btn-text:not(:disabled):hover {
  color: var(--text-primary, rgba(0, 0, 0, 0.88)) !important;
  background: var(--bg-tertiary, rgba(0, 0, 0, 0.04)) !important;
}

/* ghost 按钮（仓库按钮使用） */
.ant-btn-default.ant-btn-background-ghost:not(:disabled) {
  color: var(--text-primary, rgba(0, 0, 0, 0.88)) !important;
  border-color: var(--border-color, #d9d9d9) !important;
}
.ant-btn-default.ant-btn-background-ghost:not(:disabled):hover {
  color: var(--text-primary, rgba(0, 0, 0, 0.88)) !important;
  border-color: var(--text-primary, #4096ff) !important;
}

/* Modal 弹窗 — 全部使用 !important 对抗 antd 运行时注入 */
.ant-modal-content {
  background-color: var(--bg-elevated, #fff) !important;
}
.ant-modal-header {
  background-color: var(--bg-elevated, #fff) !important;
  border-bottom-color: var(--border-secondary, #f0f0f0) !important;
}
.ant-modal-title {
  color: var(--text-primary, #262626) !important;
}
.ant-modal-close {
  color: var(--text-tertiary, #8c8c8c) !important;
}
.ant-modal-close:hover {
  color: var(--text-primary, #262626) !important;
}
.ant-modal-body {
  background-color: var(--bg-elevated, #fff) !important;
}
.ant-modal-footer {
  background-color: var(--bg-elevated, #fff) !important;
}

/* Modal.confirm 编程式确认框 */
.ant-modal-confirm .ant-modal-confirm-body {
  background-color: var(--bg-elevated, #fff) !important;
}
.ant-modal-confirm-body .ant-modal-confirm-content {
  color: var(--text-primary, #262626) !important;
}
.ant-modal-confirm-title {
  color: var(--text-primary, #262626) !important;
}
.ant-modal-confirm-btns {
  background-color: var(--bg-elevated, #fff) !important;
}

/* Popover（仓库选择弹出面板） */
.ant-popover-inner {
  background-color: var(--bg-elevated, #fff) !important;
}
.ant-popover-title {
  color: var(--text-primary, #262626) !important;
}

/* 下拉菜单 */
.ant-dropdown-menu {
  background-color: var(--bg-elevated, #fff) !important;
}
.ant-dropdown-menu .ant-dropdown-menu-item {
  color: var(--text-primary, #262626) !important;
}
.ant-dropdown-menu .ant-dropdown-menu-item:hover {
  background-color: var(--bg-tertiary, #f5f5f5) !important;
}
/* 子菜单标题 */
.ant-dropdown-menu .ant-dropdown-menu-submenu-title {
  color: var(--text-primary, #262626) !important;
}
.ant-dropdown-menu .ant-dropdown-menu-submenu-title:hover {
  background-color: var(--bg-tertiary, #f5f5f5) !important;
}
/* 子菜单图标 */
.ant-dropdown-menu .ant-dropdown-menu-submenu-title .anticon,
.ant-dropdown-menu .ant-dropdown-menu-submenu-title .svg-icon {
  color: var(--text-secondary, #595959) !important;
}

/* 卡片 */
.ant-card {
  background-color: var(--bg-elevated, #fff) !important;
}
.ant-card-body {
  background-color: var(--bg-elevated, #fff) !important;
}

/* Layout */
.ant-layout {
  background-color: var(--bg-base, #fff) !important;
}
.ant-layout-sider {
  background-color: var(--bg-base, #fff) !important;
}
.ant-layout-content {
  background-color: var(--bg-base, #fff) !important;
}

/* 排版文本 */
.ant-typography {
  color: var(--text-primary, #262626) !important;
}

/* 输入框 */
.ant-input {
  color: var(--text-primary, #262626) !important;
  background-color: var(--bg-base, #fff) !important;
  border-color: var(--border-color, #d9d9d9) !important;
}
.ant-input[disabled] {
  color: var(--text-tertiary, #8c8c8c) !important;
  background-color: var(--bg-secondary, #fafafa) !important;
}
.ant-input-group-addon {
  color: var(--text-primary, #262626) !important;
  background-color: var(--bg-secondary, #fafafa) !important;
  border-color: var(--border-color, #d9d9d9) !important;
}

/* 标签页 */
.ant-tabs-tab .ant-tabs-tab-btn {
  color: var(--text-secondary, #595959) !important;
}
.ant-tabs-tab-active .ant-tabs-tab-btn {
  color: #1890ff !important;
}

/* 空状态 */
.ant-empty-description {
  color: var(--text-tertiary, #8c8c8c) !important;
}

/* 菜单 */
.ant-menu {
  background-color: var(--bg-elevated, #fff) !important;
}
.ant-menu-item:not(.ant-menu-item-selected) {
  color: var(--text-primary, #262626) !important;
}
.ant-menu-item:not(.ant-menu-item-selected):hover {
  color: var(--text-primary, #262626) !important;
  background-color: var(--bg-tertiary, #f5f5f5) !important;
}
.ant-menu-item-selected {
  background-color: var(--bg-tertiary, #e6f7ff) !important;
}

/* 分隔线 */
.ant-divider {
  border-top-color: var(--border-color, #f0f0f0) !important;
}

/* 卡片标题 */
.ant-card-head {
  color: var(--text-primary, #262626) !important;
  background-color: var(--bg-elevated, #fff) !important;
  border-bottom-color: var(--border-secondary, #f0f0f0) !important;
}

/* ---- 主题修复：搜索/书签/收藏 allow-clear 区域 ---- */
.ant-input-clear-icon {
  color: var(--text-tertiary, #8c8c8c) !important;
}
.ant-input-affix-wrapper .ant-input-suffix {
  background-color: var(--bg-base, #fff) !important;
}
.ant-input-affix-wrapper {
  background-color: var(--bg-base, #fff) !important;
}
.ant-input-affix-wrapper .ant-input {
  background-color: var(--bg-base, #fff) !important;
}

/* ---- 滚动条主题覆盖 ---- */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
::-webkit-scrollbar-track {
  background: var(--bg-secondary, #fafafa);
}
::-webkit-scrollbar-thumb {
  background: var(--text-disabled, #bfbfbf);
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary, #8c8c8c);
}

/* ---- 搜索框内部背景 ---- */
.ant-input-search .ant-input-wrapper,
.ant-input-search .ant-input-affix-wrapper {
  background-color: var(--bg-base, #fff) !important;
}

/* ---- 大纲统计区域文字颜色 ---- */
.outline-stats-bar {
  border-bottom-color: var(--border-secondary, #f0f0f0) !important;
}
.outline-stats__total {
  color: var(--text-primary, #262626) !important;
}
.outline-stats__row {
  color: var(--text-tertiary, #8c8c8c) !important;
}

/* ---- 右键菜单中图标颜色修复 ---- */
.ant-dropdown-menu-submenu-title .anticon,
.ant-dropdown-menu-item .anticon {
  color: var(--text-secondary, #595959) !important;
}
.ant-dropdown-menu-item-danger .anticon {
  color: #ff4d4f !important;
}

/* ---- 选择文本高亮 ---- */
::selection {
  background: rgba(24, 144, 255, 0.3);
  color: inherit;
}

/* ---- Tab 下拉菜单滚动条 ---- */
.ant-dropdown-menu::-webkit-scrollbar-track {
  background: var(--bg-elevated, #fff);
}

/* ---- 链接查看器 ---- */
.link-viewer-item {
  display: flex; justify-content: space-between; align-items: center;
  padding: 8px 12px; border-bottom: 1px solid var(--border-secondary, #f0f0f0);
}
.link-viewer-item:last-child { border-bottom: none; }
.link-viewer-name { color: #1890ff; cursor: pointer; font-size: 13px; }
.link-viewer-name:hover { text-decoration: underline; }
.link-viewer-target { color: var(--text-tertiary, #8c8c8c); font-size: 12px; }
</style>
