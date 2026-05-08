<template>
    <div class="content-view">
        <div class="content-view__tabbar">
            <div class="content-view__tabbar-scroll">
                <div
                    v-for="tab in tabs"
                    :key="tab.id"
                    class="content-view__tab"
                    :class="{ 'content-view__tab--active': tab.id === activeTabId }"
                    :style="tabItemStyle"
                    @click="activateTab(tab.id)"
                >
                    <span class="content-view__tab-title">{{ tab.name }}</span>
                    <button
                        type="button"
                        class="content-view__tab-close"
                        @click.stop="requestCloseTab(tab.id)"
                        aria-label="close tab"
                    >
                        <CloseOutlined />
                    </button>
                </div>

                <div v-if="tabs.length === 0" class="content-view__tabbar-empty">
                    {{ t("content.noFileSelected") }}
                </div>
            </div>

            <a-dropdown v-model:open="tabOverviewOpen" :trigger="['click']" placement="bottom">
                <a-button
                    type="text"
                    :disabled="tabs.length === 0"
                    :icon="h(DownOutlined)"
                    class="content-view__header-button"
                ></a-button>

                <template #overlay>
                    <div class="content-view__tab-overview">
                        <div
                            v-for="tab in tabs"
                            :key="tab.id"
                            class="content-view__tab-overview-item"
                            :class="{ 'content-view__tab-overview-item--active': tab.id === activeTabId }"
                            @click="activateTab(tab.id)"
                        >
                            <span class="content-view__tab-overview-title">{{ tab.name }}</span>
                            <button
                                type="button"
                                class="content-view__tab-overview-close"
                                @click.stop="requestCloseTab(tab.id)"
                                aria-label="close tab"
                            >
                                <CloseOutlined />
                            </button>
                        </div>
                    </div>
                </template>
            </a-dropdown>
        </div>

        <div class="content-view__panel">
            <template v-if="activeTab">
                <div class="content-view__header">
                    <div class="content-view__header-side content-view__header-side--left">
                        <a-tooltip :title="t('content.back')" placement="bottom" :arrow="false">
                            <button
                                type="button"
                                class="content-view__header-button"
                                :disabled="!canGoBack"
                                @click="handleGoBack"
                            >
                                <LeftOutlined />
                            </button>
                        </a-tooltip>

                        <a-tooltip :title="t('content.forward')" placement="bottom" :arrow="false">
                            <button
                                type="button"
                                class="content-view__header-button"
                                :disabled="!canGoForward"
                                @click="handleGoForward"
                            >
                                <RightOutlined />
                            </button>
                        </a-tooltip>
                    </div>

                    <div class="content-view__header-title">{{ activeTab.name }}</div>

                    <div class="content-view__header-side content-view__header-side--right">
                        <a-tooltip :title="t('content.changeViewMode')" placement="bottomRight" :arrow="false">
                            <button
                                type="button"
                                class="content-view__header-switch"
                                :disabled="!canSwitchView"
                                @click="toggleViewMode"
                            >
                                <ReadOutlined v-if="activeTabViewMode === 'read'" />
                                <EditOutlined v-else />
                                <span class="content-view__header-switch-state">
                                    {{ activeTabViewMode === "read" ? t("content.readView") : t("content.editView") }}
                                </span>
                            </button>
                        </a-tooltip>

                        <div v-if="isMarkdownFile">
                            <!-- 大纲 -->
                            <a-tooltip :title="t('content.outline')" :arrow="false" placement="bottom">
                                <a-button type="text" class="content-view__header-button" @click="openOutline">
                                    <BarsOutlined />
                                </a-button>
                            </a-tooltip>

                            <!-- 链接列表 -->
                            <a-tooltip :title="t('content.linkList')" :arrow="false" placement="bottom">
                                <a-button type="text" class="content-view__header-button" @click="openLinkList">
                                    <LinkOutlined />
                                </a-button>
                            </a-tooltip>
                        </div>

                        <!-- 更多选项 -->
                        <!-- <div class="content-view__more-wrap">
                        </div> -->
                        <a-dropdown overlay-class-name="more-menu-dropdown">
                            <a-button type="text" :icon="h(EllipsisOutlined)" class="content-view__header-button"/>
                            <template #overlay>
                                <a-menu>
                                    <a-menu-item :disabled="!canUndo" @click="canUndo && handleMoreClick('undo')" :icon="h(UndoOutlined)">
                                        {{ t('content.undo') }}</a-menu-item>
                                    <a-menu-item :disabled="!canRedo" @click="canRedo && handleMoreClick('redo')" :icon="h(RedoOutlined)">
                                        {{ t('content.redo') }}</a-menu-item>
                                    <a-menu-divider />
                                    <a-menu-item @click="handleMoreClick('rename')" :icon="h(EditOutlined)">
                                        {{ t('file.rename') }}</a-menu-item>
                                    <a-menu-item @click="handleMoreClick('move')" :icon="h(SvgIcon, { raw: svgIcons.move })">
                                        {{ t('file.move') }}</a-menu-item>
                                    <a-menu-item 
                                        :class="{ 'content-view__more-menu-item--favorited': isCurrentFavorited }" 
                                        @click="handleMoreClick('favorite')" 
                                        :icon="h(StarOutlined)"
                                    >
                                        {{ t('file.favorite') }}
                                    </a-menu-item>
                                    <div v-if="isMarkdownFile">
                                        <a-menu-divider />
                                        <a-menu-item
                                            @click="handleMoreClick('export')"
                                            :icon="h(DeliveredProcedureOutlined)"
                                        >
                                            {{ t('file.export') }}
                                        </a-menu-item>
                                    </div>
                                </a-menu>
                            </template>
                        </a-dropdown>
                    </div>
                </div>

                <div v-if="activeTab.loading" class="content-view__state">
                    {{ t("content.loading") }}
                </div>

                <div
                    v-else-if="activeTab.error"
                    class="content-view__state content-view__state--error"
                >
                    {{ activeTab.error }}
                </div>

                <div
                    v-else-if="isImageFile"
                    class="content-view__image"
                >
                    <div v-if="imageLoading" class="content-view__state">
                        {{ t("content.loading") }}
                    </div>
                    <img
                        v-else-if="imageSrc"
                        :src="imageSrc"
                        :alt="activeTab.name"
                    />
                    <div v-else class="content-view__state content-view__state--error">
                        {{ t("content.noData") }}
                    </div>
                </div>

                <template v-else>
                    <KeepAlive>
                        <EditView
                            v-if="activeTabViewMode === 'edit'"
                            ref="editViewRef"
                            :key="activeTabId"
                            :initial-content="activeTab?.draftContent ?? activeTab?.content ?? ''"
                            :original-content="activeTab?.content ?? ''"
                            :disabled="isSaving"
                            :is-dirty="activeTabIsDirty"
                            :is-saving="isSaving"
                        />
                    </KeepAlive>
                    <KeepAlive>
                        <ReadView
                            v-if="activeTabViewMode === 'read'"
                            :key="activeTabId"
                            :content="activeTab.content"
                            :file-path="activeTab.path"
                            :file-name="activeTab.name"
                            :top-line="savedTopLine"
                            :restore-scroll-top="getTabScrollTop(activeTabId)"
                        />
                    </KeepAlive>
                </template>
            </template>

            <div v-else class="content-view__empty">
                {{ t("content.noFileSelected") }}
            </div>
        </div>

        <!-- 重命名弹窗 -->
        <a-modal v-model:open="renameModalOpen" :title="t('file.rename')" :ok-text="t('file.confirm')" :cancel-text="t('file.cancel')" @ok="handleRename">
            <a-input :addon-before="t('file.currentName')" disabled :value="activeTab?.name" />
            <br /><br />
            <a-input :addon-before="t('file.newName')" :placeholder="t('file.enter')" v-model:value="renameNewName" />
        </a-modal>

        <!-- 移动弹窗 -->
        <a-modal v-model:open="moveModalOpen" :title="t('file.move')" :ok-text="t('file.confirm')" :cancel-text="t('file.cancel')" @ok="handleMove">
            <a-input :addon-before="t('file.currentPath')" disabled :value="activeTab?.path" />
            <br /><br />
            <a-input :addon-before="t('file.newPath')" :placeholder="t('file.pleaseSelectDirectory')" v-model:value="moveTargetPath" disabled />
            <br /><br />
            <a-button type="primary" block @click="selectMoveTarget">{{ t('file.selectNewPath') }}</a-button>
        </a-modal>

        <!-- 导出弹窗 -->
        <a-modal v-model:open="exportModalOpen" :title="t('file.export')" :ok-text="t('file.confirm')" :cancel-text="t('file.cancel')" @ok="handleExport">
            <a-input :addon-before="t('file.currentName')" disabled :value="activeTab?.name" />
            <br /><br />
            <a-input :addon-before="t('file.newName')" :placeholder="t('file.enter')" v-model:value="exportFileName" addon-after=".txt" />
            <br /><br />
            <a-input :addon-before="t('file.newPath')" :placeholder="t('file.pleaseSelectDirectory')" v-model:value="exportTargetPath" disabled />
            <br /><br />
            <a-button type="primary" block @click="selectExportTarget">{{ t('file.selectNewPath') }}</a-button>
        </a-modal>

        <!-- 链接列表弹窗-->
        <a-modal v-model:open="linkListOpen" :title="t('content.linkList')" :footer="null" width="480px" wrap-class-name="link-list-modal">
            <a-tabs v-model:activeKey="linkListTab" centered>
                <!-- 样式勿改 -->
                <a-tab-pane key="outgoing">
                    <div v-if="outgoingLinks.length === 0" class="link-list-empty">
                        {{ t('content.noData') }}
                    </div>
                    <div v-for="p in outgoingLinks" :key="p" class="link-list-item" @click="openLinkedFile(p)">
                        <span>{{ p.split('/').pop() }}</span>
                        <span class="link-list-path">{{ toRelativePath(p) }}</span>
                    </div>
                    <template #tab>
                        <a-tooltip :title="t('content.outgoingLink')" placement="top" :arrow="false">
                            <span><LinkOutlined style="margin-right:5px;" />{{ outgoingLinks.length }}</span>
                        </a-tooltip>
                    </template>
                </a-tab-pane>
                <a-tab-pane key="incoming">
                    <div v-if="incomingLinks.length === 0" class="link-list-empty">
                        {{ t('content.noData') }}
                    </div>
                    <div v-for="p in incomingLinks" :key="p" class="link-list-item" @click="openLinkedFile(p)">
                        <span>{{ p.split('/').pop() }}</span>
                        <span class="link-list-path">{{ toRelativePath(p) }}</span>
                    </div>
                    <template #tab>
                        <a-tooltip :title="t('content.incomingLink')" placement="top" :arrow="false">
                            <span><ExportOutlined style="margin-right:5px;" />{{ incomingLinks.length }}</span>
                        </a-tooltip>
                    </template>
                </a-tab-pane>
            </a-tabs>
        </a-modal>

        <!-- 大纲弹窗 -->
        <a-modal v-model:open="outlineOpen" :title="t('content.outline')" :footer="null" width="360px" wrap-class-name="outline-modal">
            <div v-if="outlineItems.length === 0" class="link-list-empty">{{ t('content.noData') }}</div>
            <template v-else>
                <div class="outline-stats-bar">
                    <span class="outline-stats__total">{{ t('content.outlineTotalCount').replace('{n}', outlineItems.length) }}</span>
                    <span v-for="d in outlineStatsDetail" :key="d.level" class="outline-stats__row">
                        {{ t('content.outlineLevelCount').replace('{level}', d.level).replace('{count}', d.count) }}
                    </span>
                </div>
                <div class="outline-modal__body">
                <div
                    v-for="h in visibleOutlineItems"
                    :key="h.index"
                    class="outline-item"
                    :style="{ paddingLeft: (h.level - 1) * 16 + 'px' }"
                >
                    <span
                        class="outline-item__toggle"
                        :class="{ 'outline-item__toggle--invisible': !hasOutlineChildren(h.index) }"
                        @click.stop="toggleOutlineCollapse(h.index)"
                    >
                        <CaretDownOutlined v-if="!collapsedHeadingIndices.has(h.index)" />
                        <CaretRightOutlined v-else />
                    </span>
                    <span class="outline-item__level">H{{ h.level }}</span>
                    <span class="outline-item__text" @click="scrollToHeading(h)">{{ h.text }}</span>
                </div>
            </div>
            </template>
        </a-modal>

    </div>
