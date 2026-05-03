<template>
    <div v-for="item in displayNodes" :key="item.id" class="file-panel__node" :class="{ 'file-panel__node--flash': item.id === flashFolderId }">
        <div class="file-panel__row">
            <a-dropdown :trigger="['contextmenu']" overlay-class-name="file-panel-menu">
                <div class="file_name_trigger">
                    <a-button
                        ghost
                        @click="ClickFile(item)"
                        class="file_name_button"
                        size="small"
                    >
                        <div class="file_name_content" :style="contentIndentStyle">
                            <RightOutlined v-if="item.is_folder && !IsExpand(item.id)" class="file_name_icon" />
                            <DownOutlined v-if="item.is_folder && IsExpand(item.id)" class="file_name_icon" />
                            <DownOutlined v-if="!item.is_folder" style="opacity: 0;" class="file_name_icon" />
                            <span class="file_name_text">{{ getDisplayName(item).name }}</span>
                            <StarFilled v-if="isFavorited(item)" class="file_name_favorite" />
                            <span v-if="getDisplayName(item).suffix" class="file_name_suffix">
                                {{ getDisplayName(item).suffix }}
                            </span>
                        </div>
                    </a-button>
                </div>
                <template #overlay>
                    <a-menu class="dropdown-menu-bordered">
                        <div v-if="item.is_folder">
                            <!-- 新建 文件夹 文件 -->
                            <a-menu-item key="1" :icon="h(FolderAddOutlined)" @click="handleCreateEntry(item, true)">
                                {{ t('file.newFolder') }}
                            </a-menu-item>
                            <a-menu-item key="2" :icon="h(FormOutlined)" @click="handleCreateEntry(item, false)">
                                {{ t('file.newNote') }}
                            </a-menu-item>
                            <a-menu-divider />
                            <!-- <a-menu-item key="3" :icon="h(InfoCircleOutlined)">移动</a-menu-item>
                            <a-menu-item key="4" :icon="h(InfoCircleOutlined)">查找</a-menu-item>
                            <a-menu-item key="5" :icon="h(StarOutlined)">收藏</a-menu-item> -->
                        </div>
                        <div v-if="!item.is_folder">
                            <!-- 新标签打开 新标签组打开 新窗口打开 副本 -->
                            <!-- <a-menu-item key="6" :icon="h(InfoCircleOutlined)">{{ t('file.newTab') }}</a-menu-item>
                            <a-menu-item key="7" :icon="h(InfoCircleOutlined)">{{ t('file.newTabGroup') }}</a-menu-item>
                            <a-menu-item key="8" :icon="h(InfoCircleOutlined)">{{ t('file.newWindow') }}</a-menu-item>
                            <a-menu-divider /> -->
                            <a-menu-item key="9" :icon="h(CopyOutlined)" @click="handleContextAction('duplicate', item)">{{ t('file.createCopy') }}</a-menu-item>
                            <!-- <a-menu-item key="12" :icon="h(InfoCircleOutlined)">合并</a-menu-item> -->
                        </div>
                        <div>
                            <!-- 移动 收藏 重命名 删除 -->
                            <a-menu-item v-if="item.is_folder" key="9-folder" :icon="h(CopyOutlined)" @click="handleContextAction('duplicate', item)">{{ t('file.createCopy') }}</a-menu-item>
                            <a-menu-item key="10" :icon="h(SvgIcon, { raw: svgIcons.move })" @click="handleContextAction('move', item)">{{ t('file.move') }}</a-menu-item>
                            <a-menu-item key="11" :icon="h(StarOutlined)" :class="{ 'file-panel-menu-item--favorited': isFavorited(item) }" @click="handleContextAction('favorite', item)">{{ t('file.favorite') }}</a-menu-item>
                            <a-menu-divider />
                            <a-menu-item key="13" :icon="h(EditOutlined)" @click="handleContextAction('rename', item)">{{ t('file.rename') }}</a-menu-item>
                            <a-menu-item key="14" :icon="h(DeleteOutlined)" @click="handleContextAction('delete', item)" style="color: red;">{{ t('file.delete') }}</a-menu-item>
                        </div>
                    </a-menu>
                </template>
            </a-dropdown>
        </div>

        <div v-if="item.is_folder && IsExpand(item.id)" class="file-panel__children">
            <p
                v-if="!item.children || item.children.length === 0"
                class="noFiles-text"
            >
                <InfoCircleOutlined></InfoCircleOutlined>
                {{ t('file.noFiles') }}
            </p>
            <div v-else>
                <FilePanel
                    :nodes="item.children"
                    :depth="depth + 1"
                    :path-segments="props.pathSegments.concat([item.info.name])"
                />
            </div>
        </div>

        <a-divider style="margin: 0; padding:1px 0" />
    </div>
