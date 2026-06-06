import { ref, computed } from "vue";
import { readNovelConfig, writeNovelConfig, readTxtMeta, writeTxtMeta } from "../menu/novelActions";
import { settings, setColorTheme, setReadFontSize, setEditFontSize } from "./settingStore";

// 配置档案集：{ "配置名": { ... } }
export const novelConfigs = ref({});
export const activeConfigName = ref("默认配置");

// txt 元数据：{ [filePath]: { inserts: [], marks: [] } }
export const txtMeta = ref({});

let initialized = false;

// ---- 默认配置内容 ----
function makeDefaultInsertTypes() {
    return [
        { id: "hidden-link", name: "隐藏链接", icon: "link", color: "#1890ff", textColor: "#ffffff", enabled: true },
        { id: "annotation", name: "批注", icon: "comment", color: "#faad14", textColor: "#ffffff", enabled: true },
        { id: "reference", name: "引用提示", icon: "quote", color: "#52c41a", textColor: "#ffffff", enabled: true },
    ];
}

function makeDefaultMarkTypes() {
    return [
        { id: "highlight", name: "重点", bgColor: "#ffd666", textColor: "#262626", enabled: true },
        { id: "foreshadow", name: "伏笔", bgColor: "#b37feb", textColor: "#ffffff", enabled: true },
        { id: "character", name: "人物", bgColor: "#69c0ff", textColor: "#262626", enabled: true },
        { id: "location", name: "地点", bgColor: "#95de64", textColor: "#262626", enabled: true },
    ];
}

export function makeDefaultConfig() {
    return {
        insertTypes: makeDefaultInsertTypes(),
        markTypes: makeDefaultMarkTypes(),
        exportDefaults: {
            includeFileNameAsChapter: true,
            includeFolderNameAsVolume: true,
            blankLineBetweenSections: true,
            outputEncoding: "utf-8",
        },
        graphStyle: {
            direction: "top-to-bottom",
            folderNodeColor: "#69c0ff",
            folderNodeBorder: "#1890ff",
            folderNodeFontSize: 14,
            fileNodeColor: "#ffffff",
            fileNodeBorder: "#d9d9d9",
            fileNodeFontSize: 12,
            edgeColor: "#bfbfbf",
            edgeWidth: 1,
            showArrows: true,
            nodeGap: 40,
            levelGap: 80,
        },
        colorTheme: "light",
        readFontSize: 3,
        editFontSize: 3,
    };
}

function normalizeConfigShape(config = {}) {
    const defaults = makeDefaultConfig();
    const insertTypes = Array.isArray(config.insertTypes) && config.insertTypes.length > 0
        ? config.insertTypes.map((item, index) => ({
            ...defaults.insertTypes[Math.min(index, defaults.insertTypes.length - 1)],
            ...item,
            enabled: item?.enabled !== false,
        }))
        : defaults.insertTypes;
    const markTypes = Array.isArray(config.markTypes) && config.markTypes.length > 0
        ? config.markTypes.map((item, index) => ({
            ...defaults.markTypes[Math.min(index, defaults.markTypes.length - 1)],
            ...item,
            enabled: item?.enabled !== false,
        }))
        : defaults.markTypes;

    return {
        ...defaults,
        ...config,
        insertTypes,
        markTypes,
        exportDefaults: {
            ...defaults.exportDefaults,
            ...(config.exportDefaults || {}),
        },
        graphStyle: {
            ...defaults.graphStyle,
            ...(config.graphStyle || {}),
        },
    };
}

function normalizeConfigMap(configMap = {}) {
    const normalized = {};
    for (const [name, config] of Object.entries(configMap)) {
        normalized[name] = normalizeConfigShape(config);
    }
    if (Object.keys(normalized).length === 0) {
        normalized["默认配置"] = makeDefaultConfig();
    }
    return normalized;
}

// ---- 当前活跃配置（便捷访问） ----
export const novelConfig = computed(() => {
    return novelConfigs.value[activeConfigName.value] || makeDefaultConfig();
});

export const enabledInsertTypes = computed(() => {
    const cfg = novelConfig.value;
    return (cfg.insertTypes || []).filter((t) => t.enabled);
});

export const enabledMarkTypes = computed(() => {
    const cfg = novelConfig.value;
    return (cfg.markTypes || []).filter((t) => t.enabled);
});

