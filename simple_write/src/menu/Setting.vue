<template>
  <a-tooltip :title="t('common.settings')" placement="right" :arrow="false">
      <a-button type="text" class="sider-button" @click="openSettings">
          <SettingOutlined />
      </a-button>
  </a-tooltip>

  <div>
    <a-modal
      :title="t('common.settings')"
      v-model:open="visible"
      :footer="null"
      :maskClosable="false"
      width="480px"
    >
      <div class="settings-body">
        <!-- 配置档案选择 -->
        <div class="settings-section">
          <div class="settings-label">{{ t('settings.configProfile') }}</div>
          <div class="config-selector">
            <a-select
              v-model:value="currentConfigName"
              style="flex:1"
              @change="onConfigSwitch"
            >
              <a-select-option v-for="name in configNames" :key="name" :value="name">{{ name }}</a-select-option>
            </a-select>
            <a-button size="small" @click="onAddConfig">+</a-button>
          </div>
          <div class="config-actions" style="margin-top:8px;display:flex;gap:8px;">
            <a-input v-model:value="configRenameInput" :placeholder="currentConfigName" size="small" style="flex:1" />
            <a-button size="small" type="primary" ghost @click="onSaveConfig">{{ t('settings.saveToConfig') }}</a-button>
            <a-button size="small" @click="onApplyConfig">{{ t('settings.apply') }}</a-button>
          </div>
        </div>

        <!-- 颜色主题 -->
        <div class="settings-section">
          <div class="settings-label">{{ t('settings.colorTheme') }}</div>
          <div class="theme-options">
            <div
              v-for="item in themeOptions"
              :key="item.value"
              class="theme-card"
              :class="{ 'theme-card--active': editConfig.colorTheme === item.value }"
              @click="editConfig.colorTheme = item.value"
            >
              <div class="theme-card__preview" :class="'theme-preview--' + item.value">
                <div class="theme-preview__bar"></div>
                <div class="theme-preview__body">
                  <div class="theme-preview__line theme-preview__line--long"></div>
                  <div class="theme-preview__line theme-preview__line--short"></div>
                  <div class="theme-preview__line theme-preview__line--mid"></div>
                </div>
              </div>
              <span class="theme-card__name">{{ item.label }}</span>
            </div>
          </div>
        </div>

        <!-- 阅读视图字体 -->
        <div class="settings-section">
          <div class="settings-label">{{ t('settings.readFontSize') }}</div>
          <div class="font-size-row">
            <a-slider
              :min="1" :max="5" :step="1"
              :value="editConfig.readFontSize"
              :marks="fontSizeMarks"
              :tip-formatter="null"
              @change="v => editConfig.readFontSize = v"
            />
          </div>
        </div>

        <!-- 编辑视图字体 -->
        <div class="settings-section">
          <div class="settings-label">{{ t('settings.editFontSize') }}</div>
          <div class="font-size-row">
            <a-slider
              :min="1" :max="5" :step="1"
              :value="editConfig.editFontSize"
              :marks="fontSizeMarks"
              :tip-formatter="null"
              @change="v => editConfig.editFontSize = v"
            />
          </div>
        </div>

        <!-- 插入设置 -->
        <div class="settings-section">
          <div class="settings-label">{{ t('settings.insertTypeSettings') }}</div>
          <div v-for="(ins, i) in editConfig.insertTypes" :key="i" class="type-row">
            <a-input v-model:value="ins.name" size="small" style="width:80px" :placeholder="t('settings.name')" />
            <input type="color" v-model="ins.color" class="native-color-input" :title="t('settings.iconColor')" />
            <input type="color" v-model="ins.textColor" class="native-color-input" :title="t('settings.textColor')" />
            <a-popover trigger="click" placement="bottom">
              <template #content>
                <div style="width:120px"><a-slider v-model:value="ins.fontSize" :min="10" :max="24" :step="1" /></div>
              </template>
              <a-button size="small" style="font-size:11px;">Aa {{ ins.fontSize || 14 }}px</a-button>
            </a-popover>
            <a-button v-if="i > 0" size="small" type="text" danger @click="editConfig.insertTypes.splice(i,1)">✕</a-button>
            <span v-else style="width:22px;text-align:center;color:var(--text-disabled);font-size:11px;">{{ t('settings.default') }}</span>
          </div>
          <a-button size="small" @click="addInsertType" style="margin-top:4px">{{ t('settings.addNewType') }}</a-button>
        </div>

        <!-- 标记设置 -->
        <div class="settings-section">
          <div class="settings-label">{{ t('settings.markTypeSettings') }}</div>
          <div v-for="(mk, i) in editConfig.markTypes" :key="i" class="type-row">
            <a-input v-model:value="mk.name" size="small" style="width:80px" :placeholder="t('settings.name')" />
            <input type="color" v-model="mk.bgColor" class="native-color-input" :title="t('settings.highlightBgColor')" />
            <input type="color" v-model="mk.textColor" class="native-color-input" :title="t('settings.textColor')" />
            <a-button v-if="i > 0" size="small" type="text" danger @click="editConfig.markTypes.splice(i,1)">✕</a-button>
            <span v-else style="width:22px;text-align:center;color:var(--text-disabled);font-size:11px;">{{ t('settings.default') }}</span>
          </div>
          <a-button size="small" @click="addMarkType" style="margin-top:4px">{{ t('settings.addNewType') }}</a-button>
        </div>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, reactive, computed, watch } from "vue";
