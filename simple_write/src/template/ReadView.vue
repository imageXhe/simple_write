<template>
    <a-dropdown :trigger="['contextmenu']" placement="bottomLeft">
        <div
            class="read-view__container"
            @click="handleLinkClick"
            @mousedown="handleImageMouseDown"
            @mousemove="handleImageMouseMove"
            @mouseup="handleImageMouseUp"
            @mouseover="handleContainerMouseOver"
            @mouseout="handleContainerMouseOut"
            @contextmenu="handleContextMenu"
        >
            <div
                v-if="content.length === 0"
                class="read-view__body read-view__body--empty"
            >
                <a-empty :description="t('content.noData')" />
            </div>

            <div
                v-else
                ref="bodyEl"
                class="read-view__body read-view__body--markdown"
                :style="{ '--footnote-heading': footnoteHeading }"
                v-html="renderedMarkdown"
            />

            <!-- 多级预览弹窗（最多 10 层） -->
            <div
                v-for="(p, i) in previews"
                :key="i"
                v-show="p.visible"
                :ref="(el) => { if (el) previewRefs[i] = el; }"
                class="wiki-preview"
                :class="{ 'wiki-preview--nested': i > 0 }"
                :data-level="i"
                :style="p.style"
                @mouseover="onPreviewMouseOver(i)"
                @mouseout="onPreviewMouseOut(i, $event)"
            >
                <div class="wiki-preview__header">{{ p.fileName }}</div>
                <div v-if="p.loading" class="wiki-preview__loading">{{ t("content.loading") }}</div>
                <div
                    v-else-if="p.html"
                    class="wiki-preview__body read-view__body--markdown"
                    :style="{ '--footnote-heading': footnoteHeading }"
                    v-html="p.html"
                    @mouseover="onPreviewBodyMouseOver(i, $event)"
                    @mouseout="onPreviewBodyMouseOut(i, $event)"
                ></div>
                <div v-else class="wiki-preview__loading">{{ t("content.noData") }}</div>
            </div>
        </div>

        <template #overlay>
            <a-menu class="dropdown-menu-bordered" @click="handleMenuClick">
                <a-menu-item key="read-newBookMark" :icon="h(BookOutlined)">{{ t("content.newBookMark") }}</a-menu-item>
                <a-menu-item key="read-makeLink" :icon="h(LinkOutlined)">{{ t('txtEditor.makeLink') }}</a-menu-item>
            </a-menu>
        </template>
    </a-dropdown>
</template>

<script setup>
import { computed, h, ref, watch, inject, nextTick, onMounted, onUnmounted, onActivated, onDeactivated } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { message } from "ant-design-vue";
import { BookOutlined, LinkOutlined } from "@ant-design/icons-vue";
import { open } from "@tauri-apps/plugin-shell";
import { Marked } from "marked";
import markedFootnote from "marked-footnote";
import hljs from "highlight.js";
import { createBookmark } from "../menu/bookmarkActions";
import { saveFileContent } from "../menu/fileActions";
import { useI18n } from "../locales";
import { findFileByName, findEntryByName, getFolderChildren, flattenTree, fileData, warehousePath } from "../stores/fileStore";
import "highlight.js/styles/github.css";

const { t } = useI18n();

const highlightCode = (code, lang) => {
    if (lang && hljs.getLanguage(lang)) {
        try {
            return hljs.highlight(code, { language: lang }).value;
        } catch { /* fall through */ }
    }
    return code;
};

// 自定义渲染器：代码块语法高亮
const codeRenderer = {
    code({ text, lang }) {
        const highlighted = highlightCode(text, lang);
        const langClass = lang ? ` class="hljs language-${lang}"` : '';
        return `<pre><code${langClass}>${highlighted}</code></pre>`;
    },
};

// 任务列表计数器，每次渲染前重置
let taskCounter = 0;

// 自定义渲染器：任务列表（去除圆点、可点击切换）
const listItemRenderer = {
    listitem(token) {
        if (token.task) {
            const idx = taskCounter++;
            const checkedAttr = token.checked ? 'checked' : '';
            return `<li class="task-list-item"><input type="checkbox" data-task-index="${idx}" ${checkedAttr}> ${token.text}</li>`;
        }
        return false; // 非任务列表项使用默认渲染
    },
};

// 为标题生成唯一 ID
let headingIdMap = {};

const slugify = (text) => {
    const slug = text
        .trim()
        .toLowerCase()
        .replace(/\s+/g, '-')
        .replace(/[^\w一-鿿-]/g, '')
        .replace(/-+/g, '-')
        .replace(/^-|-$/g, '');
    if (!slug) return 'heading';
    if (headingIdMap[slug] == null) {
        headingIdMap[slug] = 1;
        return slug;
    }
    const count = headingIdMap[slug] + 1;
    headingIdMap[slug] = count;
    return `${slug}-${count}`;
};

// 自定义渲染器：标题锚点 ID
const headingRenderer = {
    heading({ tokens, depth, text }) {
        const id = slugify(text);
        return `<h${depth} id="${id}">${this.parser.parseInline(tokens)}</h${depth}>`;
    },
};

// 预处理：转义单个 ~（非 ~~），防止被误解析为删除线
const escapeSingleTilde = (text) => {
    return text.replace(/~/g, (_, i, s) => {
        if (s[i - 1] === '~' || s[i + 1] === '~') return '~';
        return '\\~';
    });
};

const marked = new Marked()
    .use(markedFootnote({ description: t("content.footNote") }))
    .use({ renderer: codeRenderer })
    .use({ renderer: listItemRenderer })
    .use({ renderer: headingRenderer });