</template>

<script setup>
import { computed, inject, onBeforeUnmount, onMounted, ref, watch, nextTick, h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { message, Modal } from "ant-design-vue";
import {
    CloseOutlined,
    EllipsisOutlined,
    LeftOutlined,
    RightOutlined,
    BarsOutlined,
    ReadOutlined,
    EditOutlined,
    UndoOutlined,
    RedoOutlined,
    StarOutlined,
    LinkOutlined,
    ExportOutlined,
    DownOutlined,
    CaretDownOutlined,
    CaretRightOutlined,
    DeliveredProcedureOutlined
} from "@ant-design/icons-vue";
import { saveFileContent, renameFileEntry, moveFileEntry, getWarehouseRootPath } from "../menu/fileActions";
import { createFavorite, removeFavorite, fetchFavorites } from "../menu/favoriteActions";
import { scanLinks, getOutgoingLinks, getIncomingLinks } from "../menu/linkActions";
import { fileData, warehousePath, flattenTree } from "../stores/fileStore";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { Marked } from "marked";
import { useI18n } from "../locales";
import EditView from "./EditView.vue";
import ReadView from "./ReadView.vue";
import svgIcons from "../assets/icons";
import SvgIcon from "../components/SvgIcon.vue";

const { t } = useI18n();

// ---- 注入的全局状态 ----
const tabs = inject("tabs", ref([]));
const activeTabId = inject("activeTabId", ref(""));
const activateTab = inject("activateTab", () => {});
const closeTab = inject("closeTab", () => {});
const updateTab = inject("updateTab", () => {});
const replaceTabPathPrefix = inject("replaceTabPathPrefix", () => {});
const openFile = inject("openFile", null);
const canGoBack = inject("canGoBack", ref(false));
const canGoForward = inject("canGoForward", ref(false));
const goBack = inject("goBack", () => {});
const goForward = inject("goForward", () => {});

// ---- 文件类型常量 ----
const imageExtensions = ["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp", "ico"];
const editableExtensions = ["md", "txt"];

// ---- 文件扩展名工具 ----
const getFileExtension = (filePath) => {
    if (!filePath) return "";
    // 从路径中提取文件名，再取扩展名
    const fileName = filePath.split("/").filter(Boolean).pop() || filePath;
    return fileName.split(".").pop()?.toLowerCase() || "";
};

// ---- 局部状态 ----
const tabOverviewOpen = ref(false);
const isSaving = ref(false);
const isClosePromptOpen = ref(false);
const editViewRef = ref(null);

const AUTO_SAVE_INTERVAL_MS = 3 * 60 * 1000;
let autoSaveTimerId = null;
let closeWindowUnlisten = null;
let allowWindowClose = false;

// ---- 计算属性 ----
const activeTab = computed(() => {
    return tabs.value.find((tab) => tab.id === activeTabId.value) || null;
});

const activeTabViewMode = computed(() => activeTab.value?.viewMode || "read");

const activeTabIsDirty = computed(() => Boolean(activeTab.value?.isDirty));

const hasDirtyTabs = computed(() => tabs.value.some((tab) => tab.isDirty));

const isImageFile = computed(() => {
    if (!activeTab.value?.path) return false;
    return imageExtensions.includes(getFileExtension(activeTab.value.path));
});

const isMarkdownFile = computed(() => {
    if (!activeTab.value?.path) return false;
    return getFileExtension(activeTab.value.path) === 'md';
});

const canSwitchView = computed(() => {
    if (!activeTab.value || activeTab.value.loading || isSaving.value) return false;
    return editableExtensions.includes(getFileExtension(activeTab.value.path));
});

const canUndo = computed(() => {
    return canSwitchView.value && activeTabViewMode.value === 'edit' && undoCount.value > 0;
});
const canRedo = computed(() => {
    return canSwitchView.value && activeTabViewMode.value === 'edit' && redoCount.value > 0;
});

const imageSrc = ref("");
const imageLoading = ref(false);

// 加载图片 base64 数据
const loadImageData = async (filePath) => {
    if (!filePath) return;
    imageLoading.value = true;
    imageSrc.value = "";
    try {
        const dataUrl = await invoke("read_file_as_base64", { filePath });
        imageSrc.value = typeof dataUrl === "string" ? dataUrl : "";
    } catch (error) {
        console.error("读取图片失败:", error);
        imageSrc.value = "";
    } finally {
        imageLoading.value = false;
    }
};

// 当切换到图片文件时自动加载
watch(
    () => (isImageFile.value ? activeTab.value?.path : null),
    (path) => {
        if (path) loadImageData(path);
        else imageSrc.value = "";
    },
    { immediate: true }
);

// ---- tab 工具函数 ----
const getTabById = (tabId) => tabs.value.find((tab) => tab.id === tabId) || null;

const getTabDraftContent = (tab) => tab?.draftContent ?? tab?.content ?? "";

const updateTabState = (tabId, patch) => {
    if (typeof updateTab !== "function") return null;
    return updateTab(tabId, patch);
};

// ---- 保存 ----
const saveTabDraft = async (tabId) => {
    const targetTab = getTabById(tabId);
    if (!targetTab?.path) return true;

    isSaving.value = true;
    try {
        const contentToSave = getTabDraftContent(targetTab);
        await saveFileContent(targetTab.path, contentToSave);
        updateTabState(tabId, {
            content: contentToSave,
            draftContent: contentToSave,
            loading: false,
            error: "",
            isDirty: false,
        });
        undoCount.value = 0; redoCount.value = 0;
        return true;
    } catch (error) {
        message.error(error?.message || "保存文件失败");
        return false;
    } finally {
        isSaving.value = false;
    }
};

const saveAllDirtyTabs = async () => {
    const dirtyTabIds = tabs.value.filter((tab) => tab.isDirty).map((tab) => tab.id);
    for (const tabId of dirtyTabIds) {
        const saved = await saveTabDraft(tabId);
        if (!saved) return false;
    }
    return true;
};

// ---- 自动保存 ----
const startAutoSaveTimer = () => {
    if (autoSaveTimerId !== null) return;
    autoSaveTimerId = window.setInterval(() => {
        if (activeTabViewMode.value === "edit" && activeTabIsDirty.value && !isSaving.value && !isClosePromptOpen.value) {
            void saveTabDraft(activeTabId.value);
        }
    }, AUTO_SAVE_INTERVAL_MS);
};

const stopAutoSaveTimer = () => {
    if (autoSaveTimerId === null) return;
    window.clearInterval(autoSaveTimerId);
    autoSaveTimerId = null;
};

// ---- 键盘 / 窗口事件 ----
const handleGlobalKeydown = (event) => {
    if (activeTabViewMode.value !== "edit") return;
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
        event.preventDefault();
        // 同步 textarea 最新值
        const textarea = editViewRef.value?.editorTextarea;
        const tab = activeTab.value;
        if (textarea && tab) {
            const val = textarea.value;
            updateTab(tab.id, {
                draftContent: val,
                isDirty: val !== (tab.content ?? ""),
            });
        }
        void saveTabDraft(activeTabId.value);
    }
};

