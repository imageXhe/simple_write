<template>
<div class="file-list-toolbar-wrap">
        <a-flex class="file-list-toolbar" justify="space-evenly" align="center">
            <a-tooltip :title="t('file.newNote')" :arrow="false" placement="bottom">
                <a-button ghost class="file-list-toolbar-button" @click="CreateEntry({ isFolder: false })">
                    <FormOutlined />
                </a-button>
            </a-tooltip>
            <a-tooltip :title="t('file.newFolder')" :arrow="false" placement="bottom">
                <a-button ghost class="file-list-toolbar-button" @click="CreateEntry({ isFolder: true })">
                    <FolderAddOutlined />
                </a-button>
            </a-tooltip>
            <a-tooltip :title="t('file.sort')" :arrow="false" placement="bottom">
                <a-dropdown :trigger="['click']" placement="rightBottom">
                    <a-button ghost class="file-list-toolbar-button">
                        <SortDescendingOutlined />
                    </a-button>
                    <template #overlay>
                        <a-menu class="dropdown-menu-bordered">
                            <a-menu-item key="1" @click="SortAll('name_asc')">{{ t('file.nameAsc') }}</a-menu-item>
                            <a-menu-item key="2" @click="SortAll('name_desc')">{{ t('file.nameDesc') }}</a-menu-item>
                            <a-menu-divider />
                            <a-menu-item key="3" @click="SortAll('modify_time_asc')">{{ t('file.modifyTimeAsc') }}</a-menu-item>
                            <a-menu-item key="4" @click="SortAll('modify_time_desc')">{{ t('file.modifyTimeDesc') }}</a-menu-item>
                            <a-menu-divider />
                            <a-menu-item key="5" @click="SortAll('create_time_asc')">{{ t('file.createTimeAsc') }}</a-menu-item>
                            <a-menu-item key="6" @click="SortAll('create_time_desc')">{{ t('file.createTimeDesc') }}</a-menu-item>
                        </a-menu>
                    </template>
                </a-dropdown>
            </a-tooltip>
            <a-tooltip :title="expand_all ? t('file.collapseAll') : t('file.expandAll')" :arrow="false" placement="bottom">
                <a-button ghost class="file-list-toolbar-button" @click="ExpandAll">
                    <ColumnHeightOutlined />
                </a-button>
            </a-tooltip>
        </a-flex>
    </div>
    <a-divider style="margin: 0;" />
    <div style="overflow: auto;height: calc(100% - 51px);">
        <FilePanel />
    </div>

    <!-- 批量导出 modal -->
    <a-modal
        v-model:open="batchExportOpen"
        :title="t('file.batchExportTitle')"
        :ok-text="t('file.exportShort')"
        :cancel-text="t('file.cancel')"
        width="560px"
        :maskClosable="false"
        :ok-button-props="{ disabled: !batchLocked || selectedExportFilePaths.length === 0 }"
        @ok="handleBatchExport"
        @cancel="batchExportOpen = false"
    >
        <div class="batch-export-body">
            <!-- 文件选择列表 -->
            <div class="batch-export-tree">
                <div class="batch-export-tree-header">
                    <span>{{ t('file.selectExportScope') }}</span>
                    <span style="display:flex;gap:4px;">
                        <a-dropdown :trigger="['click']" placement="bottomRight">
                            <a-button size="small">{{ t('file.sort') }}</a-button>
                            <template #overlay>
                                <a-menu class="dropdown-menu-bordered">
                                    <a-menu-item key="name_asc" @click="setExportSort('name_asc')">{{ t('file.nameAsc') }}</a-menu-item>
                                    <a-menu-item key="name_desc" @click="setExportSort('name_desc')">{{ t('file.nameDesc') }}</a-menu-item>
                                    <a-menu-divider />
                                    <a-menu-item key="modify_time_asc" @click="setExportSort('modify_time_asc')">{{ t('file.modifyTimeAsc') }}</a-menu-item>
                                    <a-menu-item key="modify_time_desc" @click="setExportSort('modify_time_desc')">{{ t('file.modifyTimeDesc') }}</a-menu-item>
                                    <a-menu-divider />
                                    <a-menu-item key="create_time_asc" @click="setExportSort('create_time_asc')">{{ t('file.createTimeAsc') }}</a-menu-item>
                                    <a-menu-item key="create_time_desc" @click="setExportSort('create_time_desc')">{{ t('file.createTimeDesc') }}</a-menu-item>
                                </a-menu>
                            </template>
                        </a-dropdown>
                        <a-button size="small" @click="toggleExportExpandAll">
                            {{ exportAllExpanded ? t('file.collapseAll') : t('file.expandAll') }}
                        </a-button>
                    </span>
                </div>
                <div class="batch-export-tree-scroll">
                    <div v-if="exportableEntries.length === 0" class="batch-export-empty">{{ t('file.noTxtFiles') }}</div>
                    <div v-for="entry in exportableEntries" :key="entry.path" class="batch-export-item"
                        :style="{ paddingLeft: (entry.depth * 16 + 8) + 'px' }">
                        <span
                            v-if="entry.isFolder"
                            class="batch-export-folder-toggle"
                            @click.stop="toggleExportFolderExpand(entry.path)"
                        >
                            {{ exportExpandedFolders.has(entry.path) ? '▾' : '▸' }}
                        </span>
                        <span v-else class="batch-export-folder-toggle batch-export-folder-toggle--placeholder">•</span>
                        <a-checkbox v-if="!batchLocked"
                            :checked="entry.isFolder ? isFolderAllChecked(entry) : selectedExportPaths.has(entry.path)"
                            :indeterminate="entry.isFolder && isFolderIndeterminate(entry)"
                            @change="(e) => toggleExportPath(entry, e.target.checked)"
                        >
                            <span :style="{ color: entry.isFolder ? '#1890ff' : 'var(--text-primary, #262626)' }">
                                {{ entry.isFolder ? '📁' : '📄' }} {{ entry.name }}
                            </span>
                        </a-checkbox>
                        <span v-else style="color:var(--text-primary,#262626)">{{ entry.isFolder ? '📁' : '📄' }} {{ entry.name }}</span>
                    </div>
                </div>
            </div>

            <!-- 确认锁定 -->
            <div class="batch-export-lock">
                <a-checkbox v-model:checked="batchLocked" @change="onLockChange" :disabled="selectedExportPaths.size === 0">
                    <span style="color:var(--text-primary,#262626)">{{ t('file.confirmSelection') }}</span>
                </a-checkbox>
            </div>

            <!-- 导出规则（锁定前禁用） -->
            <div class="batch-export-rules" :class="{ 'batch-export-rules--disabled': !batchLocked }">
                <div class="batch-export-rules-header">
                    <span class="batch-export-rules-title">{{ t('file.exportRules') }}</span>
                    <a-button size="small" :disabled="!batchLocked" @click="resetAllExportRules">
                        {{ t('file.resetAllRules') }}
                    </a-button>
                </div>
                <!-- 层级选择 -->
                <div class="batch-export-rule-level" v-if="maxExportDepth > 0">
                    <span class="batch-export-rule-label">{{ t('file.selectLevel') }}</span>
                    <a-select
                        v-model:value="selectedRuleLevel"
                        size="small"
                        style="width:120px;"
                        :disabled="!batchLocked"
                    >
                        <a-select-option v-for="i in maxExportDepth" :key="i" :value="i - 1">
                            {{ t('file.levelLabel').replace('{n}', i) }}
                        </a-select-option>
                    </a-select>
                </div>
                <!-- 文件夹规则行 -->
                <div class="batch-export-rule-row" v-if="exportDepthRules[selectedRuleLevel]">
                    <span>{{ t('file.folderNameLabel') }}</span>
                    <a-input v-model:value="exportDepthRules[selectedRuleLevel].folder.prefix" size="small" style="width:50px;" :placeholder="t('file.prefix')" :disabled="!batchLocked" />
                    <span>{{ t('file.name') }}</span>
                    <a-input v-model:value="exportDepthRules[selectedRuleLevel].folder.suffix" size="small" style="width:50px;" :placeholder="t('file.suffix')" :disabled="!batchLocked" />
                    <a-checkbox v-model:checked="exportDepthRules[selectedRuleLevel].folder.blank" size="small" :disabled="!batchLocked">
                        <span style="color:var(--text-primary,#262626)">{{ t('file.blankLine') }}</span>
                    </a-checkbox>
                    <a-button size="small" danger :disabled="!batchLocked" @click="exportDepthRules[selectedRuleLevel].folder.skip = !exportDepthRules[selectedRuleLevel].folder.skip">
                        {{ exportDepthRules[selectedRuleLevel].folder.skip ? t('file.restore') : t('file.skip') }}
                    </a-button>
                </div>
                <!-- 文件规则行 -->
                <div class="batch-export-rule-row" v-if="exportDepthRules[selectedRuleLevel]">
                    <span>{{ t('file.fileNameLabel') }}</span>
                    <a-input v-model:value="exportDepthRules[selectedRuleLevel].file.prefix" size="small" style="width:50px;" :placeholder="t('file.prefix')" :disabled="!batchLocked" />
                    <span>{{ t('file.name') }}</span>
                    <a-input v-model:value="exportDepthRules[selectedRuleLevel].file.suffix" size="small" style="width:50px;" :placeholder="t('file.suffix')" :disabled="!batchLocked" />
                    <a-checkbox v-model:checked="exportDepthRules[selectedRuleLevel].file.blank" size="small" :disabled="!batchLocked">
                        <span style="color:var(--text-primary,#262626)">{{ t('file.blankLine') }}</span>
                    </a-checkbox>
                    <a-button size="small" danger :disabled="!batchLocked" @click="exportDepthRules[selectedRuleLevel].file.skip = !exportDepthRules[selectedRuleLevel].file.skip">
                        {{ exportDepthRules[selectedRuleLevel].file.skip ? t('file.restore') : t('file.skip') }}
                    </a-button>
                </div>
            </div>

            <!-- 输出设置 -->
            <div class="batch-export-options">
                <div class="batch-export-option">
                    <span>{{ t('file.outputFileName') }}</span>
                    <a-input v-model:value="exportOutputName" :placeholder="t('file.defaultOutputName')" style="width:180px;" />
                </div>
                <div class="batch-export-option">
                    <span>{{ t('file.outputDirLabel') }}</span>
                    <a-input v-model:value="exportOutputDir" :placeholder="t('file.pleaseSelectDirectory')" disabled style="width:180px;" />
                    <a-button size="small" @click="selectExportOutputDir">{{ t('file.select') }}</a-button>
                </div>
                <div v-if="exportDuplicateHint" class="batch-export-hint">{{ t('file.exportDuplicateHint') }}</div>
            </div>
        </div>
    </a-modal>

    <!-- 改名 移动 modal -->
    <a-modal
        v-model:open="actionModalOpen"
        :title="actionModalTitle"
        :confirm-loading="actionSubmitting"
        :maskClosable="false"
        @ok="handleConfirmAction"
        @cancel="resetActionModal"
    >
        <div v-if="actionModalType === 'rename'">
            <a-input :addon-before="t('file.currentName')" disabled :value="actionModalItem?.info?.name || ''" />
            <br /><br />
            <a-input
                :addon-before="t('file.newName')"
                :placeholder="t('file.enter')"
                v-model:value="actionModalName"
            />
        </div>
        <div v-else-if="actionModalType === 'move'">
            <a-input :addon-before="t('file.currentPath')" disabled :value="currentActionItemPath" />
            <br /><br />
            <a-input
                :addon-before="t('file.newPath')"
                :placeholder="t('file.pleaseSelectDirectory')"
                v-model:value="actionModalPath"
                disabled
            />
            <br /><br />
            <a-button type="primary" block @click="selectMovePath">
                {{ t('file.selectNewPath') }}
            </a-button>
        </div>
    </a-modal>