// 预览专用 marked 实例：启用单换行转 <br>
const previewMarked = new Marked({ breaks: true })
    .use(markedFootnote({ description: t("content.footNote") }))
    .use({ renderer: codeRenderer })
    .use({ renderer: listItemRenderer })
    .use({ renderer: headingRenderer });

const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "gif", "bmp", "svg", "webp", "ico"];

const props = defineProps({
    content: { type: String, required: true },
    filePath: { type: String, default: "" },
    fileName: { type: String, default: "" },
    topLine: { type: Number, default: 0 },
    restoreScrollTop: { type: Number, default: 0 },
});

const openFile = inject("openFile", null);

// ---- 链接悬停预览（最多 10 层） ----
const MAX_PREVIEWS = 5;
const previews = ref([]);       // { visible, html, fileName, loading, style, link, timer, above }
const previewRefs = ref([]);    // DOM 引用
let mainLink = null;            // 主内容区当前悬停的链接
let mainTimer = null;           // 主内容区悬停计时器

const ensureLevel = (n) => {
    while (previews.value.length <= n) {
        previews.value.push({
            visible: false, html: '', fileName: '', loading: false,
            style: { left: '0px', top: '0px' }, link: null, timer: null, above: false,
        });
    }
};

// 传递给 CSS 的脚注标题（用 JSON.stringify 保留引号供 content 使用）
const footnoteHeading = computed(() => JSON.stringify(t("content.footNote")));

const renderedMarkdown = ref("");
let renderVersion = 0;

// 判断是否为本地图片路径
const isLocalImageSrc = (src) => {
    if (!src) return false;
    return !/^(https?:|data:)/i.test(src);
};

// 解析相对路径为绝对路径
const resolveImagePath = (src, mdFilePath) => {
    const normalizedSrc = src.replace(/\\/g, "/");
    // 已经是绝对路径（Windows C:/ 或 Unix /）
    if (/^[a-zA-Z]:\//.test(normalizedSrc) || normalizedSrc.startsWith("/")) {
        return normalizedSrc;
    }
    // 相对路径：基于 md 文件所在目录解析
    const mdDir = mdFilePath.replace(/\\/g, "/").split("/").slice(0, -1).join("/");
    const parts = (mdDir + "/" + normalizedSrc).split("/");
    const resolved = [];
    for (const part of parts) {
        if (part === "..") {
            resolved.pop();
        } else if (part !== "." && part !== "") {
            resolved.push(part);
        }
    }
    // Windows 盘符处理
    if (resolved.length > 0 && /^[a-zA-Z]:$/.test(resolved[0])) {
        return resolved[0] + "/" + resolved.slice(1).join("/");
    }
    return "/" + resolved.join("/");
};

// 将 HTML 中的本地图片 src 替换为 base64 data URL
const processLocalImages = async (html, mdFilePath) => {
    // 匹配 <img> 标签中的 src 属性（支持双引号、单引号、无引号）
    const imgTagRegex = /<img\s[^>]*>/gi;
    const srcRegex = /src\s*=\s*"([^"]*)"/i;
    const srcSingleRegex = /src\s*=\s*'([^']*)'/i;

    const imgTags = [];
    let tagMatch;
    while ((tagMatch = imgTagRegex.exec(html)) !== null) {
        const tag = tagMatch[0];
        let srcMatch = tag.match(srcRegex);
        let quote = '"';
        if (!srcMatch) {
            srcMatch = tag.match(srcSingleRegex);
            quote = "'";
        }
        if (srcMatch && srcMatch[1] && isLocalImageSrc(srcMatch[1])) {
            imgTags.push({
                fullTag: tag,
                src: srcMatch[1],
                quote,
                index: tagMatch.index,
            });
        }
    }

    if (imgTags.length === 0) return html;

    // 并行加载所有本地图片
    const loadPromises = imgTags.map(async (item) => {
        const resolvedPath = resolveImagePath(item.src, mdFilePath);
        try {
            const dataUrl = await invoke("read_file_as_base64", { filePath: resolvedPath });
            return {
                ...item,
                dataUrl: typeof dataUrl === "string" && dataUrl.startsWith("data:") ? dataUrl : "",
            };
        } catch {
            return { ...item, dataUrl: "" };
        }
    });
    const results = await Promise.all(loadPromises);

    // 按位置倒序替换（避免索引偏移）
    let result = html;
    for (const r of results.sort((a, b) => b.index - a.index)) {
        const q = r.quote;
        const newSrc = r.dataUrl || ""; // 加载失败也清空 src，防止浏览器尝试 file://
        const newTag = r.fullTag.replace(
            q === '"' ? srcRegex : srcSingleRegex,
            `src=${q}${newSrc}${q}`
        );
        result = result.slice(0, r.index) + newTag + result.slice(r.index + r.fullTag.length);
    }
    return result;
};

// 将 [[filename]] 转为 wiki 链接（跳过 code/pre 标签内的内容）
const convertWikiLinks = (html) => {
    const parts = html.split(/(<pre[^>]*>[\s\S]*?<\/pre>|<code[^>]*>[\s\S]*?<\/code>)/g);
    for (let i = 0; i < parts.length; i++) {
        if (i % 2 === 0) {
            parts[i] = parts[i].replace(/\[\[([^\]]+)\]\]/g, (_, text) => {
                const escaped = text.replace(/"/g, '&quot;');
                return `<a class="wiki-link" data-filename="${escaped}" href="#">${text}</a>`;
            });
        }
    }
    return parts.join('');
};

// 渲染内容：markdown → HTML → wiki链接 → 处理本地图片
const escapeHtml = (str) => {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
};

