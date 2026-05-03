<template>
  <a-tooltip :title="t('common.help')" placement="right" :arrow="false">
    <a-button type="text" class="sider-button" @click="openHelp">
      <QuestionCircleOutlined />
    </a-button>
  </a-tooltip>

  <div>
    <a-modal
      :title="t('help.title')"
      v-model:open="visible"
      :footer="null"
      :maskClosable="false"
      width="560px"
      class="help-modal"
    >
      <!-- 项目简介 -->
      <div class="help-section">
        <div class="help-section-title">
          <InfoCircleOutlined />
          <span>{{ t('common.help') }}</span>
        </div>
        <p class="help-intro">{{ t('help.intro') }}</p>
      </div>

      <!-- 功能特性 -->
      <div class="help-section">
        <div class="help-section-title">
          <StarOutlined />
          <span>{{ t('help.features') }}</span>
        </div>
        <ul class="help-list">
          <li>{{ t('help.feature1') }}</li>
          <li>{{ t('help.feature2') }}</li>
          <li>{{ t('help.feature3') }}</li>
          <li>{{ t('help.feature4') }}</li>
          <li>{{ t('help.feature5') }}</li>
          <li>{{ t('help.feature6') }}</li>
          <li>{{ t('help.feature7') }}</li>
        </ul>
      </div>

      <!-- 操作区 -->
      <div class="help-section">
        <div class="help-section-title">
          <ToolOutlined />
          <span>{{ t('file.manage') }}</span>
        </div>
        <a-button type="primary" block @click="handleGenerateReadme" :loading="generating">
          <FileTextOutlined />
          {{ t('help.generateReadme') }}
        </a-button>
        <p class="help-desc">{{ t('help.generateReadmeDesc') }}</p>
      </div>

      <!-- GitHub 链接 -->
      <div class="help-section help-github">
        <a-divider />
        <a href="https://github.com/your-username/simple-write" target="_blank" class="github-link">
          <GithubOutlined class="github-icon" />
          <span>{{ t('help.githubLink') }}</span>
          <ExportOutlined class="github-external" />
        </a>
        <p class="help-desc">{{ t('help.githubDesc') }}</p>
      </div>
    </a-modal>
  </div>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";
import { message } from "ant-design-vue";
import {
  QuestionCircleOutlined,
  InfoCircleOutlined,
  StarOutlined,
  ToolOutlined,
  FileTextOutlined,
  GithubOutlined,
  ExportOutlined,
} from "@ant-design/icons-vue";
import { useI18n } from "../locales";

const { t } = useI18n();

const visible = ref(false);
const generating = ref(false);
const warehousePath = ref("");

const openHelp = async () => {
  // 获取当前仓库路径
  try {
    if (window.__TAURI_INTERNALS__) {
      const store = await Store.load("store.json");
      const v = await store.get("warehouse_now");
      if (v && v.path) {
        warehousePath.value = v.path + "/" + v.name;
      }
    }
  } catch (e) {
    console.error("获取仓库路径失败:", e);
  }
  visible.value = true;
};

const handleGenerateReadme = async () => {
  if (!warehousePath.value) {
    message.warning(t("file.pleaseSelectWarehouse"));
    return;
  }

  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    message.info("当前环境不支持此操作");
    return;
  }

  generating.value = true;
  try {
    await invoke("generate_readme", {
      warehousePath: warehousePath.value,
    });
    message.success(t("help.readmeGenerated"));
  } catch (error) {
    message.error(t("help.readmeExists"));
  }
  generating.value = false;
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

.help-modal .help-section {
  margin-bottom: 18px;
}

.help-section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary, #262626);
  margin-bottom: 8px;
}

.help-intro {
  color: var(--text-secondary, #595959);
  font-size: 13px;
  line-height: 1.7;
  margin: 0 0 0 4px;
}

.help-list {
  margin: 0;
  padding-left: 20px;
  list-style: disc;
}

.help-list li {
  color: var(--text-secondary, #595959);
  font-size: 13px;
  line-height: 2;
}

.help-desc {
  color: var(--text-tertiary, #8c8c8c);
  font-size: 12px;
  margin: 6px 0 0 0;
}

.help-github {
  text-align: center;
}

.github-link {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-primary, #262626);
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  padding: 6px 16px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 6px;
  transition: all 0.2s;
}

.github-link:hover {
  color: #1677ff;
  border-color: #1677ff;
}

.github-icon {
  font-size: 18px;
}

.github-external {
  font-size: 12px;
  color: var(--text-tertiary, #8c8c8c);
}
</style>