</template>

<script setup>
import { computed, inject, onBeforeUnmount, onMounted, provide, ref } from "vue";
import { message, Modal } from "ant-design-vue";
import {
    FormOutlined,
    FolderAddOutlined,
    SortDescendingOutlined,
    ColumnHeightOutlined,
    DeliveredProcedureOutlined,
} from "@ant-design/icons-vue";
import { open } from "@tauri-apps/plugin-dialog";
import FilePanel from "./FilePanel.vue";
import { createFavorite, fetchFavorites, removeFavorite } from "./favoriteActions";
import {
    createFileEntry as invokeCreateFileEntry,
    fetchFileTree,
    duplicateFileEntry as invokeDuplicateFileEntry,
    renameFileEntry as invokeRenameFileEntry,
    moveFileEntry as invokeMoveFileEntry,
    deleteFileEntry as invokeDeleteFileEntry,
} from "./fileActions";
import { useI18n } from "../locales";
import { fileData as sharedFileData, warehousePath as sharedWarehousePath, flattenTree } from "../stores/fileStore";
import { saveFileContent } from "./fileActions";
import { invoke } from "@tauri-apps/api/core";

const { t } = useI18n();
const replaceTabPathPrefix = inject("replaceTabPathPrefix", () => {});
const closeTabsByPathPrefix = inject("closeTabsByPathPrefix", () => {});