import { SettingOutlined } from "@ant-design/icons-vue";
import { message } from "ant-design-vue";
import { Store } from "@tauri-apps/plugin-store";
import { useI18n } from "../locales";
import {
  novelConfigs, activeConfigName, novelConfig,
  switchConfig, applyCurrentConfig, makeDefaultConfig,
} from "../stores/novelStore";
import {
  settings, setColorTheme, setReadFontSize, setEditFontSize,
  FONT_SIZE_LEVELS,
} from "../stores/settingStore";

const { t } = useI18n();
const visible = ref(false);

const editConfig = reactive({
  insertTypes: [],
  markTypes: [],
  colorTheme: "light",
  readFontSize: 3,
  editFontSize: 3,
});

const currentConfigName = ref(activeConfigName.value);
const configRenameInput = ref("");

const configNames = computed(() => Object.keys(novelConfigs.value));

function syncEditFromConfig() {
  const src = novelConfigs.value[currentConfigName.value] || makeDefaultConfig();
  const defaults = makeDefaultConfig();
  if (!src) return;
  editConfig.insertTypes = ((src.insertTypes && src.insertTypes.length > 0) ? src.insertTypes : defaults.insertTypes)
    .map(t => ({ ...t, fontSize: t.fontSize || 14 }));
  editConfig.markTypes = ((src.markTypes && src.markTypes.length > 0) ? src.markTypes : defaults.markTypes)
    .map(m => ({ ...m }));
  editConfig.colorTheme = src.colorTheme || defaults.colorTheme;
  editConfig.readFontSize = src.readFontSize || defaults.readFontSize;
  editConfig.editFontSize = src.editFontSize || defaults.editFontSize;
}

watch(currentConfigName, () => syncEditFromConfig());

// 全局持久化：保存到 setting.json
async function saveGlobalConfig() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) return;
  try {
    const store = await Store.load("setting.json");
    const saved = await store.get("settings");
    const current = (saved && typeof saved === "object") ? { ...saved } : {};
    current.novelConfigs = { ...novelConfigs.value };
    current.activeConfigName = activeConfigName.value;
    await store.set("settings", current);
    await store.save();
  } catch (e) { console.error("保存设置失败:", e); }
}

// 从全局存储加载配置
async function loadGlobalConfig() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) return;
  try {
    const store = await Store.load("setting.json");
    const saved = await store.get("settings");
    if (saved?.novelConfigs && typeof saved.novelConfigs === "object") {
      novelConfigs.value = saved.novelConfigs;
      activeConfigName.value = saved.activeConfigName || Object.keys(saved.novelConfigs)[0] || t('settings.defaultConfig');
    }
  } catch { /* 忽略 */ }
}

