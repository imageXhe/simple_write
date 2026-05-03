import { invoke } from "@tauri-apps/api/core";
import { getWarehouseRootPath } from "./fileActions";

export async function fetchBookmarks() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return [];
  }

  try {
    const warehousePath = await getWarehouseRootPath();

    if (!warehousePath) {
      return [];
    }

    const bookmarks = await invoke("get_bookmarks_json", { warehousePath });
    return Array.isArray(bookmarks) ? bookmarks : [];
  } catch (error) {
    if (String(error?.message || "").includes("请选择仓库")) {
      return [];
    }

    throw error;
  }
}

export async function createBookmark({ filePath, name }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return [];
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("add_bookmark", {
    warehousePath,
    filePath,
    name,
  });
}

export async function removeBookmark({ filePath }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return [];
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("remove_bookmark", {
    warehousePath,
    filePath,
  });
}