// ---- 加载配置档案（优先从全局 setting.json，回退到仓库级配置） ----
export async function loadNovelConfig() {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        novelConfigs.value = { "默认配置": makeDefaultConfig() };
        activeConfigName.value = "默认配置";
        initialized = true;
        return;
    }
    try {
        // 优先从全局 setting.json 加载
        const { Store } = await import("@tauri-apps/plugin-store");
        const store = await Store.load("setting.json");
        const saved = await store.get("settings");
        if (saved?.novelConfigs && typeof saved.novelConfigs === "object" && Object.keys(saved.novelConfigs).length > 0) {
            novelConfigs.value = normalizeConfigMap(saved.novelConfigs);
            activeConfigName.value = saved.activeConfigName || Object.keys(saved.novelConfigs)[0] || "默认配置";
            initialized = true;
            return;
        }
    } catch { /* 回退到仓库级配置 */ }

    try {
        const data = await readNovelConfig();
        if (data && data.configs && typeof data.configs === "object") {
            novelConfigs.value = normalizeConfigMap(data.configs);
            activeConfigName.value = data.activeConfig || Object.keys(data.configs)[0] || "默认配置";
        } else if (data && data.insertTypes) {
            novelConfigs.value = { "默认配置": normalizeConfigShape(data) };
            activeConfigName.value = "默认配置";
        } else {
            novelConfigs.value = { "默认配置": makeDefaultConfig() };
            activeConfigName.value = "默认配置";
        }
    } catch {
        novelConfigs.value = { "默认配置": makeDefaultConfig() };
        activeConfigName.value = "默认配置";
    }
    initialized = true;
}

// ---- 保存配置档案到仓库 ----
export async function saveNovelConfigs() {
    if (!initialized) return;
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) return;
    try {
        await writeNovelConfig({
            configs: novelConfigs.value,
            activeConfig: activeConfigName.value,
        });
    } catch (e) {
        console.error("保存小说配置失败:", e);
    }
}

// ---- 应用当前配置（设置主题、字体等 UI 效果） ----
export function applyCurrentConfig() {
    const cfg = novelConfig.value;
    if (cfg.colorTheme) setColorTheme(cfg.colorTheme);
    if (cfg.readFontSize) setReadFontSize(cfg.readFontSize);
    if (cfg.editFontSize) setEditFontSize(cfg.editFontSize);
}

// ---- 配置档案操作 ----
export function createConfig(name) {
    if (!name || novelConfigs.value[name]) return false;
    novelConfigs.value[name] = makeDefaultConfig();
    saveNovelConfigs();
    return true;
}

export function deleteConfig(name) {
    if (Object.keys(novelConfigs.value).length <= 1) return false;
    delete novelConfigs.value[name];
    if (activeConfigName.value === name) {
        activeConfigName.value = Object.keys(novelConfigs.value)[0];
    }
    saveNovelConfigs();
    return true;
}

export function switchConfig(name) {
    if (!novelConfigs.value[name]) return;
    activeConfigName.value = name;
    saveNovelConfigs();
    applyCurrentConfig();
}

// ---- txt 元数据 ----
export async function loadTxtMeta() {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
        txtMeta.value = {};
        return;
    }
    try {
        const meta = await readTxtMeta();
        txtMeta.value = meta || {};
    } catch {
        txtMeta.value = {};
    }
}

export async function saveTxtMeta() {
    if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) return;
    try {
        await writeTxtMeta(txtMeta.value);
    } catch (e) {
        console.error("保存 txt 元数据失败:", e);
    }
}

export function getFileMeta(filePath) {
    const key = (filePath || "").replace(/\\/g, "/");
    if (!txtMeta.value[key]) {
        txtMeta.value[key] = { inserts: [], marks: [] };
    }
    return txtMeta.value[key];
}

export function addInsert(filePath, insert) {
    const meta = getFileMeta(filePath);
    meta.inserts.push(insert);
    saveTxtMeta();
}

export function removeInsert(filePath, insertId) {
    const meta = getFileMeta(filePath);
    meta.inserts = meta.inserts.filter((i) => i.id !== insertId);
    saveTxtMeta();
}

export function addMark(filePath, mark) {
    const meta = getFileMeta(filePath);
    meta.marks.push(mark);
    saveTxtMeta();
}

export function removeMark(filePath, markId) {
    const meta = getFileMeta(filePath);
    meta.marks = meta.marks.filter((m) => m.id !== markId);
    saveTxtMeta();
}

export function syncFilePath(oldPath, newPath) {
    const oldKey = (oldPath || "").replace(/\\/g, "/");
    const newKey = (newPath || "").replace(/\\/g, "/");
    if (txtMeta.value[oldKey]) {
        txtMeta.value[newKey] = txtMeta.value[oldKey];
        delete txtMeta.value[oldKey];
        saveTxtMeta();
    }
}

export function clearFileMeta(filePath) {
    const key = (filePath || "").replace(/\\/g, "/");
    if (txtMeta.value[key]) {
        delete txtMeta.value[key];
        saveTxtMeta();
    }
}

let _idCounter = 0;
export function generateId() {
    _idCounter++;
    return `nvl_${Date.now()}_${_idCounter}_${Math.random().toString(36).slice(2, 8)}`;
}
