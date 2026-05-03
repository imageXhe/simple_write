import { invoke } from "@tauri-apps/api/core";
import { getWarehouseRootPath } from "./fileActions";

export async function scanLinks() {
    const warehousePath = await getWarehouseRootPath();
    return invoke("scan_links", { warehousePath });
}

export async function getOutgoingLinks(filePath) {
    const warehousePath = await getWarehouseRootPath();
    return invoke("get_outgoing_links", { warehousePath, filePath });
}

export async function getIncomingLinks(filePath) {
    const warehousePath = await getWarehouseRootPath();
    return invoke("get_incoming_links", { warehousePath, filePath });
}
