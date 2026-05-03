import { ref } from "vue";
import { Store } from "@tauri-apps/plugin-store";

// 字体档位 → px 映射
export const FONT_SIZE_MAP = { 1: 12, 2: 14, 3: 16, 4: 18, 5: 20 };

// 档位标签 key
export const FONT_SIZE_LEVELS = ["small", "smaller", "medium", "larger", "large"];

// 默认设置
const DEFAULT_SETTINGS = {
  colorTheme: "light",
  readFontSize: 3,
  editFontSize: 3,
};

// 各主题的 CSS 变量
const THEME_VARS = {
  light: {
    "--bg-base": "#ffffff",
    "--bg-secondary": "#fafafa",
    "--bg-tertiary": "#f5f5f5",
    "--bg-elevated": "#ffffff",
    "--text-primary": "#262626",
    "--text-secondary": "#595959",
    "--text-tertiary": "#8c8c8c",
    "--text-disabled": "#bfbfbf",
    "--border-color": "#e8e8e8",
    "--border-secondary": "#f0f0f0",
  },
  dark: {
    "--bg-base": "#1e1e1e",
    "--bg-secondary": "#252525",
    "--bg-tertiary": "#2d2d2d",
    "--bg-elevated": "#333333",
    "--text-primary": "#d4d4d4",
    "--text-secondary": "#999999",
    "--text-tertiary": "#6c6c6c",
    "--text-disabled": "#555555",
    "--border-color": "#3c3c3c",
    "--border-secondary": "#333333",
  },
  eyeCare: {
    "--bg-base": "#fdf6e3",
    "--bg-secondary": "#f5ecd0",
    "--bg-tertiary": "#efe2c0",
    "--bg-elevated": "#fdf6e3",
    "--text-primary": "#5c4b2c",
    "--text-secondary": "#8b7355",
    "--text-tertiary": "#a09080",
    "--text-disabled": "#c5b89e",
    "--border-color": "#d5c4a1",
    "--border-secondary": "#e8dcc8",
  },
  eyeCareGreen: {
    "--bg-base": "#e8f5e9",
    "--bg-secondary": "#dcedc8",
    "--bg-tertiary": "#c8e6c9",
    "--bg-elevated": "#e8f5e9",
    "--text-primary": "#2e4a2e",
    "--text-secondary": "#4a6b4a",
    "--text-tertiary": "#6b8a6b",
    "--text-disabled": "#9dba9d",
    "--border-color": "#a5d6a7",
    "--border-secondary": "#c8e6c9",
  },
};

// 响应式设置状态
export const settings = ref({ ...DEFAULT_SETTINGS });

// 是否已完成从磁盘加载（用于控制保存时机）
let initialized = false;

// 将主题 CSS 变量写到 :root
function applyTheme(theme) {
  const vars = THEME_VARS[theme] || THEME_VARS.light;
  const root = document.documentElement;
  Object.entries(vars).forEach(([key, value]) => {
    root.style.setProperty(key, value);
  });
}

// 将字体档位写成 CSS 变量
function applyFontSize(type, level) {
  const size = FONT_SIZE_MAP[level] || FONT_SIZE_MAP[3];
  const prop = type === "read" ? "--read-font-size" : "--edit-font-size";
  document.documentElement.style.setProperty(prop, `${size}px`);
}

// 启动时加载设置（先应用默认值避免闪烁，再异步覆盖）
export async function loadSettings() {
  // 先用默认值设置 CSS 变量
  applyTheme(DEFAULT_SETTINGS.colorTheme);
  applyFontSize("read", DEFAULT_SETTINGS.readFontSize);
  applyFontSize("edit", DEFAULT_SETTINGS.editFontSize);

  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    initialized = true;
    return;
  }

  try {
    const store = await Store.load("setting.json");
    const saved = await store.get("settings");
    if (saved && typeof saved === "object") {
      settings.value = { ...DEFAULT_SETTINGS, ...saved };
      // 用加载到的值重新应用
      applyTheme(settings.value.colorTheme);
      applyFontSize("read", settings.value.readFontSize);
      applyFontSize("edit", settings.value.editFontSize);
    }
  } catch (error) {
    console.error("加载设置失败:", error);
  }
  initialized = true;
}

// 持久化当前设置
async function saveSettings() {
  if (!initialized) return;
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) return;

  try {
    const store = await Store.load("setting.json");
    await store.set("settings", { ...settings.value });
  } catch (error) {
    console.error("保存设置失败:", error);
  }
}

// 切换颜色主题
export async function setColorTheme(theme) {
  if (!THEME_VARS[theme]) return;
  settings.value.colorTheme = theme;
  applyTheme(theme);
  await saveSettings();
}

// 设置阅读视图字体档位
export async function setReadFontSize(level) {
  if (level < 1 || level > 5) return;
  settings.value.readFontSize = level;
  applyFontSize("read", level);
  await saveSettings();
}

// 设置编辑视图字体档位
export async function setEditFontSize(level) {
  if (level < 1 || level > 5) return;
  settings.value.editFontSize = level;
  applyFontSize("edit", level);
  await saveSettings();
}
