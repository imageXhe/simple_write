import { ref } from 'vue';

// 共享文件树数据，供 FileList 和 EditView 跨组件访问
export const fileData = ref([]);
export const warehousePath = ref('');

// 将文件树展平为 { name, path, dir, isFolder } 数组（包含文件和文件夹）
export const flattenTree = (nodes, basePath) => {
    const result = [];
    const walk = (list, segs) => {
        for (const node of list) {
            if (!node || !node.info) continue;
            const fullSegs = [...segs, node.info.name];
            const dir = segs.length > 0 ? segs.join('/') + '/' : '';
            result.push({
                name: node.info.name,
                path: [basePath, ...fullSegs].filter(Boolean).join('/').replace(/\\/g, '/'),
                dir,
                isFolder: node.is_folder || false,
            });
            if (node.children?.length) {
                walk(node.children, fullSegs);
            }
        }
    };
    walk(nodes, []);
    return result;
};

// 根据文件名查找条目（含文件夹）
export const findEntryByName = (name) => {
    const all = flattenTree(fileData.value, warehousePath.value);
    const lower = name.toLowerCase();

    // 精确匹配
    const exact = all.find((f) => f.name === name);
    if (exact) return exact;

    // 补 .md 扩展名
    const withMd = all.find((f) => f.name === name + '.md');
    if (withMd) return withMd;

    // 模糊匹配（忽略大小写，忽略扩展名）
    const fuzzy = all.find((f) => {
        const fl = f.name.toLowerCase();
        if (fl === lower) return true;
        const dot = fl.lastIndexOf('.');
        return dot > 0 && fl.slice(0, dot) === lower;
    });
    return fuzzy || null;
};

// 根据文件名查找文件路径（向后兼容）
export const findFileByName = (name) => {
    const entry = findEntryByName(name);
    if (!entry) return null;
    return { path: entry.path, name: entry.name, isFolder: entry.isFolder };
};

// 获取文件夹的直接子节点（使用展平列表匹配子目录）
export const getFolderChildren = (folderPath) => {
    const norm = (p) => (p || '').replace(/\\/g, '/').replace(/\/+$/, '');
    const all = flattenTree(fileData.value, warehousePath.value);
    const target = norm(folderPath);
    const children = [];
    for (const entry of all) {
        const entryDir = norm(entry.path.split('/').slice(0, -1).join('/'));
        if (entryDir === target) {
            children.push({
                name: entry.name,
                isFolder: entry.isFolder,
            });
        }
    }
    return children;
};