const file_data = ref([]);
const warehouse_path = ref("");
const favorites = ref([]);
const actionModalOpen = ref(false);
const actionModalType = ref("");
const actionModalItem = ref(null);
const actionModalName = ref("");
const actionModalPath = ref("");
const actionSubmitting = ref(false);

const normalizePath = (path) => String(path || "").replace(/\\/g, "/");
const favoritedPaths = computed(() => new Set(
    (favorites.value || [])
        .map((favorite) => normalizePath(favorite?.path))
        .filter(Boolean)
));

const isFavoritedPath = (path) => favoritedPaths.value.has(normalizePath(path));

const normalizeComparePath = (path) => {
    const normalized = normalizePath(path).replace(/\/+$/, "");

    if (/^[A-Za-z]:$/.test(normalized)) {
        return `${normalized}/`.toLowerCase();
    }

    return normalized.toLowerCase();
};

const isPathWithinDirectory = (targetPath, rootPath) => {
    const normalizedTargetPath = normalizeComparePath(targetPath);
    const normalizedRootPath = normalizeComparePath(rootPath);

    if (!normalizedTargetPath || !normalizedRootPath) {
        return false;
    }

    return normalizedTargetPath === normalizedRootPath
        || normalizedTargetPath.startsWith(`${normalizedRootPath}/`);
};

const getRelativeSegments = (targetPath, rootPath) => {
    const normalizedTargetPath = normalizePath(targetPath).replace(/\/+$/, "");
    const normalizedRootPath = normalizePath(rootPath).replace(/\/+$/, "");

    if (!normalizedTargetPath || !normalizedRootPath || !isPathWithinDirectory(normalizedTargetPath, normalizedRootPath)) {
        return [];
    }

    if (normalizeComparePath(normalizedTargetPath) === normalizeComparePath(normalizedRootPath)) {
        return [];
    }

    const relativePath = normalizedTargetPath.slice(normalizedRootPath.length).replace(/^\/+/, "");
    return relativePath ? relativePath.split("/").filter(Boolean) : [];
};

const findNodeByKey = (nodes, key = []) => {
    const normalizedKey = Array.isArray(key) ? key : [];

    if (normalizedKey.length === 0) {
        return null;
    }

    for (const node of nodes || []) {
        const nodeKey = Array.isArray(node?.key) ? node.key : [];

        if (nodeKey.length === normalizedKey.length && nodeKey.every((segment, index) => segment === normalizedKey[index])) {
            return node;
        }

        const nestedNode = findNodeByKey(node?.children, normalizedKey);
        if (nestedNode) {
            return nestedNode;
        }
    }

    return null;
};

const targetDirectoryHasSameName = (targetParentPath, entryName) => {
    const relativeSegments = getRelativeSegments(targetParentPath, warehouse_path.value);

    if (relativeSegments.length === 0) {
        return (file_data.value || []).some((node) => node?.info?.name === entryName);
    }

    const targetNode = findNodeByKey(file_data.value || [], relativeSegments);
    if (!targetNode?.is_folder) {
        return false;
    }

    return (targetNode.children || []).some((child) => child?.info?.name === entryName);
};

