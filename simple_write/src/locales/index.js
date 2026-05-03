import { ref, computed } from "vue";
import { Store } from "@tauri-apps/plugin-store";
import zh from "./zh";
import en from "./en";

// 支持的语言列表
export const languages = [
  { code: "zh", name: "中文" },
  { code: "en", name: "English" },
];

// 语言映射
const messages = {
  zh,
  en,
};

// 当前语言
const currentLang = ref("zh");

// 加载保存的语言设置
export async function loadLanguage() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return;
  }

  try {
    const store = await Store.load("store.json");
    const savedLang = await store.get("language");
    if (savedLang && messages[savedLang]) {
      currentLang.value = savedLang;
    }
  } catch (error) {
    console.error("Failed to load language:", error);
  }
}

// 切换语言
export async function setLanguage(lang) {
  if (!messages[lang]) {
    console.error(`Language ${lang} is not supported`);
    return;
  }

  currentLang.value = lang;

  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return;
  }

  try {
    const store = await Store.load("store.json");
    await store.set("language", lang);
  } catch (error) {
    console.error("Failed to save language:", error);
  }
}

// 获取当前语言
export function getCurrentLanguage() {
  return currentLang.value;
}

// 获取翻译文本
export function t(path) {
  const keys = path.split(".");
  let result = messages[currentLang.value];

  for (const key of keys) {
    if (result && typeof result === "object") {
      result = result[key];
    } else {
      return path; // 如果找不到翻译，返回原始路径
    }
  }

  return result || path;
}

// 创建响应式翻译函数
export function useI18n() {
  return {
    t,
    currentLang: computed(() => currentLang.value),
    setLanguage,
    languages,
  };
}
