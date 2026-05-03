<template>
  <a-tooltip title="Language" placement="right" :arrow="false">
      <a-button type="text" class="sider-button" @click="changeLanguage">
        <GlobalOutlined />
      </a-button>
  </a-tooltip>

  <div>
    <a-modal 
      :title="t('common.language')" 
      v-model:open="LanguageModalVisible" 
      destroyOnClose
      :maskClosable="false"
      @ok="handleOk"
      :okText="t('common.ok')"
      :cancelText="t('common.cancel')"
    >
      <a-menu>
        <a-menu-item 
          v-for="lang in languages" 
          :key="lang.code"
          @click="handleLanguageChange(lang.code)"
          :class="{ 'ant-menu-item-selected': selectedLang === lang.code }"
        >
          {{ lang.name }}
        </a-menu-item>
      </a-menu>
    </a-modal>
  </div>
</template>

<script setup>
import { ref, onMounted  } from 'vue'
import { useI18n, loadLanguage } from "../locales";
import { GlobalOutlined } from '@ant-design/icons-vue';

const { t, currentLang, setLanguage, languages } = useI18n();
onMounted(async () => {
  await loadLanguage();
});

const selectedLang = ref(currentLang.value);
const handleLanguageChange = (langCode) => {
  selectedLang.value = langCode;
};

const LanguageModalVisible = ref(false);
const changeLanguage = () => {
  LanguageModalVisible.value = true;
  selectedLang.value = currentLang.value;
};

const handleOk = async () => {
  await setLanguage(selectedLang.value);
  LanguageModalVisible.value = false;
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
</style>
