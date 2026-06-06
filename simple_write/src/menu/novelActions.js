import { invoke } from "@tauri-apps/api/core";
import { getWarehouseRootPath } from "./fileActions";

// 读取小说写作配置（不存在时返回默认值）
export async function readNovelConfig() {
    const warehousePath = await getWarehouseRootPath();
    return invoke("read_novel_config", { warehousePath });
}

// 保存小说写作配置
export async function writeNovelConfig(config) {
    const warehousePath = await getWarehouseRootPath();
    return invoke("write_novel_config", { warehousePath, config });
}

// 读取所有 txt 元数据
export async function readTxtMeta() {
    const warehousePath = await getWarehouseRootPath();
    return invoke("read_txt_meta", { warehousePath });
}

// 保存所有 txt 元数据
export async function writeTxtMeta(meta) {
    const warehousePath = await getWarehouseRootPath();
    return invoke("write_txt_meta", { warehousePath, meta });
}

// 批量导出小说合集
export async function exportStoryBundle(request) {
    const warehousePath = await getWarehouseRootPath();
    return invoke("export_story_bundle", { request: { ...request, warehousePath } });
}

// 自定义链接
export async function readCustomLinks() {
    const warehousePath = await getWarehouseRootPath();
    return invoke("read_custom_links", { warehousePath });
}

export async function addCustomLink(link) {
    const warehousePath = await getWarehouseRootPath();
    return invoke("add_custom_link", { warehousePath, link });
}

export async function removeCustomLink(id) {
    const warehousePath = await getWarehouseRootPath();
    return invoke("remove_custom_link", { warehousePath, id });
}