const handleBeforeUnload = (event) => {
    if (!hasDirtyTabs.value) return;
    event.preventDefault();
    event.returnValue = t("content.closeTabMessage");
    return t("content.closeTabMessage");
};

const closeCurrentWindow = async (currentWindow) => {
    allowWindowClose = true;
    try {
        await currentWindow.close();
    } finally {
        allowWindowClose = false;
    }
};

const openUnsavedChangesDialog = ({ saveAction, onSaveConfirmed, onDiscardConfirmed }) => {
    if (isClosePromptOpen.value) return;
    isClosePromptOpen.value = true;
    Modal.confirm({
        title: t("content.closeTabTitle"),
        content: t("content.closeTabMessage"),
        okText: t("content.saveAndClose"),
        cancelText: t("content.closeWithoutSaving"),
        centered: true,
        maskClosable: false,
        async onOk() {
            const saved = await saveAction();
            if (!saved) return Promise.reject(new Error("save failed"));
            isClosePromptOpen.value = false;
            await onSaveConfirmed();
            return undefined;
        },
        async onCancel() {
            isClosePromptOpen.value = false;
            await onDiscardConfirmed();
        },
    });
};

const handleWindowCloseRequested = (currentWindow) => async (event) => {
    if (allowWindowClose || isClosePromptOpen.value || !hasDirtyTabs.value) return;
    event.preventDefault();
    openUnsavedChangesDialog({
        saveAction: saveAllDirtyTabs,
        onSaveConfirmed: async () => { await closeCurrentWindow(currentWindow); },
        onDiscardConfirmed: async () => { await closeCurrentWindow(currentWindow); },
    });
};