function openSettings() {
  loadGlobalConfig().then(() => {
    currentConfigName.value = activeConfigName.value;
    configRenameInput.value = "";
    syncEditFromConfig();
    visible.value = true;
  });
}

function onConfigSwitch(name) { currentConfigName.value = name; }

function onAddConfig() {
  const base = configRenameInput.value.trim() || t('settings.newConfig');
  let name = base;
  let n = 1;
  while (novelConfigs.value[name]) { n++; name = `${base}(${n})`; }
  novelConfigs.value[name] = makeDefaultConfig();
  configRenameInput.value = "";
  currentConfigName.value = name;
  syncEditFromConfig();
  saveGlobalConfig();
}

function onSaveConfig() {
  const targetName = configRenameInput.value.trim() || currentConfigName.value;
  if (targetName !== currentConfigName.value && novelConfigs.value[targetName]) {
    message.warning(t('settings.configNameExists')); return;
  }
  novelConfigs.value[targetName] = {
    insertTypes: editConfig.insertTypes.map(t => ({ ...t })),
    markTypes: editConfig.markTypes.map(m => ({ ...m })),
    colorTheme: editConfig.colorTheme,
    readFontSize: editConfig.readFontSize,
    editFontSize: editConfig.editFontSize,
    exportDefaults: novelConfig.value.exportDefaults || {},
    graphStyle: novelConfig.value.graphStyle || {},
  };
  if (targetName !== currentConfigName.value) {
    delete novelConfigs.value[currentConfigName.value];
    currentConfigName.value = targetName;
    activeConfigName.value = targetName;
  }
  configRenameInput.value = "";
  saveGlobalConfig();
  message.success(t('content.saved'));
}

function onApplyConfig() {
  onSaveConfig();
  switchConfig(currentConfigName.value);
  message.success(t('settings.applied'));
}

function addInsertType() {
  editConfig.insertTypes.push({
    id: "custom_" + Date.now(),
    name: t('settings.newType'),
    color: "#1890ff",
    textColor: "#ffffff",
    fontSize: 14,
    enabled: true,
  });
}

function addMarkType() {
  editConfig.markTypes.push({
    id: "custom_" + Date.now(),
    name: t('settings.newType'),
    bgColor: "#ffd666",
    textColor: "#262626",
    enabled: true,
  });
}

// 主题选项
const themeOptions = computed(() => [
  { value: "light", label: t("settings.light") },
  { value: "dark", label: t("settings.dark") },
  { value: "eyeCare", label: t("settings.eyeCare") },
  { value: "eyeCareGreen", label: t("settings.eyeCareGreen") },
]);

const fontSizeMarks = computed(() => ({
  1: t("settings.small"),
  2: t("settings.smaller"),
  3: t("settings.medium"),
  4: t("settings.larger"),
  5: t("settings.large"),
}));
</script>

<style scoped>
.sider-button {
  width: 36px; height: 36px;
  display: flex; align-items: center; justify-content: center;
  padding: 0; margin: 0 10px 5px 10px;
}

