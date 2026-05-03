import { invoke } from "@tauri-apps/api/core";
import { getWarehouseRootPath } from "./fileActions";

export async function fetchFavorites() {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return [];
  }

  try {
    const warehousePath = await getWarehouseRootPath();

    if (!warehousePath) {
      return [];
    }

    const favorites = await invoke("get_favorites_json", { warehousePath });
    return Array.isArray(favorites) ? favorites : [];
  } catch (error) {
    if (String(error?.message || "").includes("璇烽€夋嫨浠撳簱")) {
      return [];
    }

    throw error;
  }
}

export async function createFavorite({ filePath, itemType = "file" }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return [];
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("add_favorite", {
    warehousePath,
    filePath,
    itemType,
  });
}

export async function removeFavorite({ filePath }) {
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return [];
  }

  const warehousePath = await getWarehouseRootPath();
  return invoke("remove_favorite", {
    warehousePath,
    filePath,
  });
}