const requestCloseTab = (tabId) => {
    const targetTab = getTabById(tabId);
    if (!targetTab) return;
    if (!targetTab.isDirty) {
        closeTab(tabId);
        return;
    }
    openUnsavedChangesDialog({
        saveAction: () => saveTabDraft(tabId),
        onSaveConfirmed: async () => { closeTab(tabId); },
        onDiscardConfirmed: async () => { closeTab(tabId); },
    });
};

// ---- 导航 / 视图切换 ----
const handleGoBack = () => {
    if (canGoBack.value && typeof goBack === "function") goBack();
};

const handleGoForward = () => {
    if (canGoForward.value && typeof goForward === "function") goForward();
};

// ---- 更多选项：收藏 ----
const favoritedPaths = ref(new Set());

const loadFavoritedPaths = async () => {
    try {
        const list = await fetchFavorites();
        favoritedPaths.value = new Set((Array.isArray(list) ? list : []).map((f) => f.filePath || f.path || ''));
    } catch { /* 静默失败 */ }
};

const isCurrentFavorited = computed(() => {
    const path = activeTab.value?.path;
    return path ? favoritedPaths.value.has(path) : false;
});

watch(() => activeTab.value?.path, () => { loadFavoritedPaths(); }, { immediate: true });

const handleToggleFavorite = async () => {
    const tab = activeTab.value;
    if (!tab?.path) return;
    try {
        if (isCurrentFavorited.value) {
            await removeFavorite({ filePath: tab.path });
        } else {
            await createFavorite({ filePath: tab.path, itemType: 'file' });
        }
        await loadFavoritedPaths();
        window.dispatchEvent(new CustomEvent("simple-write:favorites-updated"));
        message.success(t("message.success"));
    } catch (e) {
        message.error(e?.message || t("message.error"));
    }
};

// ---- 更多选项：重命名 ----
const renameModalOpen = ref(false);
const renameNewName = ref('');

const openRenameModal = () => {
    const tab = activeTab.value;
    if (!tab) return;
    const name = (tab.name || '').replace(/\.[^.]+$/, '');
    renameNewName.value = name;
    renameModalOpen.value = true;
};

const handleRename = async () => {
    const tab = activeTab.value;
    if (!tab) return;
    const newBase = renameNewName.value.trim();
    if (!newBase) { message.warning(t("file.pleaseEnterName")); return; }
    const oldName = tab.path.split('/').filter(Boolean).pop() || tab.name;
    const ext = oldName.includes('.') ? '.' + oldName.split('.').pop() : '';
    const newName = newBase.includes('.') ? newBase : newBase + ext;
    if (newName === oldName) { message.warning(t("file.sameName")); return; }
    const key = tab.path.replace(/\\/g, '/').split('/').filter(Boolean);
    const wsPath = (await getWarehouseRootPath()) || '';
    const wsSegs = wsPath.replace(/\\/g, '/').split('/').filter(Boolean);
    const itemKey = key.slice(wsSegs.length);
    try {
        await renameFileEntry({ key: itemKey, newName });
        const oldPath = tab.path;
        const parentPath = oldPath.split('/').slice(0, -1).join('/');
        const newPath = parentPath + '/' + newName;
        replaceTabPathPrefix(oldPath, newPath);
        message.success(t("message.success"));
    } catch (e) {
        message.error(e?.message || t("message.error"));
    } finally {
        renameModalOpen.value = false;
    }
};

// ---- 更多选项：移动 ----
const moveModalOpen = ref(false);
const moveTargetPath = ref('');

const exportModalOpen = ref(false);
const exportFileName = ref('');
const exportTargetPath = ref('');

const openMoveModal = () => {
    moveTargetPath.value = '';
    moveModalOpen.value = true;
};

const selectMoveTarget = async () => {
    const selected = await openDialog({ multiple: false, directory: true });
    if (typeof selected === 'string' && selected.trim()) {
        moveTargetPath.value = selected;
    }
};