.settings-body { padding: 4px 0; }
.settings-body {
  max-height: 70vh;
  overflow-y: auto;
  padding-right: 6px;
}
.settings-section { margin-bottom: 20px; }
.settings-section:last-child { margin-bottom: 0; }
.settings-label {
  font-size: 14px; font-weight: 500;
  color: var(--text-primary, #262626);
  margin-bottom: 10px;
}

.config-selector { display: flex; gap: 8px; align-items: center; }

/* 主题卡片 */
.theme-options { display: flex; gap: 12px; }
.theme-card {
  flex: 1; display: flex; flex-direction: column; align-items: center;
  gap: 8px; padding: 8px; border-radius: 8px; cursor: pointer;
  border: 2px solid transparent; transition: border-color 0.2s, background 0.2s;
}
.theme-card:hover { background: var(--bg-secondary, #fafafa); }
.theme-card--active { border-color: #1890ff; }
.theme-card__name { font-size: 12px; color: var(--text-secondary, #595959); }
.theme-card__preview {
  width: 100%; height: 56px; border-radius: 6px; overflow: hidden;
  display: flex; flex-direction: column; border: 1px solid var(--border-color, #e8e8e8);
}

/* 主题预览色块 */
.theme-preview--light { background: #fff; }
.theme-preview--light .theme-preview__bar { height: 14px; background: #f5f5f5; border-bottom: 1px solid #e8e8e8; }
.theme-preview--light .theme-preview__body { flex: 1; padding: 8px 10px; display: flex; flex-direction: column; gap: 5px; }
.theme-preview--light .theme-preview__line { height: 4px; background: #e8e8e8; border-radius: 2px; }
.theme-preview--light .theme-preview__line--long { width: 90%; }
.theme-preview--light .theme-preview__line--short { width: 55%; }
.theme-preview--light .theme-preview__line--mid { width: 72%; }

.theme-preview--dark { background: #1e1e1e; }
.theme-preview--dark .theme-preview__bar { height: 14px; background: #2d2d2d; border-bottom: 1px solid #3c3c3c; }
.theme-preview--dark .theme-preview__body { flex: 1; padding: 8px 10px; display: flex; flex-direction: column; gap: 5px; }
.theme-preview--dark .theme-preview__line { height: 4px; background: #3c3c3c; border-radius: 2px; }
.theme-preview--dark .theme-preview__line--long { width: 90%; }
.theme-preview--dark .theme-preview__line--short { width: 55%; }
.theme-preview--dark .theme-preview__line--mid { width: 72%; }

.theme-preview--eyeCare { background: #fdf6e3; }
.theme-preview--eyeCare .theme-preview__bar { height: 14px; background: #efe2c0; border-bottom: 1px solid #d5c4a1; }
.theme-preview--eyeCare .theme-preview__body { flex: 1; padding: 8px 10px; display: flex; flex-direction: column; gap: 5px; }
.theme-preview--eyeCare .theme-preview__line { height: 4px; background: #d5c4a1; border-radius: 2px; }
.theme-preview--eyeCare .theme-preview__line--long { width: 90%; }
.theme-preview--eyeCare .theme-preview__line--short { width: 55%; }
.theme-preview--eyeCare .theme-preview__line--mid { width: 72%; }

.theme-preview--eyeCareGreen { background: #e8f5e9; }
.theme-preview--eyeCareGreen .theme-preview__bar { height: 14px; background: #c8e6c9; border-bottom: 1px solid #a5d6a7; }
.theme-preview--eyeCareGreen .theme-preview__body { flex: 1; padding: 8px 10px; display: flex; flex-direction: column; gap: 5px; }
.theme-preview--eyeCareGreen .theme-preview__line { height: 4px; background: #a5d6a7; border-radius: 2px; }
.theme-preview--eyeCareGreen .theme-preview__line--long { width: 90%; }
.theme-preview--eyeCareGreen .theme-preview__line--short { width: 55%; }
.theme-preview--eyeCareGreen .theme-preview__line--mid { width: 72%; }

/* 字体 */
.font-size-row { padding: 0 4px; }
.font-size-row :deep(.ant-slider-mark-text) { font-size: 11px; white-space: nowrap; color: var(--text-tertiary, #8c8c8c); }

/* 类型设置行 */
.type-row {
  display: flex; align-items: center; gap: 6px;
  padding: 4px 0; border-bottom: 1px solid var(--border-secondary, #f0f0f0);
}
.type-row:last-child { border-bottom: none; }

.native-color-input {
  width: 26px; height: 26px; border: 1px solid var(--border-color, #d9d9d9);
  border-radius: 4px; cursor: pointer; padding: 1px;
}
</style>