</template>

<script setup>
import { computed, h, inject, ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import {
    DownOutlined,
    RightOutlined,
    InfoCircleOutlined,
    FolderAddOutlined,
    FormOutlined,
    StarOutlined,
    StarFilled,
    DeleteOutlined,
    EditOutlined,
    CopyOutlined,
} from "@ant-design/icons-vue";
import svgIcons from "../assets/icons";
import SvgIcon from "../components/SvgIcon.vue";
import { useI18n } from "../locales";

const { t } = useI18n();

const props = defineProps({
    nodes: {
        type: Array,
        default: undefined,
    },
    depth: {
        type: Number,
        default: 0,
    },
    pathSegments: {
        type: Array,
        default: () => [],
    },
});

const injectedData = inject("data", ref([]));
const sort = inject("sort_all", ref(""));
const expand = inject("expand_all", ref(false));
const createFileEntry = inject("createFileEntry", null);
const warehousePath = inject("warehousePath", ref(""));
const openFile = inject("openFile", null);
const handleFileContextAction = inject("handleFileContextAction", null);
const favoritedPaths = inject("favoritedPaths", computed(() => new Set()));

const sourceNodes = computed(() => props.nodes ?? injectedData.value ?? []);

const sortedNodes = computed(() => {
    const nodes = [...sourceNodes.value];
    const sortValue = sort.value;

    if (sortValue === "name_asc") {
        nodes.sort((a, b) => a.info.name.localeCompare(b.info.name));
    } else if (sortValue === "name_desc") {
        nodes.sort((a, b) => a.info.name.localeCompare(b.info.name)).reverse();
    } else if (sortValue === "modify_time_asc") {
        nodes.sort((a, b) => a.info.modify_time.localeCompare(b.info.modify_time));
    } else if (sortValue === "modify_time_desc") {
        nodes.sort((a, b) => a.info.modify_time.localeCompare(b.info.modify_time)).reverse();
    } else if (sortValue === "create_time_asc") {
        nodes.sort((a, b) => a.info.create_time.localeCompare(b.info.create_time));
    } else if (sortValue === "create_time_desc") {
        nodes.sort((a, b) => a.info.create_time.localeCompare(b.info.create_time)).reverse();
    }

    return nodes;
});

const displayNodes = computed(() => sortedNodes.value);
const expandedIds = ref([]);
const flashFolderId = ref('');

// 根据路径在文件树中查找文件夹 id
const findFolderIdByPath = (nodes, targetPath) => {
    const norm = (p) => (p || '').replace(/\\/g, '/').replace(/\/+$/, '');
    for (const node of nodes) {
        if (!node?.info) continue;
        const nodePath = norm(buildFilePath([...(node.key || [])]));
        if (nodePath === norm(targetPath)) return node.id;
        if (node.children?.length) {
            const found = findFolderIdByPath(node.children, targetPath);
            if (found) return found;
        }
    }
    return null;
};

// 展开并闪烁文件夹
const expandAndFlashFolder = (folderPath) => {
    const id = findFolderIdByPath(injectedData.value, folderPath);
    if (!id) return;
    // 展开（如果未展开）
    if (!expandedIds.value.includes(id)) {
        expandedIds.value.push(id);
    }
    // 闪烁提示
    flashFolderId.value = id;
    nextTick(() => {
        setTimeout(() => { flashFolderId.value = ''; }, 1200);
    });
};

const handleExpandFolderEvent = (e) => {
    const { path } = e.detail || {};
    if (path) expandAndFlashFolder(path);
};

onMounted(() => {
    window.addEventListener('simple-write:expand-folder', handleExpandFolderEvent);
});

onUnmounted(() => {
    window.removeEventListener('simple-write:expand-folder', handleExpandFolderEvent);
});

const contentIndentStyle = computed(() => ({
    paddingLeft: `${props.depth * 16 + 8}px`,
}));

const buildFilePath = (segments) => {
    return [warehousePath.value || "", ...segments].filter(Boolean).join("/").replace(/\\/g, "/");
};

const splitDisplayName = (name) => {
    const rawName = typeof name === "string" ? name : "";
    const lastDotIndex = rawName.lastIndexOf(".");

    if (lastDotIndex <= 0) {
        return {
            name: rawName,
            suffix: "",
        };
    }

    return {
        name: rawName.slice(0, lastDotIndex),
        suffix: rawName.slice(lastDotIndex + 1).toUpperCase(),
    };
};

const getDisplayName = (item) => {
    if (item?.is_folder) {
        return {
            name: item?.info?.name || "",
            suffix: "",
        };
    }

    return splitDisplayName(item?.info?.name || "");
};

const isFavorited = (item) => {
    const itemPath = buildFilePath([...props.pathSegments, item?.info?.name || ""]);
  return favoritedPaths.value.has(itemPath);
};

const syncExpandedIds = () => {
    expandedIds.value = displayNodes.value
        .filter((item) => item.is_folder)
        .map((item) => item.id);
};

watch(
    expand,
    (expandValue) => {
        if (expandValue) {
            syncExpandedIds();
        } else {
            expandedIds.value = [];
        }
    },
    { immediate: true }
);

watch(displayNodes, () => {
    if (expand.value) {
        syncExpandedIds();
    }
});

const ClickFile = (item) => {
    if (!item.is_folder) {
        if (typeof openFile === "function") {
            const displayName = getDisplayName(item);
            openFile({
                filePath: buildFilePath([...props.pathSegments, item.info.name]),
                fileName: displayName.name,
            });
        }
        return;
    }

    if (IsExpand(item.id)) {
        const index = expandedIds.value.indexOf(item.id);
        if (index !== -1) {
            expandedIds.value.splice(index, 1);
        }
    } else {
        expandedIds.value.push(item.id);
    }
};

const IsExpand = (id) => {
    return expandedIds.value.includes(id);
};

const handleCreateEntry = async (item, isFolder) => {
    if (!createFileEntry) {
        return;
    }

    await createFileEntry({
        isFolder,
        parentKey: item.key ?? [],
    });
};

const handleContextAction = async (action, item) => {
    if (typeof handleFileContextAction !== "function") {
        return;
    }

    await handleFileContextAction({
        action,
        item,
    });
};
</script>

<style scoped>
.file-panel__node {
    width: 100%;
}
.file_panel__row {
    width: 100%;
    box-sizing: border-box;
}
.file_name_trigger {
    width: 100%;
}
.file_name_button {
    width: 100%;
    color: var(--text-primary, #262626);
    text-align: start;
    padding: 0;
    height: 35px;
    text-overflow: ellipsis;
    font-size: 14px;
    overflow: hidden;
    box-sizing: border-box;
}
.file_name_content {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    box-sizing: border-box;
}
.file_name_text {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.file_name_suffix {
    flex-shrink: 0;
    margin: 0 5px;
    color: var(--text-tertiary, #8c8c8c);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.04em;
    white-space: nowrap;
}
.file_name_favorite {
    flex-shrink: 0;
    /* margin-left: 5px;
    margin-right: 5px; */
    padding : 0 5px;
    color: #fadb14;
    font-size: 13px;
}
.file_panel__children {
    padding-left: 0;
}
.file_panel__empty {
    color: var(--text-tertiary, #8c8c8c);
    text-align: center;
    font-size: small;
    margin: 5px;
}
.file_name_icon {
    margin: 0 5px;
}
:deep(.ant-collapse > .ant-collapse-item > .ant-collapse-header) {
    padding: 0 5px;
}
:deep(.ant-collapse .ant-collapse-content > .ant-collapse-content-box) {
    padding: 0;
}
:deep(.ant-collapse > .ant-collapse-item > .ant-collapse-header) {
    padding: 0 5px;
}
:deep(.ant-collapse .ant-collapse-content > .ant-collapse-content-box) {
    padding: 0;
}

.noFiles-text{
    color: var(--text-tertiary, #8c8c8c);
    text-align: center;
    font-size: small;
    margin: 5px;
}

.file-panel__node--flash {
    animation: file-panel-flash 0.3s ease-in-out 2;
}

@keyframes file-panel-flash {
    0%, 100% { background-color: transparent; }
    50% { background-color: #b1ddff; border-radius: 4px; }
}
</style>

<style>
/* 右键菜单收藏按钮（dropdown overlay 渲染到 body，必须全局样式） */
.file-panel-menu .file-panel-menu-item--favorited,
.file-panel-menu .file-panel-menu-item--favorited .ant-menu-title-content {
    color: #faad14 !important;
}
</style>