const isMarkdown = computed(() => {
    // filePath 才有完整扩展名，fileName 可能已被去扩展名
    const p = (props.filePath || props.fileName || '').toLowerCase();
    return p.endsWith('.md');
});

const renderContent = async () => {
    const version = ++renderVersion;
    if (!isMarkdown.value) {
        // 非 md 文件以纯文本展示
        renderedMarkdown.value = escapeHtml(props.content ?? '');
        if (props.restoreScrollTop > 0) {
            nextTick(() => {
                if (bodyEl.value && bodyEl.value.scrollHeight > props.restoreScrollTop) {
                    bodyEl.value.scrollTop = props.restoreScrollTop;
                }
            });
        }
        return;
    }
    try {
        headingIdMap = {}; // 每次渲染前重置标题 ID 映射
        taskCounter = 0; // 每次渲染前重置任务列表计数器
        let html = marked.parse(escapeSingleTilde(props.content ?? ''));
        // 去掉脚注 li 内部的 <p> 标签
        html = html.replace(
            /(<li id="footnote-[^"]*">)\s*<p>([\s\S]*?)<\/p>(\s*<\/li>)/g,
            "$1$2$3"
        );
        // [[filename]] → wiki 链接
        html = convertWikiLinks(html);
        if (props.filePath) {
            html = await processLocalImages(html, props.filePath);
        }
        // 仅当没有更新的渲染请求时才写入
        if (version === renderVersion) {
            renderedMarkdown.value = html;
            // 统一在 nextTick 中处理折叠+滚动恢复（绘制前完成，避免闪烁）
            const restoreByLine = props.topLine > 0;
            const totalLines = restoreByLine ? (props.content ?? '').split('\n').length : 0;
            const restoreByPixel = !restoreByLine && props.restoreScrollTop > 0;
            nextTick(() => {
                if (isMarkdown.value) {
                    setupCollapsibleHeadings();
                }
                if (!bodyEl.value) return;
                if (restoreByLine && totalLines > 1 && bodyEl.value.scrollHeight > bodyEl.value.clientHeight) {
                    const ratio = props.topLine / (totalLines - 1);
                    bodyEl.value.scrollTop = ratio * (bodyEl.value.scrollHeight - bodyEl.value.clientHeight);
                } else if (restoreByPixel && bodyEl.value.scrollHeight > props.restoreScrollTop) {
                    bodyEl.value.scrollTop = props.restoreScrollTop;
                }
            });
        }
    } catch {
        if (version === renderVersion) {
            renderedMarkdown.value = props.content ?? "";
        }
    }
};

// 监听内容变化，重新渲染
watch(
    () => props.content,
    () => { renderContent(); },
    { immediate: true }
);

// 切换文件/标签时关闭所有预览弹窗
watch(
    () => props.filePath,
    () => {
        closePreviewsFrom(0);
        clearTimeout(mainTimer);
        mainLink = null;
    }
);

const handleCreateBookmark = async () => {
    if (!props.filePath) {
        message.warning(t("message.warning"));
        return;
    }
    try {
        const name = rightClickSnippet.value || props.fileName;
        await createBookmark({
            filePath: props.filePath,
            name,
        });
        rightClickSnippet.value = "";
        window.dispatchEvent(new CustomEvent("simple-write:bookmarks-updated"));
        message.success(t("message.success"));
    } catch (error) {
        message.error(error?.message || t("message.error"));
    }
};

// 翻转任务列表中第 taskIndex 个任务的状态
const toggleTaskInContent = (content, taskIndex) => {
    const lines = content.split('\n');
    let count = 0;
    for (let i = 0; i < lines.length; i++) {
        const match = lines[i].match(/^(\s*)[-*+]\s+\[([ xX])\]\s+(.+)/);
        if (match) {
            if (count === taskIndex) {
                const newState = match[2] === ' ' ? 'x' : ' ';
                lines[i] = `${match[1]}- [${newState}] ${match[3]}`;
                return lines.join('\n');
            }
            count++;
        }
    }
    return content;
};