const handleMove = async () => {
    const tab = activeTab.value;
    if (!tab?.path) return;
    const target = moveTargetPath.value.trim();
    if (!target) { message.warning(t("file.selectNewPath")); return; }
    const oldPath = tab.path.replace(/\\/g, '/');
    const currentParent = oldPath.split('/').slice(0, -1).join('/');
    if (target.replace(/\\/g, '/') === currentParent) { message.warning(t("file.samePath")); return; }
    const wsPath = (await getWarehouseRootPath()) || '';
    const wsSegs = wsPath.replace(/\\/g, '/').split('/').filter(Boolean);
    const key = oldPath.split('/').filter(Boolean);
    const itemKey = key.slice(wsSegs.length);
    try {
        await moveFileEntry({ key: itemKey, newParentPath: target });
        const name = oldPath.split('/').pop();
        const newPath = target.replace(/\\/g, '/') + '/' + name;
        replaceTabPathPrefix(oldPath, newPath);
        message.success(t("message.success"));
    } catch (e) {
        message.error(e?.message || t("message.error"));
    } finally {
        moveModalOpen.value = false;
    }
};

// 将 markdown 内容转为纯文本（去除所有格式符号）
const markdownToPlainText = (md) => {
    const marked = new Marked();
    const html = marked.parse(md ?? '');
    // 去除 HTML 标签，解码实体
    return html
        .replace(/<[^>]*>/g, '')
        .replace(/&amp;/g, '&')
        .replace(/&lt;/g, '<')
        .replace(/&gt;/g, '>')
        .replace(/&quot;/g, '"')
        .replace(/&#39;/g, "'")
        .replace(/&nbsp;/g, ' ');
};

const selectExportTarget = async () => {
    const selected = await openDialog({ multiple: false, directory: true });
    if (typeof selected === 'string' && selected.trim()) {
        const wsPath = (warehousePath.value || '').replace(/\\/g, '/');
        const selPath = selected.replace(/\\/g, '/');
        if (!selPath.startsWith(wsPath)) {
            message.warning(t('file.pathMustBeWithinWarehouse'));
            return;
        }
        exportTargetPath.value = selected;
    }
};

const handleExport = async () => {
    const tab = activeTab.value;
    if (!tab) return;
    const name = exportFileName.value.trim();
    if (!name) { message.warning(t('file.pleaseEnterName')); return; }
    const targetPath = exportTargetPath.value.trim();
    if (!targetPath) { message.warning(t('file.pleaseSelectDirectory')); return; }
    try {
        const plainText = markdownToPlainText(tab.content ?? '');
        // 去重：导出路径下已有同名文件时追加 (1)、(2)...
        const basePath = targetPath.replace(/\\/g, '/').replace(/\/+$/, '');
        const allFiles = flattenTree(fileData.value, warehousePath.value);
        const siblings = new Set(
            allFiles
                .map(f => f.path.replace(/\\/g, '/'))
                .filter(p => p.startsWith(basePath + '/') && p.lastIndexOf('/') === basePath.length)
                .map(p => p.split('/').pop())
        );
        let finalName = name;
        let counter = 0;
        while (siblings.has(finalName + '.txt')) {
            counter++;
            finalName = `${name}(${counter})`;
        }
        const filePath = basePath + '/' + finalName + '.txt';
        await saveFileContent(filePath, plainText);
        window.dispatchEvent(new CustomEvent('simple-write:file-updated', { detail: { path: filePath } }));
        message.success(t('message.success'));
    } catch (e) {
        message.error(e?.message || t('message.error'));
    } finally {
        exportModalOpen.value = false;
    }
};

const undoCount = ref(0);
const redoCount = ref(0);

// 编辑时标记可撤销，保存后重置
watch(() => activeTabIsDirty.value, (dirty) => {
    if (dirty) undoCount.value = Math.max(undoCount.value, 1);
});
watch(() => activeTab.value?.id, () => { undoCount.value = 0; redoCount.value = 0; });

const handleMoreClick = (key) => {
    if (key === 'undo') {
        const textarea = editViewRef.value?.editorTextarea;
        if (textarea) {
            textarea.focus();
            const ok = document.execCommand('undo');
            if (ok) { undoCount.value = Math.max(0, undoCount.value - 1); redoCount.value++; }
        }
    } else if (key === 'redo') {
        const textarea = editViewRef.value?.editorTextarea;
        if (textarea) {
            textarea.focus();
            const ok = document.execCommand('redo');
            if (ok) { redoCount.value = Math.max(0, redoCount.value - 1); undoCount.value++; }
        }
    } else if (key === 'rename') {
        nextTick(() => openRenameModal());
    } else if (key === 'move') {
        nextTick(() => openMoveModal());
    } else if (key === 'favorite') {
        handleToggleFavorite();
    } else if (key === 'export') {
        const tab = activeTab.value;
        if (tab) {
            const base = (tab.name || '').replace(/\.md$/i, '');
            exportFileName.value = base;
            exportTargetPath.value = warehousePath.value || '';
            exportModalOpen.value = true;
        }
    }
};

const savedTopLine = ref(0); // 第一屏顶部的源文本行号

// 每个标签页的阅读视图滚动位置（编辑视图由 KeepAlive 保留）
const tabScrollTops = ref({});

// 接收 ReadView onDeactivated 上报的滚动位置
const handleSaveScroll = (e) => {
    const { filePath, scrollTop } = e.detail || {};
    if (!filePath || scrollTop == null) return;
    const tab = tabs.value.find(t => t.path === filePath);
    if (tab && scrollTop !== (tabScrollTops.value[tab.id] || 0)) {
        tabScrollTops.value = { ...tabScrollTops.value, [tab.id]: scrollTop };
    }
};

const getTabScrollTop = (tabId) => {
    return tabScrollTops.value[tabId] || 0;
};

const getTextareaTopLine = () => {
    const ta = editViewRef.value?.editorTextarea;
    if (!ta) return 0;
    const style = window.getComputedStyle(ta);
    const lineH = parseFloat(style.lineHeight) || parseFloat(style.fontSize) * 1.6 || 20;
    const padTop = parseFloat(style.paddingTop) || 0;
    return Math.max(0, Math.floor((ta.scrollTop - padTop) / lineH));
};

const getBodyTopLine = (body) => {
    // 从 body 的滚动位置推算源文本行号
    if (!body || body.scrollHeight <= body.clientHeight) return 0;
    const totalLines = (activeTab.value?.content ?? '').split('\n').length;
    if (totalLines <= 1) return 0;
    return Math.round((body.scrollTop / (body.scrollHeight - body.clientHeight)) * (totalLines - 1));
};

const toggleViewMode = async () => {
    if (!canSwitchView.value || !activeTab.value) return;

    // 保存当前顶部行号
    if (activeTabViewMode.value === "edit") {
        savedTopLine.value = getTextareaTopLine();
    } else {
        const body = document.querySelector('.read-view__body--markdown');
        savedTopLine.value = getBodyTopLine(body);
    }

    if (activeTabViewMode.value === "read") {
        updateTabState(activeTab.value.id, {
            viewMode: "edit",
            draftContent: getTabDraftContent(activeTab.value),
            isDirty: activeTab.value.isDirty || false,
        });
        // 切到编辑：根据行号还原 textarea 滚动
        nextTick(() => {
            const ta = editViewRef.value?.editorTextarea;
            if (!ta || savedTopLine.value <= 0) return;
            const style = window.getComputedStyle(ta);
            const lineH = parseFloat(style.lineHeight) || parseFloat(style.fontSize) * 1.6 || 20;
            const padTop = parseFloat(style.paddingTop) || 0;
            ta.scrollTop = savedTopLine.value * lineH + padTop;
        });
        return;
    }

    // 切换到阅读视图前先同步 textarea 最新值
    const textarea = editViewRef.value?.editorTextarea;
    const tab = activeTab.value;
    if (textarea && tab) {
        const val = textarea.value;
        updateTab(tab.id, {
            draftContent: val,
            isDirty: val !== (tab.content ?? ""),
        });
    }

    if (activeTabIsDirty.value) {
        const saved = await saveTabDraft(activeTab.value.id);
        if (!saved) return;
    }

    // 切到阅读：把行号作为 prop 传给 ReadView，由 ReadView 渲染后定位
    updateTabState(activeTab.value.id, { viewMode: "read" });
};

// ---- 监听器 ----
watch(
    () => activeTab.value?.viewMode,
    (mode) => {
        if (mode === "edit") startAutoSaveTimer();
        else stopAutoSaveTimer();
    },
    { immediate: true }
);

watch(
    () => tabs.value.length,
    (tabCount) => {
        if (tabCount === 0) tabOverviewOpen.value = false;
    },
    { immediate: true }
);

watch(
    () => hasDirtyTabs.value,
    (hasDirty) => {
        if (!hasDirty) isClosePromptOpen.value = false;
    }
);


// ---- 任务列表切换同步 ----
const handleTaskToggled = (e) => {
    const { path, content } = e.detail || {};
    if (!path) return;
    const tab = tabs.value.find(t => t.path === path);
    if (tab) {
        updateTab(tab.id, { content, draftContent: content, isDirty: false });
    }
};

// ---- 生命周期 ----
onMounted(async () => {
    window.addEventListener("keydown", handleGlobalKeydown);
    window.addEventListener("simple-write:task-toggled", handleTaskToggled);
    window.addEventListener("simple-write:save-scroll", handleSaveScroll);
    if (!window.__TAURI_INTERNALS__) {
        window.addEventListener("beforeunload", handleBeforeUnload);
        return;
    }
    try {
        const currentWindow = getCurrentWindow();
        closeWindowUnlisten = await currentWindow.onCloseRequested(handleWindowCloseRequested(currentWindow));
    } catch (error) {
        console.error("Failed to register window close handler:", error);
    }
});

onBeforeUnmount(() => {
    stopAutoSaveTimer();
    window.removeEventListener("keydown", handleGlobalKeydown);
    window.removeEventListener("simple-write:task-toggled", handleTaskToggled);
    window.removeEventListener("simple-write:save-scroll", handleSaveScroll);
    window.removeEventListener("beforeunload", handleBeforeUnload);
    if (typeof closeWindowUnlisten === "function") {
        closeWindowUnlisten();
        closeWindowUnlisten = null;
    }
});

// ---- 链接列表 ----
const linkListOpen = ref(false);
const linkListTab = ref('outgoing');
const outgoingLinks = ref([]);
const incomingLinks = ref([]);
const linkWarehousePath = ref('');

const toRelativePath = (abs) => {
    if (!linkWarehousePath.value) return abs;
    let rel = abs.replace(/\\/g, '/');
    const wh = linkWarehousePath.value.replace(/\\/g, '/');
    if (rel.startsWith(wh)) rel = rel.slice(wh.length);
    if (rel.startsWith('/')) rel = rel.slice(1);
    return rel || abs;
};

const openLinkList = async () => {
    linkListOpen.value = true;
    linkListTab.value = 'outgoing';
    outgoingLinks.value = [];
    incomingLinks.value = [];
    if (!activeTab.value?.path) return;
    try {
        linkWarehousePath.value = (await getWarehouseRootPath()) || '';
        await scanLinks();
        const [out, incoming] = await Promise.all([
            getOutgoingLinks(activeTab.value.path),
            getIncomingLinks(activeTab.value.path),
        ]);
        outgoingLinks.value = out;
        incomingLinks.value = incoming;
    } catch (e) {
        message.error(e?.message || t('message.error'));
    }
};

const openLinkedFile = (filePath) => {
    if (!filePath) return;
    // 检查是否为文件夹
    const norm = (p) => (p || '').replace(/\\/g, '/').replace(/\/+$/, '');
    const target = norm(filePath);
    const all = flattenTree(fileData.value, warehousePath.value);
    const entry = all.find((f) => norm(f.path) === target);
    if (entry?.isFolder) {
        window.dispatchEvent(new CustomEvent('simple-write:expand-folder', {
            detail: { path: filePath, name: entry.name },
        }));
        return;
    }
    const name = filePath.split('/').filter(Boolean).pop() || filePath;
    if (typeof openFile === 'function') {
        openFile({ filePath, fileName: name });
    }
};

// ---- 大纲 ----
const outlineOpen = ref(false);
const collapsedHeadingIndices = ref(new Set());

const stripMarkdown = (text) => {
    return text
        .replace(/\[\[([^\]]+)\]\]/g, '$1')
        .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
        .replace(/\[\^[^\]]+\]/g, '')
        .replace(/\*\*(.+?)\*\*/g, '$1')
        .replace(/(?<!\*)\*(?!\*)(.+?)(?<!\*)\*(?!\*)/g, '$1')
        .replace(/~~(.+?)~~/g, '$1')
        .replace(/==(.+?)==/g, '$1')
        .replace(/`(.+?)`/g, '$1')
        .trim();
};

