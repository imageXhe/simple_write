<template>
  <div class="bookmark-container">
    <a-empty v-if="bookmarks.length === 0" :description="t('file.noData')" :image="simpleImage" />
    <div v-else>
      <div class="bookmark-search">
        <a-input
          v-model:value="searchText"
          :placeholder="t('file.search')"
          allow-clear
          size="small"
        />
      </div>
      <a-empty v-if="filteredBookmarks.length === 0" :description="t('file.noData')" :image="simpleImage" />
      <a-dropdown v-for="item in filteredBookmarks" :key="item.filePath" :trigger="['contextmenu']">
        <div class="bookmark-item">
          <a-button type="text" class="bookmark-item__button" @click="openBookmark(item)">
            <span class="bookmark-item__name">{{ getBookmarkName(item) }}</span>
            <span class="bookmark-item__path">{{ getRelPath(item.filePath) }}</span>
          </a-button>
        </div>
        <template #overlay>
          <a-menu class="dropdown-menu-bordered">
            <a-menu-item key="delete" @click="handleDeleteBookmark(item)" style="color: red;">
              {{ t('file.deleteBookmark') }}
            </a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>
    </div>
  </div>
</template>

<script setup>
import { inject, onBeforeUnmount, onMounted, ref, computed } from "vue";
import { Empty, message, Modal } from "ant-design-vue";
import { fetchBookmarks, removeBookmark } from "./bookmarkActions";
import { useI18n } from "../locales";
import { warehousePath } from "../stores/fileStore";

const simpleImage = Empty.PRESENTED_IMAGE_SIMPLE;
const { t } = useI18n();

const bookmarks = ref([]);
const searchText = ref("");
const openFile = inject("openFile", null);

const filteredBookmarks = computed(() => {
  const keyword = searchText.value.trim().toLowerCase();
  if (!keyword) return bookmarks.value;
  return bookmarks.value.filter((item) => {
    const name = getBookmarkName(item).toLowerCase();
    const path = getRelPath(item.filePath).toLowerCase();
    return name.includes(keyword) || path.includes(keyword);
  });
});

const getBookmarkName = (item) => {
  if (item?.name) {
    return String(item.name);
  }

  const path = String(item?.filePath || "").replace(/\\/g, "/");
  const rawName = path.split("/").filter(Boolean).pop() || path;

  const lastDotIndex = rawName.lastIndexOf(".");
  return lastDotIndex > 0 ? rawName.slice(0, lastDotIndex) : rawName;
};

const getRelPath = (absPath) => {
  const abs = (absPath || '').replace(/\\/g, '/');
  const wh = (warehousePath.value || '').replace(/\\/g, '/');
  if (abs.startsWith(wh)) {
    const rel = abs.slice(wh.length).replace(/^\//, '');
    return rel || abs;
  }
  return abs;
};

let refreshTimer = null;

const loadBookmarks = async () => {
  try {
    bookmarks.value = await fetchBookmarks();
  } catch (error) {
    bookmarks.value = [];
    message.error(error?.message || t("message.error"));
  }
};

// 从路径提取纯文件名（不含扩展名），用于标签页标题
const getFileNameFromPath = (filePath) => {
  const path = String(filePath || "").replace(/\\/g, "/");
  const rawName = path.split("/").filter(Boolean).pop() || path;
  const lastDotIndex = rawName.lastIndexOf(".");
  return lastDotIndex > 0 ? rawName.slice(0, lastDotIndex) : rawName;
};

const openBookmark = (item) => {
  if (item?.type === "folder") {
    message.warning(t("message.warning"));
    return;
  }

  if (typeof openFile !== "function" || !item?.filePath) {
    return;
  }

  openFile({
    filePath: item.filePath,
    fileName: getFileNameFromPath(item.filePath),
  });
};

const handleDeleteBookmark = (item) => {
  Modal.confirm({
    title: t('file.deleteBookmark'),
    content: t('file.confirmDeleteBookmark'),
    okText: t('file.confirm'),
    cancelText: t('file.cancel'),
    async onOk() {
      try {
        await removeBookmark({ filePath: item.filePath });
        await loadBookmarks();
        window.dispatchEvent(new CustomEvent("simple-write:bookmarks-updated"));
        message.success(t("message.success"));
      } catch (error) {
        message.error(error?.message || t("message.error"));
        throw error;
      }
    },
  });
};

const handleBookmarksUpdated = () => {
  void loadBookmarks();
};

onMounted(async () => {
  await loadBookmarks();
  refreshTimer = window.setInterval(loadBookmarks, 2000);
  window.addEventListener("simple-write:bookmarks-updated", handleBookmarksUpdated);
});

onBeforeUnmount(() => {
  if (refreshTimer) {
    window.clearInterval(refreshTimer);
    refreshTimer = null;
  }

  window.removeEventListener("simple-write:bookmarks-updated", handleBookmarksUpdated);
});
</script>

<style scoped>
.bookmark-container {
  height: 100%;
  padding: 5px;
  overflow-y: auto;
}

.bookmark-search {
  margin-bottom: 8px;
}

.bookmark-item {
  padding: 3px 0;
  border-bottom: 1px solid var(--border-secondary, #f0f0f0);
}

.bookmark-item__button {
  width: 100%;
  height: auto;
  display: flex;
  align-items: flex-start;
  gap: 4px;
  flex-direction: column;
}

.bookmark-item__name {
  width: 100%;
  color: var(--text-primary, #262626);
  text-align: left;
  font-size: 14px;
  line-height: 1.4;
  word-break: break-word;
}

.bookmark-item__path {
  width: 100%;
  color: var(--text-tertiary, #8c8c8c);
  text-align: left;
  font-size: 12px;
  line-height: 1.4;
  word-break: break-all;
}
</style>