const handleLinkClick = async (e) => {
    // 处理标题折叠箭头点击
    const toggle = e.target.closest('.heading-toggle');
    if (toggle) {
        e.preventDefault();
        e.stopPropagation();
        const heading = toggle.parentElement;
        const key = `${heading.tagName}:${heading.textContent.replace(/[▼▶]/g, '').trim()}`;
        const set = collapsedHeadings.value;
        if (set.has(key)) {
            set.delete(key);
        } else {
            set.add(key);
        }
        collapsedHeadings.value = new Set(set);
        toggle.textContent = set.has(key) ? '▶' : '▼';
        const wrapper = heading.nextElementSibling;
        if (wrapper?.classList.contains('section-body')) {
            wrapper.style.display = set.has(key) ? 'none' : '';
        }
        return;
    }

    // 处理任务列表 checkbox 点击
    const checkbox = e.target.closest('input[type="checkbox"][data-task-index]');
    if (checkbox) {
        e.preventDefault();
        e.stopPropagation();
        const taskIndex = parseInt(checkbox.dataset.taskIndex, 10);
        const newContent = toggleTaskInContent(props.content ?? '', taskIndex);
        if (newContent !== props.content && props.filePath) {
            // 乐观更新：先翻转 checkbox 状态
            checkbox.checked = !checkbox.checked;
            try {
                await saveFileContent(props.filePath, newContent);
                window.dispatchEvent(new CustomEvent("simple-write:task-toggled", {
                    detail: { path: props.filePath, content: newContent },
                }));
            } catch (error) {
                // 保存失败，恢复状态
                checkbox.checked = !checkbox.checked;
                message.error(error?.message || t("message.error"));
            }
        }
        return;
    }

    // 处理图片点击：激活/取消
    const img = e.target.closest('img');
    if (img && !img.closest('.wiki-preview')) {
        e.preventDefault();
        if (activeImage.value === img) {
            deactivateImage();
        } else {
            deactivateImage();
            img.style.outline = '2px solid #1890ff';
            img.style.cursor = 'ew-resize';
            activeImage.value = img;
        }
        return;
    }
    // 点击非图片区域，取消激活
    if (!img) {
        deactivateImage();
    }

    const target = e.target.closest('a');
    if (!target) return;

    // 关闭所有预览弹窗
    closePreviewsFrom(0);
    clearTimeout(mainTimer);
    mainLink = null;

    // Wiki 内部链接 [[filename]]
    if (target.classList.contains('wiki-link')) {
        e.preventDefault();
        const filename = target.dataset.filename;
        if (!filename) return;
        const found = findFileByName(filename);
        if (!found) {
            message.warning(`未找到文件: ${filename}`);
            return;
        }
        // 文件夹：在文件面板中展开并闪烁
        if (found.isFolder) {
            window.dispatchEvent(new CustomEvent('simple-write:expand-folder', {
                detail: { path: found.path, name: found.name },
            }));
            return;
        }
        if (typeof openFile === 'function') {
            openFile({ filePath: found.path, fileName: found.name });
        }
        return;
    }

    // 外部链接 http/https
    const href = target.getAttribute('href');
    if (href && /^https?:\/\//i.test(href)) {
        e.preventDefault();
        try {
            await open(href);
        } catch {
            // Tauri shell 不可用时降级到 window.open
            window.open(href, '_blank');
        }
        return;
    }

    // 内部锚点链接 #fragment（标题跳转、脚注引用等）
    if (href && href.startsWith('#') && href.length > 1) {
        e.preventDefault();
        let rawId = href.slice(1);
        // URL 解码：中文等非 ASCII 字符在 href 中可能被 percent-encode
        try { rawId = decodeURIComponent(rawId); } catch { /* 保持原值 */ }
        let targetEl = null;
        const body = bodyEl.value;
        if (!body) return;

        // 1) 精确 ID 匹配（脚注 footnote-1、自定义锚点等）
        targetEl = body.querySelector(`[id="${rawId.replace(/"/g, '\\"')}"]`);

        // 2) 回退：遍历标题，参照大纲 scrollToHeading 的文本匹配逻辑
        if (!targetEl) {
            const headings = body.querySelectorAll('h1, h2, h3, h4, h5, h6');
            // 去除折叠箭头后做 slug 比对
            const stripToggle = (t) => (t || '').replace(/[▼▶]/g, '').trim();
            for (const h of headings) {
                if (slugify(stripToggle(h.textContent)) === rawId) {
                    targetEl = h;
                    break;
                }
            }
            // 模糊匹配：slug 以 rawId 开头（处理带 -2/-3 去重后缀的标题）
            if (!targetEl) {
                for (const h of headings) {
                    if (slugify(stripToggle(h.textContent)).startsWith(rawId + '-')) {
                        targetEl = h;
                        break;
                    }
                }
            }
        }

        if (targetEl) {
            targetEl.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
        return;
    }
};

// ---- 预览统一函数 ----

// 根据弹窗实际高度重新定位
const repositionAt = (level) => {
    const p = previews.value[level];
    if (!p) return;
    const el = previewRefs.value[level];
    if (!el || !p.link) return;
    const linkRect = p.link.getBoundingClientRect();
    const h = el.offsetHeight;
    const w = el.offsetWidth;
    const m = 4;
    let left = level === 0 ? linkRect.left : linkRect.right + m;
    let top = p.above ? linkRect.top - h - m : linkRect.bottom + m;
    if (top + h > window.innerHeight - m) { top = linkRect.top - h - m; p.above = true; }
    if (left + w > window.innerWidth - m) left = level === 0 ? left : linkRect.left - w - m;
    if (left < m) left = m;
    p.style = { left: left + 'px', top: top + 'px' };
};

// 加载文件内容到指定预览层
const loadPreviewAt = async (level, link, filename) => {
    ensureLevel(level);
    const p = previews.value[level];
    const m = 4;
    const lr = link.getBoundingClientRect();
    let left = level === 0 ? lr.left : lr.right + m;
    let top = lr.bottom + m;
    p.above = false;
    if (top + 240 > window.innerHeight - m) { top = lr.top - 240 - m; p.above = true; }
    if (level > 0 && left + 360 > window.innerWidth - m) left = lr.left - 360 - m;
    p.style = { left: left + 'px', top: top + 'px' };

    const entry = findEntryByName(filename);
    if (!entry) {
        p.fileName = filename;
        if (level === 0) p.html = '';
        p.loading = false; p.visible = true;
        return;
    }
    p.fileName = entry.name;
    if (level === 0) { p.html = ''; p.loading = true; }
    p.visible = true;

    // 文件夹：递归渲染目录树
    if (entry.isFolder) {
        const norm = (p) => (p || '').replace(/\\/g, '/').replace(/\/+$/, '');
        const all = flattenTree(fileData.value, warehousePath.value);
        const base = norm(entry.path);
        const baseSegs = base.split('/').filter(Boolean);
        // 收集该文件夹下所有条目，按路径深度缩进
        const tree = [];
        for (const f of all) {
            const fSegs = norm(f.path).split('/').filter(Boolean);
            // 检查是否在 base 下
            if (fSegs.length <= baseSegs.length) continue;
            let isChild = true;
            for (let i = 0; i < baseSegs.length; i++) {
                if (fSegs[i] !== baseSegs[i]) { isChild = false; break; }
            }
            if (isChild) {
                const depth = fSegs.length - baseSegs.length - 1;
                tree.push({ ...f, depth: Math.max(0, depth) });
            }
        }
        let html = '<div style="font-size:12px;line-height:1.8;padding:4px 0;">';
        if (tree.length === 0) {
            html += '<span style="color:#bfbfbf;">空文件夹</span>';
        } else {
            for (const t of tree) {
                const icon = t.isFolder ? '📁' : '📄';
                const indent = '&nbsp;&nbsp;'.repeat(t.depth);
                html += `<div style="padding:1px 0;">${indent}${icon} ${escapeHtml(t.name)}</div>`;
            }
        }
        html += '</div>';
        p.html = html;
        p.loading = false;
        nextTick(() => repositionAt(level));
        return;
    }

    const ext = entry.name.split('.').pop()?.toLowerCase();
    const isImage = IMAGE_EXTENSIONS.includes(ext);
    try {
        if (isImage) {
            const dataUrl = await invoke("read_file_as_base64", { filePath: entry.path });
            if (p.link !== link) return;
            p.html = typeof dataUrl === 'string'
                ? `<img src="${dataUrl}" style="max-width:100%;max-height:${level === 0 ? 260 : 220}px;display:block;border-radius:4px;" alt="${entry.name}" />`
                : '';
        } else {
            const content = await invoke("get_file_content", { filePath: entry.path });
            if (p.link !== link) return;
            const raw = typeof content === 'string' ? content : '';
            headingIdMap = {};
            taskCounter = 0;
            let html = previewMarked.parse(escapeSingleTilde(raw));
            html = convertWikiLinks(html);
            if (p.link !== link) return;
            html = await processLocalImages(html, entry.path);
            if (p.link !== link) return;
            p.html = html;
        }
    } catch { if (p.link === link && level === 0) p.html = ''; }
    finally { if (p.link === link) { p.loading = false; nextTick(() => repositionAt(level)); } }
};

// 关闭 level 及之后的所有预览
const closePreviewsFrom = (level) => {
    for (let i = level; i < MAX_PREVIEWS; i++) {
        const p = previews.value[i];
        if (!p) break;
        clearTimeout(p.timer);
        p.visible = false; p.link = null;
    }
    if (level === 0) { mainLink = null; clearTimeout(mainTimer); }
};

// 延迟关闭
const scheduleCloseFrom = (level) => {
    ensureLevel(level);
    const p = previews.value[level];
    clearTimeout(p.timer);
    p.timer = setTimeout(() => closePreviewsFrom(level), 300);
};

// ---- 预览弹窗鼠标事件 ----

const onPreviewMouseOver = (level) => {
    clearTimeout(previews.value[level]?.timer);
};

const onPreviewMouseOut = (level, e) => {
    if (e.relatedTarget && e.currentTarget.contains(e.relatedTarget)) return;
    if (e.relatedTarget?.closest?.('.wiki-link')) return;
    // 移到更深层弹窗不关闭
    for (let i = level + 1; i < MAX_PREVIEWS; i++) {
        if (previews.value[i]?.visible && e.relatedTarget?.closest?.(`.wiki-preview[data-level="${i}"]`)) return;
    }
    scheduleCloseFrom(level);
};

// ---- 预览 body 内链接悬停（触发下一层） ----

const onPreviewBodyMouseOver = (level, e) => {
    if (level + 1 >= MAX_PREVIEWS) return;
    const link = e.target.closest('.wiki-link');
    if (!link) return;
    const p = previews.value[level + 1];
    if (p?.link === link) return;
    // 关闭更深层（level+2 及之后），保留当前层旧内容避免闪烁
    closePreviewsFrom(level + 2);
    ensureLevel(level + 1);
    const next = previews.value[level + 1];
    clearTimeout(next.timer);
    next.link = link;
    const filename = link.dataset.filename;
    if (!filename) { next.link = null; return; }
    // 深层预览响应更快（150ms），第一层保持 500ms 防误触
    const delay = level === 0 ? 500 : 150;
    next.timer = setTimeout(() => {
        if (next.link === link) loadPreviewAt(level + 1, link, filename);
    }, delay);
};

const onPreviewBodyMouseOut = (level, e) => {
    const related = e.relatedTarget;
    if (level + 1 < MAX_PREVIEWS && related?.closest?.('.wiki-link') === previews.value[level + 1]?.link) return;
    for (let i = level + 1; i < MAX_PREVIEWS; i++) {
        if (related?.closest?.(`.wiki-preview[data-level="${i}"]`)) return;
    }
    scheduleCloseFrom(level + 1);
};

// ---- 主内容区链接悬停事件委托 ----

const handleContainerMouseOver = (e) => {
    const link = e.target.closest('.wiki-link');
    if (e.target.closest('.wiki-preview')) {
        clearTimeout(mainTimer);
        return;
    }
    if (!link) {
        if (mainLink && !e.target.closest('.wiki-preview')) scheduleCloseFrom(0);
        return;
    }
    if (link === mainLink) {
        clearTimeout(previews.value[0]?.timer); // 取消待处理的关闭计时
        return;
    }
    clearTimeout(mainTimer);
    closePreviewsFrom(0);
    mainLink = link;
    const filename = link.dataset.filename;
    if (!filename) { mainLink = null; return; }
    mainTimer = setTimeout(() => {
        if (mainLink === link) {
            // 主链接 → level 0 预览
            ensureLevel(0);
            previews.value[0].link = link;
            loadPreviewAt(0, link, filename);
        }
    }, 500);
};

const handleContainerMouseOut = (e) => {
    const related = e.relatedTarget;
    if (related?.closest?.('.wiki-link') === mainLink) return;
    if (related?.closest?.('.wiki-preview')) { clearTimeout(mainTimer); return; }
    scheduleCloseFrom(0);
};

// ---- 以下旧代码已移除 ----
const handleMenuClick = async ({ key }) => {
    if (!key) return;
    if (key === "read-newBookMark") {
        await handleCreateBookmark();
        return;
    }
    if (key === "read-makeLink") {
        const selectedText = window.getSelection?.()?.toString?.().trim?.() || "";
        window.dispatchEvent(new CustomEvent("simple-write:open-make-link", {
            detail: {
                filePath: props.filePath,
                targetText: selectedText || props.fileName || "",
                selectedText,
            },
        }));
        return;
    }
    message.info("阅读视图右键功能待定");
};

const bodyEl = ref(null);
const rightClickSnippet = ref("");

// ---- 标题折叠/展开 ----
const collapsedHeadings = ref(new Set());

const setupCollapsibleHeadings = () => {
    const body = bodyEl.value;
    if (!body) return;

    // 清理上一次渲染的残留
    body.querySelectorAll('.section-body').forEach(w => {
        while (w.firstChild) w.parentNode.insertBefore(w.firstChild, w);
        w.remove();
    });
    body.querySelectorAll('.heading-toggle').forEach(t => t.remove());

    const headings = Array.from(body.querySelectorAll('h1, h2, h3, h4, h5, h6'));
    if (headings.length === 0) return;

    // 从后往前处理，避免 DOM 变动影响后续节点索引
    for (let i = headings.length - 1; i >= 0; i--) {
        const heading = headings[i];
        const level = parseInt(heading.tagName[1]);

        // 收集当前标题之后、下一个同级/上级标题之前的所有兄弟节点
        const contentNodes = [];
        let node = heading.nextSibling;
        while (node) {
            if (node.nodeType === Node.ELEMENT_NODE && /^H[1-6]$/.test(node.tagName)) {
                const nodeLevel = parseInt(node.tagName[1]);
                if (nodeLevel <= level) break;
            }
            const next = node.nextSibling;
            contentNodes.push(node);
            node = next;
        }

        if (contentNodes.length > 0) {
            const wrapper = document.createElement('div');
            wrapper.className = 'section-body';
            contentNodes.forEach(n => wrapper.appendChild(n));
            heading.parentNode.insertBefore(wrapper, heading.nextSibling);

            const key = `${heading.tagName}:${heading.textContent.trim()}`;
            if (collapsedHeadings.value.has(key)) {
                wrapper.style.display = 'none';
            }
        }

        // 插入折叠箭头
        const key = `${heading.tagName}:${heading.textContent.trim()}`;
        const isCollapsed = collapsedHeadings.value.has(key);
        const toggle = document.createElement('span');
        toggle.className = 'heading-toggle';
        toggle.textContent = isCollapsed ? '▶' : '▼';
        heading.insertBefore(toggle, heading.firstChild);
    }
};

// ---- 图片激活与拖拽缩放 ----
const activeImage = ref(null);
const resizing = ref(false);
const resizeStartX = ref(0);
const resizeStartWidth = ref(0);

const deactivateImage = () => {
    if (activeImage.value) {
        activeImage.value.style.outline = '';
        activeImage.value.style.cursor = '';
        activeImage.value = null;
    }
    // 不清除 width/maxWidth/maxHeight，持久化用户拖拽缩放结果
};

const handleImageMouseDown = (e) => {
    if (!activeImage.value || e.target !== activeImage.value) {
        deactivateImage();
        return;
    }
    if (e.button !== 0) return; // 仅左键
    e.preventDefault();
    // 锁定当前渲染宽度后解除 CSS 限制，避免尺寸跳变
    const currentWidth = activeImage.value.offsetWidth;
    activeImage.value.style.width = currentWidth + 'px';
    activeImage.value.style.maxWidth = 'none';
    activeImage.value.style.maxHeight = 'none';
    resizing.value = true;
    resizeStartX.value = e.clientX;
    resizeStartWidth.value = currentWidth;
    window.addEventListener('mousemove', onResizeMove);
    window.addEventListener('mouseup', onResizeUp);
};

const onResizeMove = (e) => {
    if (!resizing.value || !activeImage.value) return;
    const dx = e.clientX - resizeStartX.value;
    const newWidth = Math.max(80, resizeStartWidth.value + dx);
    activeImage.value.style.width = newWidth + 'px';
};

const onResizeUp = () => {
    resizing.value = false;
    window.removeEventListener('mousemove', onResizeMove);
    window.removeEventListener('mouseup', onResizeUp);
};

const handleImageMouseMove = () => {
    // 保留空壳：容器上的 mousemove 不再处理缩放，缩放由 window 级 onResizeMove 处理
};

const handleImageMouseUp = () => {
    // 保留空壳：缩放由 window 级 onResizeUp 处理
};

// 从点击坐标提取最多 maxChars 个前置字符
const getTextSnippetAtPoint = (x, y, maxChars = 20) => {
    let range = null;
    if (document.caretRangeFromPoint) {
        range = document.caretRangeFromPoint(x, y);
    } else if (document.caretPositionFromPoint) {
        const pos = document.caretPositionFromPoint(x, y);
        if (pos) {
            range = document.createRange();
            range.setStart(pos.offsetNode, pos.offset);
            range.collapse(true);
        }
    }
    if (!range || range.startContainer.nodeType !== Node.TEXT_NODE) return '';
    const text = range.startContainer.textContent;
    const offset = range.startOffset;
    const start = Math.max(0, offset - maxChars);
    return text.slice(start, offset).trim();
};

// 右键时记录点击处文本片段
const handleContextMenu = (e) => {
    const snippet = getTextSnippetAtPoint(e.clientX, e.clientY);
    rightClickSnippet.value = snippet;
};

// ==================== SearchList 搜索结果点击 → 滚动到匹配位置 ====================
const pendingScrollMatch = ref(null)

// 高亮可视区域内的匹配文本（不触发额外滚动）
const highlightNearby = (container, matchText) => {
    if (!matchText) return
    const viewTop = container.scrollTop
    const viewBottom = viewTop + container.clientHeight
    const walker = document.createTreeWalker(container, NodeFilter.SHOW_TEXT)
    let node
    while ((node = walker.nextNode())) {
        const idx = node.textContent.indexOf(matchText)
        if (idx === -1) continue
        const range = document.createRange()
        range.setStart(node, idx)
        range.setEnd(node, idx + matchText.length)
        const rects = range.getClientRects()
        // 检查是否在可视区域内
        for (const rect of rects) {
            if (rect.bottom >= viewTop && rect.top <= viewBottom) {
                const span = document.createElement('span')
                span.style.cssText = 'background:#ffd666;padding:0 2px;border-radius:2px;'
                range.surroundContents(span)
                setTimeout(() => {
                    const parent = span.parentNode
                    if (parent) {
                        while (span.firstChild) parent.insertBefore(span.firstChild, span)
                        parent.removeChild(span)
                        parent.normalize()
                    }
                }, 2000)
                return
            }
        }
        range.detach()
    }
}

// 等比滚动：用 startPos 计算相对位置，不受 markdown 渲染影响
const scrollToMatch = (container, matchText, startPos, rawLen) => {
    if (!container || rawLen <= 0) return false

    // 等比计算滚动位置
    const ratio = Math.min(1, Math.max(0, startPos / rawLen))
    container.scrollTop = ratio * (container.scrollHeight - container.clientHeight)

    // 滚动到位后高亮可视区域内的匹配文本
    if (matchText) {
        setTimeout(() => highlightNearby(container, matchText), 150)
    }

    return true
}

const tryScrollToMatch = () => {
    const pending = pendingScrollMatch.value
    if (!pending || !bodyEl.value) return
    nextTick(() => {
        if (scrollToMatch(bodyEl.value, pending.matchText, pending.startPos, pending.rawLen)) {
            pendingScrollMatch.value = null
        }
    })
}

const handleScrollToMatch = (e) => {
    const { filePath, matchText, startPos } = e.detail || {}
    if (!filePath || !matchText) return
    if (filePath !== props.filePath) return
    pendingScrollMatch.value = {
        matchText,
        startPos: startPos ?? 0,
        rawLen: (props.content || '').length,
    }
    tryScrollToMatch()
}

// 内容加载完成后重试滚动
watch(() => props.content, () => {
    if (pendingScrollMatch.value) {
        // 更新 rawLen（内容可能已加载）
        pendingScrollMatch.value.rawLen = (props.content || '').length
        tryScrollToMatch()
    }
})

// KeepAlive 缓存后重新激活时恢复滚动位置
onActivated(() => {
    if (props.restoreScrollTop > 0) {
        nextTick(() => {
            if (bodyEl.value && bodyEl.value.scrollHeight > props.restoreScrollTop) {
                bodyEl.value.scrollTop = props.restoreScrollTop;
            }
        });
    }
});

// KeepAlive 缓存前保存滚动位置（DOM 此时一定还在文档中）
onDeactivated(() => {
    const scrollTop = bodyEl.value?.scrollTop ?? 0;
    if (scrollTop > 0 && props.filePath) {
        window.dispatchEvent(new CustomEvent('simple-write:save-scroll', {
            detail: { filePath: props.filePath, scrollTop },
        }));
    }
});

onMounted(() => {
    window.addEventListener('simple-write:scroll-to-match', handleScrollToMatch)
})

onUnmounted(() => {
    window.removeEventListener('simple-write:scroll-to-match', handleScrollToMatch)
})
</script>

<style scoped>
.read-view__container {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
}

.read-view__body {
    margin: 0;
    padding: 16px;
    flex: 1;
    min-height: 0;
    border-radius: 8px;
    background: var(--bg-secondary, #fafafa);
    border: 1px solid var(--border-secondary, #f0f0f0);
    color: var(--text-primary, #262626);
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.6;
    font-size: var(--read-font-size, 16px);
    font-family: Consolas, "SFMono-Regular", Menlo, Monaco, "Liberation Mono", monospace;
    overflow: auto;
}

.read-view__body--empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
}
</style>

<style>
.read-view__body--markdown {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    font-size: var(--read-font-size, 16px);
    padding: 32px 40px;
    background: var(--bg-base, #fff);
    border-color: var(--border-color, #e8e8e8);
}

.read-view__body--markdown h1,
.read-view__body--markdown h2,
.read-view__body--markdown h3,
.read-view__body--markdown h4,
.read-view__body--markdown h5,
.read-view__body--markdown h6 {
    margin-top: 1.2em;
    margin-bottom: 0.6em;
    font-weight: 600;
    line-height: 1.3;
    color: var(--text-primary, #1f1f1f);
}

/* 标题折叠箭头 */
.read-view__body--markdown .heading-toggle {
    cursor: pointer;
    margin-right: 6px;
    font-size: 0.7em;
    color: #bfbfbf;
    user-select: none;
    vertical-align: middle;
    display: inline-block;
    transition: color 0.15s;
}
.read-view__body--markdown .heading-toggle:hover {
    color: #595959;
}
.read-view__body--markdown h1 { font-size: 1.8em; border-bottom: 1px solid var(--border-secondary, #eee); padding-bottom: 0.3em; }
.read-view__body--markdown h2 { font-size: 1.5em; border-bottom: 1px solid var(--border-secondary, #eee); padding-bottom: 0.25em; }
.read-view__body--markdown h3 { font-size: 1.25em; }
.read-view__body--markdown h4 { font-size: 1.1em; }

.read-view__body--markdown p { margin: 0.6em 0; }

.read-view__body--markdown ul,
.read-view__body--markdown ol {
    padding-left: 2em;
    margin: 0.3em 0;
}

.read-view__body--markdown li { margin: 0.05em 0; }

/* 列表项内 <p> 不产生额外间距（松散列表解析时 marked 会在 li 内包裹 <p>） */
.read-view__body--markdown li > p { margin: 0; }

/* 任务列表：隐藏浏览器默认圆点 */
.read-view__body--markdown .task-list-item {
    list-style: none;
}

.read-view__body--markdown .task-list-item input[type="checkbox"] {
    margin-right: 6px;
    cursor: pointer;
    width: 14px;
    height: 14px;
    accent-color: #1890ff;
}

.read-view__body--markdown code {
    background: var(--bg-tertiary, #f5f5f5);
    border: 1px solid var(--border-secondary, #eee);
    border-radius: 3px;
    padding: 2px 6px;
    font-family: Consolas, "SFMono-Regular", Menlo, Monaco, "Liberation Mono", monospace;
    font-size: 0.9em;
}

.read-view__body--markdown pre {
    background: var(--bg-secondary, #f6f8fa);
    border: 1px solid var(--border-color, #e8e8e8);
    border-radius: 6px;
    padding: 16px;
    overflow: auto;
    margin: 0.8em 0;
}

.read-view__body--markdown pre code {
    background: none;
    border: none;
    padding: 0;
    font-size: 0.85em;
}

.read-view__body--markdown blockquote {
    border-left: 4px solid #1890ff;
    padding: 4px 16px;
    margin: 0.8em 0;
    color: var(--text-secondary, #595959);
    background: var(--bg-secondary, #fafafa);
}

.read-view__body--markdown table {
    border-collapse: collapse;
    margin: 0.8em 0;
    width: 100%;
}

.read-view__body--markdown th,
.read-view__body--markdown td {
    border: 1px solid var(--border-color, #e8e8e8);
    padding: 8px 12px;
    text-align: left;
}

.read-view__body--markdown th {
    background: var(--bg-secondary, #fafafa);
    font-weight: 600;
}

.read-view__body--markdown hr {
    border: none;
    border-top: 2px solid var(--border-color, #e8e8e8);
    margin: 1.5em 0;
}

.read-view__body--markdown a {
    color: #1890ff;
    text-decoration: none;
}

.read-view__body--markdown a:hover { text-decoration: underline; }

.read-view__body--markdown img {
    max-width: 200px;
    max-height: 200px;
    width: auto;
    height: auto;
    object-fit: contain;
    border-radius: 4px;
    cursor: pointer;
    transition: outline 0.15s;
}

/* 脚注样式 */
.read-view__body--markdown .footnotes {
    margin-top: 2em;
    padding-top: 1em;
    border-top: 1px solid var(--border-color, #e8e8e8);
    font-size: 0.9em;
    color: var(--text-secondary, #595959);
}

/* 隐藏扩展生成的英文标题，用本地化文本替代 */
.read-view__body--markdown .footnotes > h2 {
    font-size: 0;
    margin: 0 0 0.6em;
}

.read-view__body--markdown .footnotes > h2::before {
    content: var(--footnote-heading, "Footnotes");
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--text-primary, #1f1f1f);
}

.read-view__body--markdown .footnotes ol {
    padding-left: 1.5em;
    margin: 0;
}

.read-view__body--markdown .footnotes li {
    margin: 0.15em 0;
    list-style-position: inside;
}

/* 脚注内容紧接序号，不换行 */
.read-view__body--markdown .footnotes li p {
    display: contents;
}

.read-view__body--markdown .footnote-ref {
    font-size: 0.75em;
    vertical-align: super;
    line-height: 1;
}

.read-view__body--markdown .footnote-backref {
    margin-left: 4px;
    font-size: 0.85em;
    text-decoration: none;
}

/* ---- 链接悬停预览弹窗 ---- */
.wiki-preview {
    position: fixed;
    z-index: 1060;
    width: 440px;
    max-height: 300px;
    background: var(--bg-elevated, #fff);
    border-radius: 8px;
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08), 0 3px 6px rgba(0, 0, 0, 0.12);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.wiki-preview--nested {
    width: 360px;
    max-height: 260px;
    z-index: 1070;
}

.wiki-preview__header {
    padding: 6px 12px 8px;
    font-size: 12px;
    color: var(--text-tertiary, #8c8c8c);
    border-bottom: 1px solid var(--border-secondary, #f0f0f0);
    flex-shrink: 0;
}

.wiki-preview__loading {
    padding: 24px 12px;
    text-align: center;
    color: #bfbfbf;
    font-size: 13px;
}

.wiki-preview__body {
    margin: 0;
    padding: 12px 16px;
    flex: 1;
    min-height: 80px;
    overflow: auto;
    background: var(--bg-base, #fff);
    color: var(--text-primary, #262626);
}

.wiki-preview__body :deep(*) {
    color: inherit;
}

/* v-html 内的链接样式（需要 :deep() 穿透） */
.wiki-preview__body :deep(a) {
    color: #1890ff;
    text-decoration: none;
}

.wiki-preview__body :deep(a:hover) {
    text-decoration: underline;
}

.wiki-preview__body :deep(img) {
    max-width: 100%;
    width: auto;
    height: auto;
    border-radius: 4px;
}
</style>
