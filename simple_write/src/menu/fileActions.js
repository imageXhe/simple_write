import { invoke } from "@tauri-apps/api/core";
import { Store } from "@tauri-apps/plugin-store";
import { useI18n } from "../locales";

const { t } = useI18n();

function getDefaultEntryName(isFolder) {
  if (isFolder) {
    return t("file.newFolderName");
  }

  return t("file.newFileName");
}

function normalizeKey(key) {
  return Array.isArray(key) ? key : [];
}

/**
 * 获取仓库记录的异步函数
 * @returns {Promise<Object>} 返回包含仓库路径和名称的对象
 * @throws {Error} 当仓库路径或名称不存在时抛出错误
 */
async function getWarehouseRecord() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return null;
  }

  // 从JSON文件加载商店数据
  const store = await Store.load("store.json");
  // 从商店数据中获取当前仓库信息
  const warehouse = await store.get("warehouse_now");

  // 检查仓库对象是否存在，并且包含必要的path和name属性
  if (!warehouse?.path || !warehouse?.name) {
    // 如果缺少必要信息，抛出错误
    throw new Error("请先选择仓库");
  }

  // 返回仓库信息
  return warehouse;
}

/**
 * 获取仓库的根路径
 * @returns {Promise<string>} 返回仓库的根路径字符串
 */
export async function getWarehouseRootPath() {
  const warehouse = await getWarehouseRecord(); // 获取仓库记录

  if (!warehouse) {
    return "";
  }

  return `${warehouse.path}/${warehouse.name}`; // 拼接路径和仓库名称作为根路径返回
}

/**
 * 获取文件树结构的异步函数
 * 该函数通过调用系统接口获取仓库根路径和文件数据
 * @returns {Promise<Object>} 返回一个包含仓库路径和文件数据的对象
 */
export async function fetchFileTree() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return {
      warehousePath: "",
      fileData: [],
    };
  }

  // 获取仓库根路径
  const warehousePath = await getWarehouseRootPath();
  // 调用系统接口获取文件JSON数据
  const fileData = await invoke("get_file_json", { warehousePath });

  // 返回包含仓库路径和文件数据的对象
  return {
    warehousePath,
    fileData,
  };
}

export async function createFileEntry({ isFolder, parentKey = [] }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return null;
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("create_file", {
    request: {
      isFolder,
      name: getDefaultEntryName(isFolder),
      keyParent: normalizeKey(parentKey),
      warehousePath,
    },
  });
}

export async function duplicateFileEntry({ key = [] }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return null;
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("duplicate_file_entry", {
    request: {
      key: normalizeKey(key),
      copySuffix: t("file.copySuffix"),
      warehousePath,
    },
  });
}

export async function renameFileEntry({ key = [], newName }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return null;
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("rename_file_entry", {
    request: {
      key: normalizeKey(key),
      newName,
      warehousePath,
    },
  });
}

export async function moveFileEntry({ key = [], newParentPath }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return null;
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("move_file_entry", {
    request: {
      key: normalizeKey(key),
      newParentPath,
      warehousePath,
    },
  });
}

export async function deleteFileEntry({ key = [] }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return null;
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("delete_file_entry", {
    request: {
      key: normalizeKey(key),
      warehousePath,
    },
  });
}

export async function saveFileContent(filePath, content) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return null;
  }

  return invoke("save_file_content", {
    filePath,
    content,
  });
}

// 快速粘贴：读取剪贴板内容并追加到仓库根目录文件
export async function quickPaste(fileName) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return null;
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("quick_paste", {
    warehousePath,
    fileName,
  });
}
