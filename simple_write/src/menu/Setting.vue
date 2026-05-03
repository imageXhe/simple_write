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
      width="420px"
    >
      <div class="settings-body">
        <!-- 颜色主题 -->
        <div class="settings-section">
          <div class="settings-label">{{ t('settings.colorTheme') }}</div>
          <div class="theme-options">
            <div
              v-for="item in themeOptions"
              :key="item.value"
              class="theme-card"
              :class="{ 'theme-card--active': settings.colorTheme === item.value }"
              @click="handleThemeChange(item.value)"
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
              :min="1"
              :max="5"
              :step="1"
              :value="settings.readFontSize"
              :marks="fontSizeMarks"
              :tip-formatter="null"
              @change="handleReadFontChange"
            />
          </div>
        </div>

        <!-- 编辑视图字体 -->
        <div class="settings-section">
          <div class="settings-label">{{ t('settings.editFontSize') }}</div>
          <div class="font-size-row">
            <a-slider
              :min="1"
              :max="5"
              :step="1"
              :value="settings.editFontSize"
              :marks="fontSizeMarks"
              :tip-formatter="null"
              @change="handleEditFontChange"
            />
          </div>
        </div>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, computed } from "vue";
import { SettingOutlined } from "@ant-design/icons-vue";
import { useI18n } from "../locales";
import {
  settings,
  setColorTheme,
  setReadFontSize,
  setEditFontSize,
  FONT_SIZE_LEVELS,
} from "../stores/settingStore";

const { t } = useI18n();

const visible = ref(false);

const openSettings = () => {
  visible.value = true;
};

// 主题选项
const themeOptions = computed(() => [
  { value: "light", label: t("settings.light") },
  { value: "dark", label: t("settings.dark") },
  { value: "eyeCare", label: t("settings.eyeCare") },
  { value: "eyeCareGreen", label: t("settings.eyeCareGreen") },
]);

// 字体档位滑块刻度
const fontSizeMarks = computed(() => ({
  1: t("settings.small"),
  2: t("settings.smaller"),
  3: t("settings.medium"),
  4: t("settings.larger"),
  5: t("settings.large"),
}));

const handleThemeChange = (theme) => {
  setColorTheme(theme);
};

const handleReadFontChange = (level) => {
  setReadFontSize(level);
};

const handleEditFontChange = (level) => {
  setEditFontSize(level);
};
</script>

<style scoped>
.sider-button {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  margin: 0 10px 5px 10px;
}

.settings-body {
  padding: 4px 0;
}

.settings-section {
  margin-bottom: 24px;
}

.settings-section:last-child {
  margin-bottom: 0;
}

.settings-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #262626);
  margin-bottom: 12px;
}

/* ---- 主题卡片 ---- */
.theme-options {
  display: flex;
  gap: 12px;
}

.theme-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border-radius: 8px;
  cursor: pointer;
  border: 2px solid transparent;
  transition: border-color 0.2s, background 0.2s;
}

.theme-card:hover {
  background: var(--bg-secondary, #fafafa);
}

.theme-card--active {
  border-color: #1890ff;
}

.theme-card__name {
  font-size: 12px;
  color: var(--text-secondary, #595959);
}

/* 主题预览色块 */
.theme-card__preview {
  width: 100%;
  height: 56px;
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color, #e8e8e8);
}

.theme-preview--light {
  background: #fff;
}
.theme-preview--light .theme-preview__bar {
  height: 14px;
  background: #f5f5f5;
  border-bottom: 1px solid #e8e8e8;
}
.theme-preview--light .theme-preview__body {
  flex: 1;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.theme-preview--light .theme-preview__line {
  height: 4px;
  background: #e8e8e8;
  border-radius: 2px;
}
.theme-preview--light .theme-preview__line--long { width: 90%; }
.theme-preview--light .theme-preview__line--short { width: 55%; }
.theme-preview--light .theme-preview__line--mid { width: 72%; }

.theme-preview--dark {
  background: #1e1e1e;
}
.theme-preview--dark .theme-preview__bar {
  height: 14px;
  background: #2d2d2d;
  border-bottom: 1px solid #3c3c3c;
}
.theme-preview--dark .theme-preview__body {
  flex: 1;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.theme-preview--dark .theme-preview__line {
  height: 4px;
  background: #3c3c3c;
  border-radius: 2px;
}
.theme-preview--dark .theme-preview__line--long { width: 90%; }
.theme-preview--dark .theme-preview__line--short { width: 55%; }
.theme-preview--dark .theme-preview__line--mid { width: 72%; }

.theme-preview--eyeCare {
  background: #fdf6e3;
}
.theme-preview--eyeCare .theme-preview__bar {
  height: 14px;
  background: #efe2c0;
  border-bottom: 1px solid #d5c4a1;
}
.theme-preview--eyeCare .theme-preview__body {
  flex: 1;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.theme-preview--eyeCare .theme-preview__line {
  height: 4px;
  background: #d5c4a1;
  border-radius: 2px;
}
.theme-preview--eyeCare .theme-preview__line--long { width: 90%; }
.theme-preview--eyeCare .theme-preview__line--short { width: 55%; }
.theme-preview--eyeCare .theme-preview__line--mid { width: 72%; }

.theme-preview--eyeCareGreen {
  background: #e8f5e9;
}
.theme-preview--eyeCareGreen .theme-preview__bar {
  height: 14px;
  background: #c8e6c9;
  border-bottom: 1px solid #a5d6a7;
}
.theme-preview--eyeCareGreen .theme-preview__body {
  flex: 1;
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 5px;
}
.theme-preview--eyeCareGreen .theme-preview__line {
  height: 4px;
  background: #a5d6a7;
  border-radius: 2px;
}
.theme-preview--eyeCareGreen .theme-preview__line--long { width: 90%; }
.theme-preview--eyeCareGreen .theme-preview__line--short { width: 55%; }
.theme-preview--eyeCareGreen .theme-preview__line--mid { width: 72%; }

/* ---- 字体档位滑块 ---- */
.font-size-row {
  padding: 0 4px;
}

.font-size-row :deep(.ant-slider-mark-text) {
  font-size: 11px;
  white-space: nowrap;
  color: var(--text-tertiary, #8c8c8c);
}
</style>
