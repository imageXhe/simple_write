<template>
  <div class="favorite-container">
    <a-empty v-if="favorites.length === 0" :description="t('file.noData')" :image="simpleImage" />
    <div v-else>
      <div class="favorite-search">
        <a-input
          v-model:value="searchText"
          :placeholder="t('file.search')"
          allow-clear
          size="small"
        />
      </div>
      <a-empty v-if="filteredFavorites.length === 0" :description="t('file.noData')" :image="simpleImage" />
      <a-dropdown v-for="item in filteredFavorites" :key="item.path" :trigger="['contextmenu']">
        <div class="favorite-item">
          <a-button type="text" class="favorite-item__button" @click="openFavorite(item)">
            <span class="favorite-item__name">
              <span v-if="getItemType(item) === 'folder'" style="margin-right:4px;">📁</span>
              {{ getFavoriteName(item) }}
            </span>
            <span class="favorite-item__path">{{ getRelPath(item.path) }}</span>
          </a-button>
        </div>
        <template #overlay>
          <a-menu class="dropdown-menu-bordered">
            <a-menu-item key="unfavorite" @click="handleUnfavorite(item)" style="color: red;">
              {{ t('file.unfavorite') }}
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
import { fetchFavorites, removeFavorite } from "./favoriteActions";
import { useI18n } from "../locales";
import { warehousePath } from "../stores/fileStore";

const simpleImage = Empty.PRESENTED_IMAGE_SIMPLE;
const { t } = useI18n();

const favorites = ref([]);
const searchText = ref("");
const openFile = inject("openFile", null);

const filteredFavorites = computed(() => {
  const keyword = searchText.value.trim().toLowerCase();
  if (!keyword) return favorites.value;
  return favorites.value.filter((item) => {
    const name = getFavoriteName(item).toLowerCase();
    return name.includes(keyword);
  });
});

const getItemType = (item) => String(item?.type || item?.itemType || item?.item_type || "file");

const getFavoriteName = (item) => {
  const path = String(item?.path || "").replace(/\\/g, "/");
  const rawName = path.split("/").filter(Boolean).pop() || path;
  if (getItemType(item) === "folder") return rawName;
  const lastDotIndex = rawName.lastIndexOf(".");
  return lastDotIndex > 0 ? rawName.slice(0, lastDotIndex) : rawName;
};

// 相对路径显示
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

const loadFavorites = async () => {
  try {
    favorites.value = await fetchFavorites();
  } catch (error) {
    favorites.value = [];
    message.error(error?.message || t("message.error"));
  }
};

const openFavorite = (item) => {
  const itemType = getItemType(item);
  if (itemType === "folder") {
    window.dispatchEvent(new CustomEvent('simple-write:switch-view', { detail: { view: 'filelist' } }));
    window.dispatchEvent(new CustomEvent('simple-write:expand-folder', {
      detail: { path: item.path, name: getFavoriteName(item) },
    }));
    return;
  }

  if (typeof openFile !== "function" || !item?.path) return;
  openFile({ filePath: item.path, fileName: getFavoriteName(item) });
};

const handleUnfavorite = (item) => {
  Modal.confirm({
    title: t('file.unfavorite'),
    content: t('file.confirmUnfavorite'),
    okText: t('file.confirm'),
    cancelText: t('file.cancel'),
    async onOk() {
      try {
        await removeFavorite({ filePath: item.path });
        await loadFavorites();
        window.dispatchEvent(new CustomEvent("simple-write:favorites-updated"));
        message.success(t("message.success"));
      } catch (error) {
        message.error(error?.message || t("message.error"));
        throw error;
      }
    },
  });
};

const handleFavoritesUpdated = () => {
  void loadFavorites();
};

onMounted(async () => {
  await loadFavorites();
  refreshTimer = window.setInterval(loadFavorites, 2000);
  window.addEventListener("simple-write:favorites-updated", handleFavoritesUpdated);
});

onBeforeUnmount(() => {
  if (refreshTimer) {
    window.clearInterval(refreshTimer);
    refreshTimer = null;
  }

  window.removeEventListener("simple-write:favorites-updated", handleFavoritesUpdated);
});
</script>

<style scoped>
.favorite-container {
  height: 100%;
  padding: 5px;
  overflow-y: auto;
}

.favorite-search {
  margin-bottom: 8px;
}

.favorite-item {
  padding: 3px 0;
  border-bottom: 1px solid var(--border-secondary, #f0f0f0);
}

.favorite-item__button {
  width: 100%;
  height: auto;
  display: flex;
  align-items: flex-start;
  gap: 4px;
  flex-direction: column;
}

.favorite-item__name {
  width: 100%;
  color: var(--text-primary, #262626);
  text-align: left;
  font-size: 14px;
  line-height: 1.4;
  word-break: break-word;
}

.favorite-item__path {
  width: 100%;
  color: var(--text-tertiary, #8c8c8c);
  text-align: left;
  font-size: 12px;
  line-height: 1.4;
  word-break: break-all;
}
</style>
