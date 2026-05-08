<template>
<div class="file-list-toolbar-wrap">
        <a-flex class="file-list-toolbar" justify="space-evenly" align="center">
            <a-tooltip :title="t('file.newNote')" :arrow="false" placement="bottom">
                <a-button class="file-list-toolbar-button" @click="CreateEntry({ isFolder: false })">
                    <FormOutlined />
                </a-button>
            </a-tooltip>
            <a-tooltip :title="t('file.newFolder')" :arrow="false" placement="bottom">
                <a-button class="file-list-toolbar-button" @click="CreateEntry({ isFolder: true })">
                    <FolderAddOutlined />
                </a-button>
            </a-tooltip>
            <a-tooltip :title="t('file.sort')" :arrow="false" placement="bottom">
                <a-dropdown :trigger="['click']" placement="rightBottom">
                    <a-button class="file-list-toolbar-button">
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
                <a-button class="file-list-toolbar-button" @click="ExpandAll">
                    <ColumnHeightOutlined />
                </a-button>
            </a-tooltip>
        </a-flex>
    </div>
    <a-divider style="margin: 0;" />
    <div style="overflow: auto;height: calc(100% - 51px);">
        <FilePanel />
    </div>

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
import { fileData as sharedFileData, warehousePath as sharedWarehousePath } from "../stores/fileStore";

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
</style>