const extractOutline = (content) => {
    if (!content) return [];
    const lines = content.split('\n');
    const items = [];
    const occurrenceCounter = {};
    for (let i = 0; i < lines.length; i++) {
        const m = lines[i].match(/^(#{1,6})\s+(.+)/);
        if (m) {
            const level = m[1].length;
            const text = stripMarkdown(m[2]);
            const key = `${level}:${text}`;
            const occurrenceIndex = occurrenceCounter[key] || 0;
            occurrenceCounter[key] = occurrenceIndex + 1;
            items.push({ level, text, occurrenceIndex, index: items.length });
        }
    }
    return items;
};

const outlineItems = computed(() => {
    return extractOutline(activeTab.value?.content ?? '');
});

const outlineStatsDetail = computed(() => {
    const breakdown = {};
    for (const item of outlineItems.value) {
        breakdown[item.level] = (breakdown[item.level] || 0) + 1;
    }
    return Object.entries(breakdown)
        .filter(([, count]) => count > 0)
        .map(([level, count]) => ({ level: Number(level), count }))
        .sort((a, b) => a.level - b.level);
});

const hasOutlineChildren = (i) => {
    const items = outlineItems.value;
    const currentLevel = items[i].level;
    for (let j = i + 1; j < items.length; j++) {
        if (items[j].level <= currentLevel) return false;
        if (items[j].level > currentLevel) return true;
    }
    return false;
};

const visibleOutlineItems = computed(() => {
    const items = outlineItems.value;
    if (collapsedHeadingIndices.value.size === 0) return items;

    const result = [];
    const hideStack = [];
    for (let i = 0; i < items.length; i++) {
        const item = items[i];
        while (hideStack.length > 0 && hideStack[hideStack.length - 1].level >= item.level) {
            hideStack.pop();
        }
        const isHidden = hideStack.some(e => e.hidden);
        if (!isHidden) result.push(item);
        hideStack.push({ level: item.level, hidden: collapsedHeadingIndices.value.has(i) });
    }
    return result;
});

const toggleOutlineCollapse = (i) => {
    const set = collapsedHeadingIndices.value;
    if (set.has(i)) {
        set.delete(i);
    } else {
        set.add(i);
    }
    // 触发响应式更新
    collapsedHeadingIndices.value = new Set(set);
};

const openOutline = () => {
    collapsedHeadingIndices.value = new Set();
    outlineOpen.value = true;
};

const scrollToHeading = (h) => {
    outlineOpen.value = false;
    if (activeTabViewMode.value !== 'read') return;
    nextTick(() => {
        const body = document.querySelector('.read-view__body--markdown');
        if (!body) return;
        const headings = body.querySelectorAll('h1, h2, h3, h4, h5, h6');
        // 找到第 occurrenceIndex 个匹配的标题元素
        let matchCount = 0;
        for (const el of headings) {
            if (el.textContent?.trim() === h.text && el.tagName === `H${h.level}`) {
                if (matchCount === h.occurrenceIndex) {
                    el.scrollIntoView({ behavior: 'smooth', block: 'start' });
                    return;
                }
                matchCount++;
            }
        }
        // 精确匹配未找到对应序号的，回退到第一个匹配
        for (const el of headings) {
            if (el.textContent?.trim() === h.text && el.tagName === `H${h.level}`) {
                el.scrollIntoView({ behavior: 'smooth', block: 'start' });
                return;
            }
        }
        // 模糊查找
        for (const el of headings) {
            if (el.textContent?.trim().includes(h.text) && el.tagName === `H${h.level}`) {
                el.scrollIntoView({ behavior: 'smooth', block: 'start' });
                return;
            }
        }
    });
};

// ---- 样式辅助 ----
const tabItemStyle = computed(() => {
    const tabCount = tabs.value.length;
    if (tabCount <= 0) return {};
    return {
        width: `calc((100% - ${Math.max(tabCount - 1, 0) * 4}px) / ${tabCount})`,
    };
});
</script>

<style scoped>
.content-view {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    background: var(--bg-base, #fff);
    overflow: hidden;
}

/* ---- tabbar ---- */
.content-view__tabbar {
    height: 34px;
    margin-top: 4px;
    display: flex;
    align-items: stretch;
    gap: 4px;
    padding: 0 8px;
    border-bottom: 1px solid var(--border-color, #e8e8e8);
    overflow: hidden;
    background: var(--bg-secondary, #fafafa);
    flex-shrink: 0;
    box-sizing: border-box;
}

.content-view__tabbar-scroll {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: stretch;
    gap: 4px;
    overflow: hidden;
}

.content-view__tabbar-empty {
    display: flex;
    align-items: center;
    color: var(--text-tertiary, #8c8c8c);
    font-size: 13px;
    padding: 0 4px 6px;
    white-space: nowrap;
    flex-shrink: 0;
}

.content-view__tab {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 10px 0 12px;
    border: 1px solid var(--border-color, #e8e8e8);
    border-bottom: none;
    border-radius: 8px 8px 0 0;
    background: var(--bg-tertiary, #f5f5f5);
    color: var(--text-secondary, #595959);
    cursor: pointer;
    user-select: none;
    flex: 0 0 auto;
    min-width: 0;
    max-width: 200px;
    box-sizing: border-box;
    transform: translateY(1px);
    overflow: hidden;
    transition: width 0.28s cubic-bezier(0.22, 1, 0.36, 1) 0.12s, background-color 0.16s ease, color 0.16s ease, border-color 0.16s ease;
}

.content-view__tab--active {
    background: var(--bg-base, #fff);
    color: var(--text-primary, #1f1f1f);
    border-color: var(--text-disabled, #d9d9d9);
}

.content-view__tab-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
}

.content-view__tab-close {
    width: 20px;
    height: 20px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: inherit;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
}

.content-view__tab-close:hover {
    background: rgba(0, 0, 0, 0.06);
}

.content-view__tabbar-overview {
    width: 30px;
    height: 30px;
    padding: 0;
    flex-shrink: 0;
    border: 1px solid transparent;
    border-radius: 8px;
    color: #595959;
    margin-bottom: -1px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
}

.content-view__tabbar-overview:disabled {
    color: #bfbfbf;
}

.content-view__tab-overview {
    min-width: 220px;
    max-height: 320px;
    overflow: auto;
    padding: 6px;
    background: var(--bg-base, #fff);
    border: 1px solid var(--border-color, #e8e8e8);
    border-radius: 8px;
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08);
}

.content-view__tab-overview-item {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: space-between;
    padding: 6px 8px;
    border-radius: 6px;
    cursor: pointer;
    color: var(--text-secondary, #595959);
}

.content-view__tab-overview-item:hover {
    background: var(--bg-tertiary, #f5f5f5);
}

.content-view__tab-overview-item--active {
    background: #e6f4ff;
    color: #0958d9;
}

.content-view__tab-overview-title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
}

.content-view__tab-overview-close {
    width: 22px;
    height: 22px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: inherit;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
    padding: 0;
}

.content-view__tab-overview-close:hover {
    background: rgba(0, 0, 0, 0.08);
}

/* ---- panel ---- */
.content-view__panel {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 16px;
    padding-top: 5px;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
}

.content-view__header {
    margin-bottom: 10px;
    padding-bottom: 5px;
    border-bottom: 1px solid var(--border-color, #e8e8e8);
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 34px;
}

.content-view__header-side {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
}

.content-view__header-side--left {
    justify-content: flex-start;
}

.content-view__header-side--right {
    justify-content: flex-end;
    margin-left: auto;
}

.content-view__header-title {
    flex: 1;
    min-width: 0;
    font-size: 15px;
    font-weight: 600;
    color: var(--text-primary, #1f1f1f);
    text-align: center;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding: 0 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 34px;
    box-sizing: border-box;
}

.content-view__header-button,
.content-view__header-switch {
    height: 34px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--text-secondary, #595959);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0 10px;
    flex-shrink: 0;
    transition: background-color 0.16s ease, color 0.16s ease, opacity 0.16s ease;
}

.content-view__header-switch {
    gap: 6px;
    min-width: 108px;
    font-size: 13px;
    white-space: nowrap;
}

.content-view__header-switch-state {
    padding: 2px 6px;
    border-radius: 999px;
    background: var(--bg-tertiary, #f5f5f5);
    color: var(--text-tertiary, #8c8c8c);
    font-size: 12px;
    line-height: 1;
}

.content-view__header-button:hover:not(:disabled),
.content-view__header-switch:hover:not(:disabled) {
    background: rgba(0, 0, 0, 0.06);
    color: var(--text-primary, #1f1f1f);
}

.content-view__header-button:disabled,
.content-view__header-switch:disabled {
    color: #bfbfbf;
    cursor: not-allowed;
    opacity: 0.8;
}

/* ---- 状态 ---- */
.content-view__state {
    padding: 16px;
    color: var(--text-secondary, #595959);
    background: var(--bg-secondary, #fafafa);
    border: 1px solid var(--border-secondary, #f0f0f0);
    border-radius: 8px;
}

.content-view__state--error {
    color: #cf1322;
    background: #fff1f0;
    border-color: #ffa39e;
}

.content-view__image {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: auto;
    background: var(--bg-tertiary, #f5f5f5);
    border-radius: 8px;
    border: 1px solid var(--border-color, #e8e8e8);
}

.content-view__image img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
}

.content-view__empty {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary, #8c8c8c);
    font-size: 14px;
}

</style>

<style>
/* dropdown overlay 渲染到 body，必须全局样式 */
.more-menu-dropdown .content-view__more-menu-item--favorited,
.more-menu-dropdown .content-view__more-menu-item--favorited .ant-menu-title-content {
    color: #faad14 !important;
}

/* 链接列表项 */
.link-list-modal .ant-tabs-content {
    max-height: 360px;
    overflow-y: auto;
}

.link-list-item {
    padding: 8px 12px;
    cursor: pointer;
    border-radius: 6px;
    transition: background 0.15s;
    display: flex;
    flex-direction: column;
}

.link-list-item:hover {
    background: var(--bg-tertiary, #f5f5f5);
}

.link-list-item span:first-child {
    font-size: 13px;
    color: #1890ff;
}

/* 链接列表 tab 数字 */
.link-list-tab {
    display: inline-flex;
    align-items: center;
    padding: 4px 12px;
    font-size: 13px;
    color: #595959;
}

/* 大纲弹窗滚动 */
.outline-modal .ant-modal-body {
    max-height: 420px;
    overflow-y: auto;
}

.outline-modal__body {
    min-height: 0;
}

/* 大纲弹窗内统计条 */
.outline-stats-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    padding: 4px 8px 10px;
    border-bottom: 1px solid #f0f0f0;
    margin-bottom: 6px;
}

.outline-stats__total {
    font-size: 13px;
    color: #262626;
    font-weight: 600;
}

.outline-stats__row {
    font-size: 12px;
    color: #8c8c8c;
}

/* 大纲项折叠箭头 */
.outline-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    cursor: pointer;
    border-radius: 6px;
    transition: background 0.15s;
}

.outline-item:hover {
    background: var(--bg-tertiary, #f5f5f5);
}

.outline-item__toggle {
    width: 16px;
    height: 16px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    font-size: 10px;
    color: var(--text-tertiary, #8c8c8c);
    border-radius: 4px;
    transition: background 0.15s;
}

.outline-item__toggle:hover {
    background: rgba(0, 0, 0, 0.06);
}

.outline-item__toggle--invisible {
    visibility: hidden;
}

.outline-item__level {
    font-size: 10px;
    color: var(--text-tertiary, #8c8c8c);
    font-weight: 600;
    flex-shrink: 0;
    width: 18px;
    text-align: center;
}

.outline-item__text {
    font-size: 13px;
    color: var(--text-primary, #262626);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* 弹窗提示文字 */
.modal-form-hint {
    margin-bottom: 8px;
    font-size: 12px;
    color: #8c8c8c;
}

/* 弹窗路径输入框光标 */
.modal-path-input {
    cursor: pointer;
}

/* 链接/大纲空状态 */
.link-list-empty {
    text-align: center;
    color: #bfbfbf;
    padding: 24px;
}

/* 链接路径显示 */
.link-list-path {
    font-size: 11px;
    color: #bfbfbf;
}
</style>