const isMoveConflictError = (error) => {
    const messageText = String(error?.message || error || "");
    return messageText.includes("目标路径已存在同名文件或文件夹")
        || messageText.includes("same name")
        || messageText.includes("already exists");
};

const buildItemPath = (key = []) => {
    return [warehouse_path.value || "", ...(Array.isArray(key) ? key : [])]
        .filter(Boolean)
        .join("/")
        .replace(/\\/g, "/");
};

const getBaseDisplayName = (item) => {
    const rawName = item?.info?.name || "";

    if (item?.is_folder) {
        return rawName;
    }

    const lastDotIndex = rawName.lastIndexOf(".");
    return lastDotIndex > 0 ? rawName.slice(0, lastDotIndex) : rawName;
};

const buildRenamedEntryName = (item, nextName) => {
    const trimmedName = String(nextName || "").trim();

    if (!trimmedName || item?.is_folder) {
        return trimmedName;
    }

    if (trimmedName.includes(".")) {
        return trimmedName;
    }

    const currentName = item?.info?.name || "";
    const lastDotIndex = currentName.lastIndexOf(".");

    if (lastDotIndex <= 0) {
        return trimmedName;
    }

    return `${trimmedName}${currentName.slice(lastDotIndex)}`;
};

const buildTargetPath = (parentPath, entryName) => {
    return [normalizePath(parentPath), entryName].filter(Boolean).join("/");
};

const actionModalTitle = computed(() => {
    if (actionModalType.value === "move") {
        return t("file.move");
    }

    if (actionModalType.value === "rename") {
        return t("file.rename");
    }

    return "";
});

const currentActionItemPath = computed(() => {
    return buildItemPath(actionModalItem.value?.key || []);
});

const loadFileTree = async () => {
    try {
        const { warehousePath, fileData } = await fetchFileTree();
        warehouse_path.value = warehousePath || "";
        file_data.value = Array.isArray(fileData) ? fileData : [];
        // 同步到共享 store，供 EditView 等组件使用
        sharedWarehousePath.value = warehouse_path.value;
        sharedFileData.value = file_data.value;
    } catch (error) {
        file_data.value = [];
        sharedFileData.value = [];
        message.error(error?.message || t('file.getFileListFailed'));
    }
};

const loadFavorites = async () => {
    try {
        favorites.value = await fetchFavorites();
    } catch (error) {
        favorites.value = [];
        message.error(error?.message || t("message.error"));
    }
};

let refreshTimer = null;

const refreshData = async () => {
    await Promise.all([loadFileTree(), loadFavorites()]);
};

const refreshOnFocus = async () => {
    await refreshData();
};

const handleFavoritesUpdated = () => {
    void loadFavorites();
};

onMounted(async () => {
    await refreshData();

    refreshTimer = window.setInterval(refreshData, 2000);
    window.addEventListener("focus", refreshOnFocus);
    document.addEventListener("visibilitychange", refreshOnFocus);
    window.addEventListener("simple-write:favorites-updated", handleFavoritesUpdated);
});

onBeforeUnmount(() => {
    if (refreshTimer) {
        window.clearInterval(refreshTimer);
        refreshTimer = null;
    }

    window.removeEventListener("focus", refreshOnFocus);
    document.removeEventListener("visibilitychange", refreshOnFocus);
    window.removeEventListener("simple-write:favorites-updated", handleFavoritesUpdated);
});

provide("data", file_data);
provide("warehousePath", warehouse_path);
provide("refreshFileTree", loadFileTree);
provide("favoritedPaths", favoritedPaths);

const CreateEntry = async ({ isFolder, parentKey = [] } = {}) => {
    try {
        await invokeCreateFileEntry({
            isFolder,
            parentKey,
        });
        await loadFileTree();
    } catch (error) {
        message.error(error?.message || t('message.error'));
    }
};

provide("createFileEntry", CreateEntry);

const resetActionModal = () => {
    actionModalOpen.value = false;
    actionModalType.value = "";
    actionModalItem.value = null;
    actionModalName.value = "";
    actionModalPath.value = "";
    actionSubmitting.value = false;
};

const openRenameDialog = (item) => {
    actionModalType.value = "rename";
    actionModalItem.value = item;
    actionModalName.value = getBaseDisplayName(item);
    actionModalPath.value = "";
    actionModalOpen.value = true;
};

const openMoveDialog = (item) => {
    actionModalType.value = "move";
    actionModalItem.value = item;
    actionModalName.value = "";
    actionModalPath.value = "";
    actionModalOpen.value = true;
};

const selectMovePath = async () => {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        return;
    }

    const selectedPath = await open({
        multiple: false,
        directory: true,
        defaultPath: warehouse_path.value || undefined,
    });

    if (typeof selectedPath === "string" && selectedPath.trim()) {
        if (!isPathWithinDirectory(selectedPath, warehouse_path.value)) {
            message.warning(t("file.pathMustBeWithinWarehouse"));
            return;
        }

        actionModalPath.value = selectedPath;
    }
};

const duplicateEntry = async (item) => {
    try {
        await invokeDuplicateFileEntry({ key: item?.key || [] });
        await loadFileTree();
    } catch (error) {
        message.error(error?.message || t("message.error"));
    }
};

const favoriteEntry = async (item) => {
    const itemPath = buildItemPath(item?.key || []);

    try {
        if (isFavoritedPath(itemPath)) {
            await removeFavorite({
                filePath: itemPath,
            });
        } else {
            await createFavorite({
                filePath: itemPath,
                itemType: item?.is_folder ? "folder" : "file",
            });
        }

        await loadFavorites();
        window.dispatchEvent(new CustomEvent("simple-write:favorites-updated"));
        message.success(t("message.success"));
    } catch (error) {
        message.error(error?.message || t("message.error"));
    }
};

const renameEntry = async () => {
    const item = actionModalItem.value;
    const newName = String(actionModalName.value || "").trim();

    if (!newName) {
        message.warning(t("file.pleaseEnterName"));
        return false;
    }

    const oldPath = buildItemPath(item?.key || []);
    const nextEntryName = buildRenamedEntryName(item, newName);

    if (!nextEntryName || nextEntryName === item?.info?.name) {
        message.warning(t("file.sameName"));
        return false;
    }

    const parentPath = oldPath.split("/").slice(0, -1).join("/");
    const nextPath = buildTargetPath(parentPath, nextEntryName);

    await invokeRenameFileEntry({
        key: item?.key || [],
        newName,
    });

    replaceTabPathPrefix(oldPath, nextPath);
    await loadFileTree();
    return true;
};

const moveEntry = async () => {
    const item = actionModalItem.value;
    const newParentPath = String(actionModalPath.value || "").trim();

    if (!newParentPath) {
        message.warning(t("file.selectNewPath"));
        return false;
    }

    const oldPath = buildItemPath(item?.key || []);
    const currentParentPath = oldPath.split("/").slice(0, -1).join("/");

    if (normalizePath(newParentPath) === normalizePath(currentParentPath)) {
        message.warning(t("file.samePath"));
        return false;
    }

    if (!isPathWithinDirectory(newParentPath, warehouse_path.value)) {
        message.warning(t("file.pathMustBeWithinWarehouse"));
        return false;
    }

    if (targetDirectoryHasSameName(newParentPath, item?.info?.name || "")) {
        message.warning(t("file.moveTargetExists"));
        return false;
    }

    const nextPath = buildTargetPath(newParentPath, item?.info?.name || "");

    await invokeMoveFileEntry({
        key: item?.key || [],
        newParentPath,
    });

    replaceTabPathPrefix(oldPath, nextPath);
    await loadFileTree();
    return true;
};

const confirmDuplicateOrDeleteEntry = (item, action) => {
    if (action === "duplicate") {
        Modal.confirm({
            title: t("file.createCopy"),
            content: t("file.confirmDuplicate"),
            okText: t("file.confirm"),
            cancelText: t("file.cancel"),
            async onOk() {
                try {
                    const itemPath = buildItemPath(item?.key || []);
                    await duplicateEntry(item);
                    message.success(t("message.success"));
                } catch (error) {
                    message.error(error?.message || t("message.error"));
                    throw error;
                }
            },
        }); 
    }else{
        Modal.confirm({
            title: t("file.delete"),
            content: t("file.confirmDelete"),
            okText: t("file.confirm"),
            cancelText: t("file.cancel"),
            async onOk() {
                try {
                    const itemPath = buildItemPath(item?.key || []);
                    await invokeDeleteFileEntry({ key: item?.key || [] });
                    closeTabsByPathPrefix(itemPath);
                    await loadFileTree();
                    message.success(t("message.success"));
                } catch (error) {
                    message.error(error?.message || t("message.error"));
                    throw error;
                }
            },
        }); 
    }
};

const handleConfirmAction = async () => {
    if (!actionModalItem.value) {
        resetActionModal();
        return;
    }

    actionSubmitting.value = true;

    try {
        let finished = false;

        if (actionModalType.value === "rename") {
            finished = await renameEntry();
        } else if (actionModalType.value === "move") {
            finished = await moveEntry();
        }

        if (!finished) {
            actionSubmitting.value = false;
            return;
        }

        message.success(t("message.success"));
        resetActionModal();
    } catch (error) {
        actionSubmitting.value = false;
        if (actionModalType.value === "move" && isMoveConflictError(error)) {
            message.warning(t("file.moveTargetExists"));
            return;
        }

        message.error(error?.message || t("message.error"));
    }
};

const handleFileContextAction = async ({ action, item }) => {
    if (!item) {
        return;
    }

    if (action === "duplicate") {
        confirmDuplicateOrDeleteEntry(item, action);
        return;
    }

    if (action === "rename") {
        openRenameDialog(item);
        return;
    }

    if (action === "move") {
        openMoveDialog(item);
        return;
    }

    if (action === "favorite") {
        await favoriteEntry(item);
        return;
    }

    if (action === "delete") {
        confirmDuplicateOrDeleteEntry(item, action);
    }
};

provide("handleFileContextAction", handleFileContextAction);

// ---- 批量导出 ----
const batchExportOpen = ref(false);
const batchLocked = ref(false);
const selectedExportPaths = ref(new Set());
const exportOutputName = ref(t('file.defaultOutputName'));
const exportOutputDir = ref("");
const exportSortKey = ref("name_asc");
const exportExpandedFolders = ref(new Set());
const exportAllExpanded = ref(true);

// 导出规则 — 每层级各自存储文件夹和文件两套规则
const exportDepthRules = ref([]);
// [{ folder: {prefix, suffix, blank, skip}, file: {prefix, suffix, blank, skip} }]
const selectedRuleLevel = ref(0);

// 最大路径深度（文件夹数+文件名 = parts.length）
const maxExportDepth = computed(() => {
    if (!batchLocked.value) return 0;
    const paths = selectedExportFilePaths.value;
    if (paths.length === 0) return 0;
    const wh = (warehouse_path.value || "").replace(/\\/g, "/").replace(/\/+$/, "");
    let max = 0;
    for (const p of paths) {
        const rel = p.startsWith(wh) ? p.slice(wh.length).replace(/^\//, "") : p;
        max = Math.max(max, rel.split("/").length);
    }
    return max;
});

const makeDefaultRule = () => ({ prefix: "", suffix: "", blank: true, skip: false });

const initExportRules = () => {
    const depth = maxExportDepth.value;
    const old = exportDepthRules.value;
    const rules = [];
    for (let i = 0; i < depth; i++) {
        rules.push({
            folder: { ...makeDefaultRule(), ...old[i]?.folder },
            file: { ...makeDefaultRule(), ...old[i]?.file },
        });
    }
    exportDepthRules.value = rules;
    if (selectedRuleLevel.value >= depth) {
        selectedRuleLevel.value = Math.max(0, depth - 1);
    }
};

const resetAllExportRules = () => {
    for (const rule of exportDepthRules.value) {
        Object.assign(rule.folder, makeDefaultRule());
        Object.assign(rule.file, makeDefaultRule());
    }
};

const exportDuplicateHint = computed(() => {
    if (!exportOutputDir.value || !exportOutputName.value) return false;
    const dir = exportOutputDir.value.replace(/\\/g, "/").replace(/\/+$/, "");
    const targetPath = dir + "/" + exportOutputName.value + ".txt";
    return exportableEntries.value.some(e => e.path.replace(/\\/g, "/") === targetPath);
});

const compareExportNodes = (a, b) => {
    const sortValue = exportSortKey.value;
    if (sortValue === "name_asc") {
        return a.info.name.localeCompare(b.info.name, 'zh-CN');
    }
    if (sortValue === "name_desc") {
        return b.info.name.localeCompare(a.info.name, 'zh-CN');
    }
    if (sortValue === "modify_time_asc") {
        return a.info.modify_time.localeCompare(b.info.modify_time);
    }
    if (sortValue === "modify_time_desc") {
        return b.info.modify_time.localeCompare(a.info.modify_time);
    }
    if (sortValue === "create_time_asc") {
        return a.info.create_time.localeCompare(b.info.create_time);
    }
    if (sortValue === "create_time_desc") {
        return b.info.create_time.localeCompare(a.info.create_time);
    }
    return 0;
};

const sortExportNodes = (nodes) => {
    return [...(nodes || [])].sort((a, b) => {
        if (a.is_folder !== b.is_folder) return a.is_folder ? -1 : 1;
        return compareExportNodes(a, b);
    });
};

const buildExportPathFromKey = (key = []) => {
    return [warehouse_path.value || "", ...(Array.isArray(key) ? key : [])]
        .filter(Boolean)
        .join("/")
        .replace(/\\/g, "/");
};

const folderHasTxtDescendants = (node) => {
    if (!node?.is_folder) {
        return /\.txt$/i.test(node?.info?.name || "");
    }
    return (node.children || []).some((child) => folderHasTxtDescendants(child));
};

const collectFolderSelectionPaths = (node, paths = []) => {
    const path = buildExportPathFromKey(node?.key || []);
    if (path) paths.push(path);
    for (const child of node?.children || []) {
        if (child?.is_folder) {
            collectFolderSelectionPaths(child, paths);
        } else if (/\.txt$/i.test(child?.info?.name || "")) {
            paths.push(buildExportPathFromKey(child.key || []));
        }
    }
    return paths;
};

const hasSelectedDescendant = (folderPath) => {
    for (const path of selectedExportPaths.value) {
        if (path === folderPath || path.startsWith(folderPath + "/")) return true;
    }
    return false;
};

const flattenExportTree = (nodes, depth = 0) => {
    const result = [];
    for (const node of sortExportNodes(nodes)) {
        const path = buildExportPathFromKey(node.key || []);
        const entry = {
            name: node.info.name,
            path,
            isFolder: node.is_folder,
            depth,
            key: node.key || [],
            children: node.children || [],
        };

        if (entry.isFolder) {
            const includeWhenUnlocked = true;
            const includeWhenLocked = hasSelectedDescendant(path);
            if ((!batchLocked.value && includeWhenUnlocked) || (batchLocked.value && includeWhenLocked)) {
                result.push(entry);
            }
            if (exportExpandedFolders.value.has(path)) {
                result.push(...flattenExportTree(node.children || [], depth + 1));
            }
            continue;
        }

        if (!/\.txt$/i.test(entry.name)) {
            continue;
        }

        if (!batchLocked.value || selectedExportPaths.value.has(path)) {
            result.push(entry);
        }
    }
    return result;
};

const exportableEntries = computed(() => flattenExportTree(file_data.value, 0));

const selectedExportFilePaths = computed(() => {
    const orderedFiles = [];
    const walk = (nodes) => {
        for (const node of sortExportNodes(nodes)) {
            const path = buildExportPathFromKey(node.key || []);
            if (node.is_folder) {
                walk(node.children || []);
            } else if (/\.txt$/i.test(node.info?.name || "") && selectedExportPaths.value.has(path)) {
                orderedFiles.push(path);
            }
        }
    };
    walk(file_data.value || []);
    return orderedFiles;
});

const getFolderTxtChildren = (folderPath) => {
    const result = [];
    const walk = (nodes) => {
        for (const node of nodes || []) {
            const path = buildExportPathFromKey(node.key || []);
            if (node.is_folder) {
                if (path === folderPath || path.startsWith(folderPath + "/")) {
                    walk(node.children || []);
                }
            } else if (/\.txt$/i.test(node.info?.name || "") && path.startsWith(folderPath + "/")) {
                result.push({ path });
            }
        }
    };
    walk(file_data.value || []);
    return result;
};

const isFolderAllChecked = (folderEntry) => {
    if (!folderEntry.isFolder) return false;
    const children = getFolderTxtChildren(folderEntry.path);
    if (children.length === 0) return selectedExportPaths.value.has(folderEntry.path);
    return selectedExportPaths.value.has(folderEntry.path) && children.every((e) => selectedExportPaths.value.has(e.path));
};

const isFolderIndeterminate = (folderEntry) => {
    if (!folderEntry.isFolder) return false;
    const children = getFolderTxtChildren(folderEntry.path);
    if (children.length === 0) return false;
    const checkedCount = children.filter((e) => selectedExportPaths.value.has(e.path)).length;
    return checkedCount > 0 && (checkedCount < children.length || !selectedExportPaths.value.has(folderEntry.path));
};

const toggleExportPath = (entry, checked) => {
    const newSet = new Set(selectedExportPaths.value);
    if (entry.isFolder) {
        for (const path of collectFolderSelectionPaths({
            key: entry.key,
            children: entry.children,
            is_folder: true,
        })) {
            if (checked) newSet.add(path);
            else newSet.delete(path);
        }
    } else if (entry.path.endsWith(".txt")) {
        if (checked) newSet.add(entry.path);
        else newSet.delete(entry.path);
    }
    selectedExportPaths.value = newSet;
};

const setExportSort = (value) => {
    exportSortKey.value = value;
};

const collectAllFolderPaths = (nodes, acc = []) => {
    for (const node of nodes || []) {
        if (!node?.is_folder) continue;
        const path = buildExportPathFromKey(node.key || []);
        if (path) acc.push(path);
        collectAllFolderPaths(node.children || [], acc);
    }
    return acc;
};

const toggleExportFolderExpand = (folderPath) => {
    const next = new Set(exportExpandedFolders.value);
    if (next.has(folderPath)) next.delete(folderPath);
    else next.add(folderPath);
    exportExpandedFolders.value = next;
    exportAllExpanded.value = collectAllFolderPaths(file_data.value || []).every((path) => next.has(path));
};

const toggleExportExpandAll = () => {
    if (exportAllExpanded.value) {
        exportExpandedFolders.value = new Set();
        exportAllExpanded.value = false;
    } else {
        exportExpandedFolders.value = new Set(collectAllFolderPaths(file_data.value || []));
        exportAllExpanded.value = true;
    }
};

const onLockChange = () => {
    // 锁定由 checkbox v-model 控制，锁定后只能查看已选文件
    if (batchLocked.value && selectedExportPaths.value.size === 0) {
        message.warning(t('file.pleaseSelectFiles'));
        batchLocked.value = false;
        return;
    }
    if (batchLocked.value) {
        initExportRules();
    }
};

const selectExportOutputDir = async () => {
    const { open: openDialog } = await import("@tauri-apps/plugin-dialog");
    const selected = await openDialog({ multiple: false, directory: true, defaultPath: warehouse_path.value || undefined });
    if (typeof selected === "string" && selected.trim()) {
        const sel = selected.replace(/\\/g, "/");
        const wh = (warehouse_path.value || "").replace(/\\/g, "/");
        if (!sel.startsWith(wh)) { message.warning(t('file.outputDirMustInWarehouse')); return; }
        exportOutputDir.value = selected;
    }
};

const openBatchExportModal = () => {
    exportOutputDir.value = warehouse_path.value || "";
    selectedExportPaths.value = new Set();
    exportOutputName.value = t('file.defaultOutputName');
    batchLocked.value = false;
    exportSortKey.value = "name_asc";
    exportExpandedFolders.value = new Set(collectAllFolderPaths(file_data.value || []));
    exportAllExpanded.value = true;
    exportDepthRules.value = [];
    selectedRuleLevel.value = 0;
    batchExportOpen.value = true;
};

// 监听来自 App.vue 侧边栏按钮的事件
onMounted(() => {
    window.addEventListener("simple-write:open-batch-export", openBatchExportModal);
});
onBeforeUnmount(() => {
    window.removeEventListener("simple-write:open-batch-export", openBatchExportModal);
});

const handleBatchExport = async () => {
    const paths = [...selectedExportFilePaths.value];
    if (paths.length === 0) { message.warning(t('file.pleaseSelectFiles')); return; }
    if (!exportOutputDir.value.trim()) { message.warning(t('file.pleaseSelectOutputDir')); return; }
    if (exportDuplicateHint.value) { message.warning(t('file.exportDuplicateHintFull')); return; }

    const dir = exportOutputDir.value.replace(/\\/g, "/").replace(/\/+$/, "");
    const outputPath = dir + "/" + exportOutputName.value;

    try {
        // 手动拼接导出（支持自定义命名规则）
        const allTxt = paths.map((path) => ({
            path,
            isFolder: false,
        }));
        let output = "";
        let previousFolderParts = [];

        for (let i = 0; i < allTxt.length; i++) {
            const f = allTxt[i];
            let content = "";
            try { content = await invoke("get_file_content", { filePath: f.path }); } catch {}

            // 获取相对路径段
            const wh = (warehouse_path.value || "").replace(/\\/g, "/").replace(/\/+$/, "");
            const rel = f.path.replace(/\\/g, "/");
            let relPath = rel.startsWith(wh) ? rel.slice(wh.length).replace(/^\//, "") : rel;
            const parts = relPath.split("/");

            // 文件夹名作为卷名（跳过文件名自身的部分）
            const folderParts = parts.slice(0, -1);
            let commonDepth = 0;
            while (
                commonDepth < previousFolderParts.length &&
                commonDepth < folderParts.length &&
                previousFolderParts[commonDepth] === folderParts[commonDepth]
            ) {
                commonDepth++;
            }
            for (let d = commonDepth; d < folderParts.length; d++) {
                const rule = exportDepthRules.value[d]?.folder || {};
                if (!rule.skip) {
                    const prefix = rule.prefix || "";
                    const suffix = rule.suffix || "";
                    if (output) output += "\n";
                    output += `# ${prefix}${folderParts[d]}${suffix}\n`;
                    if (rule.blank !== false) output += "\n";
                }
            }
            previousFolderParts = folderParts;

            // 文件名作为章节名（使用该文件所在深度的 file 规则）
            const fileDepth = parts.length - 1;
            const curFileRule = exportDepthRules.value[fileDepth]?.file || {};
            if (!curFileRule.skip) {
                const fileName = parts[parts.length - 1].replace(/\.txt$/i, "");
                const prefix = curFileRule.prefix || "";
                const suffix = curFileRule.suffix || "";
                if (curFileRule.blank !== false && output) output += "\n";
                output += `## ${prefix}${fileName}${suffix}\n\n`;
            }

            output += content.trim();
            if (i < allTxt.length - 1) output += "\n\n";
        }

        await saveFileContent(outputPath + ".txt", output);
        window.dispatchEvent(new CustomEvent('simple-write:file-updated', { detail: { path: outputPath + ".txt" } }));
        message.success(t('file.exportSuccess'));
        batchExportOpen.value = false;
    } catch (e) {
        message.error(e?.message || t('file.exportFailed'));
    }
};

const sort_all = ref("");
provide("sort_all", sort_all);

const SortAll = (e) => {
    sort_all.value = e;
};

const expand_all = ref(false);
provide("expand_all", expand_all);

const ExpandAll = () => {
    expand_all.value = !expand_all.value;
};
</script>

<style scoped>
.file-list-toolbar-wrap {
    margin: 5px 0;
}

.file-list-toolbar {
    width: 100%;
    align-items: center;
}

.file-list-toolbar-button {
    width: 34px;
    height: 34px;
    min-width: 34px;
    padding: 0;
    border-radius: 8px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
}

/* 批量导出 */
.batch-export-body { display: flex; flex-direction: column; gap: 14px; }

.batch-export-tree { border: 1px solid var(--border-color, #e8e8e8); border-radius: 8px; overflow: hidden; }

.batch-export-tree-header {
    padding: 6px 12px; font-size: 13px; color: var(--text-secondary, #595959);
    border-bottom: 1px solid var(--border-secondary, #f0f0f0);
    background: var(--bg-secondary, #fafafa);
    display: flex; justify-content: space-between; align-items: center;
}

.batch-export-tree-scroll { max-height: 220px; overflow: auto; }

.batch-export-item { padding: 3px 8px; border-bottom: 1px solid var(--border-secondary, #f0f0f0); }
.batch-export-item:last-child { border-bottom: none; }
.batch-export-folder-toggle {
    display: inline-flex;
    width: 16px;
    margin-right: 4px;
    align-items: center;
    justify-content: center;
    color: var(--text-tertiary, #8c8c8c);
    cursor: pointer;
    user-select: none;
}
.batch-export-folder-toggle--placeholder {
    cursor: default;
}

.batch-export-empty { padding: 24px; text-align: center; color: #bfbfbf; font-size: 13px; }

.batch-export-lock { padding: 0 4px; }

.batch-export-rules {
    border: 1px solid var(--border-color, #e8e8e8); border-radius: 8px;
    padding: 8px 12px;
}
.batch-export-rules-header {
    display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;
}
.batch-export-rules-title { font-size: 13px; font-weight: 600; color: var(--text-primary, #262626); }
.batch-export-rules--disabled { opacity: 0.5; pointer-events: none; }

.batch-export-rule-level {
    display: flex; align-items: center; gap: 8px; margin-bottom: 8px;
    font-size: 13px; color: var(--text-primary, #262626);
}
.batch-export-rule-label { white-space: nowrap; }

.batch-export-rule-row {
    display: flex; align-items: center; gap: 6px; font-size: 13px;
    color: var(--text-primary, #262626); margin-bottom: 6px;
}
.batch-export-rule-row:last-child { margin-bottom: 0; }

.batch-export-options { display: flex; flex-direction: column; gap: 10px; }

.batch-export-option {
    display: flex; align-items: center; gap: 8px;
    font-size: 13px; color: var(--text-primary, #262626);
}

.batch-export-hint {
    font-size: 12px; color: #faad14; padding-left: 4px;
}
</style>
