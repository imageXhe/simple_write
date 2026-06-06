<template>
    <a-dropdown :trigger="['contextmenu']" placement="bottomLeft">
    <div class="txt-read-view">
        <div
            v-if="content.length === 0"
            class="txt-read-view__empty"
        >
            <a-empty :description="t('content.noData')" />
        </div>
        <div
            v-else
            ref="bodyEl"
            class="txt-read-view__body"
            v-html="renderedHtml"
            @mouseover="handleBodyMouseOver"
            @mousemove="handleBodyMouseMove"
            @mouseout="handleBodyMouseOut"
        />
    </div>
        <template #overlay>
            <a-menu class="dropdown-menu-bordered" @click="handleMenuClick">
                <a-menu-item key="txt-read-makeLink">{{ t('txtEditor.makeLink') }}</a-menu-item>
            </a-menu>
        </template>
    </a-dropdown>
    <div
        v-if="preview.visible"
        class="txt-read-preview"
        :style="{ left: preview.x + 'px', top: preview.y + 'px' }"
    >
        <div class="txt-read-preview__header">{{ preview.title }}</div>
        <div class="txt-read-preview__body">{{ preview.content }}</div>
    </div>
</template>

<script setup>
import { computed, ref, watch, nextTick, onActivated, onDeactivated } from "vue";
import { getFileMeta, enabledInsertTypes, enabledMarkTypes } from "../stores/novelStore";
import { useI18n } from "../locales";

const { t } = useI18n();

const props = defineProps({
    content: { type: String, required: true },
    filePath: { type: String, default: "" },
    fileName: { type: String, default: "" },
    topLine: { type: Number, default: 0 },
    restoreScrollTop: { type: Number, default: 0 },
});

const bodyEl = ref(null);
const preview = ref({
    visible: false,
    title: "",
    content: "",
    x: 0,
    y: 0,
});

const esc = (s) => String(s || "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

const renderedHtml = computed(() => {
    const text = props.content ?? "";
    const meta = getFileMeta(props.filePath);
    const marks = meta?.marks || [];
    const inserts = meta?.inserts || [];

    const charStyles = new Array(text.length).fill(null);
    for (const mark of marks) {
        const start = mark.range?.start?.utf16Offset ?? 0;
        const end = mark.range?.end?.utf16Offset ?? 0;
        const markType = enabledMarkTypes.value.find((item) => item.id === mark.typeId);
        if (!markType) continue;
        for (let i = Math.max(0, start); i < Math.min(text.length, end); i++) {
            charStyles[i] = markType;
        }
    }

    const insertMap = new Map();
    for (const ins of inserts) {
        const pos = ins.anchor?.utf16Offset ?? 0;
        const insertType = enabledInsertTypes.value.find((item) => item.id === ins.typeId);
        if (!insertType) continue;
        const tipParts = [insertType.name];
        if (ins.payload) tipParts.push(ins.payload);
        if (ins.linkedText) tipParts.push(ins.linkedText);
        if (ins.linkedFilePath) tipParts.push(ins.linkedFilePath.split("/").pop() || ins.linkedFilePath);
        const arr = insertMap.get(pos) || [];
        arr.push({
            color: insertType.color,
            title: insertType.name,
            tip: tipParts.slice(1).join(" | ") || insertType.name,
        });
        insertMap.set(pos, arr);
    }

    let html = "";
    let currentStyle = null;
    let inSpan = false;

    for (let i = 0; i <= text.length; i++) {
        if (insertMap.has(i)) {
            if (inSpan) {
                html += "</span>";
                inSpan = false;
                currentStyle = null;
            }
            for (const ins of insertMap.get(i)) {
                html += `<sub class="txt-read-view__insert" data-preview-title="${esc(ins.title)}" data-preview-content="${esc(ins.tip)}" style="color:${ins.color}">▾</sub>`;
            }
        }
        if (i === text.length) {
            if (inSpan) html += "</span>";
            break;
        }

        const style = charStyles[i];
        const styleKey = style ? `${style.bgColor}|${style.textColor}|${style.name}` : null;
        if (styleKey !== currentStyle) {
            if (inSpan) {
                html += "</span>";
                inSpan = false;
            }
            if (style) {
                html += `<span class="txt-read-view__mark" data-preview-title="${t('txtEditor.mark')}" data-preview-content="${esc(style.name)}" style="background:${style.bgColor};color:${style.textColor}">`;
                inSpan = true;
            }
            currentStyle = styleKey;
        }

        const ch = text[i];
        html += ch === "\n" ? "<br>" : esc(ch);
    }

    return html || "<br>";
});

// 滚动恢复
watch(
    () => props.content,
    () => {
        if (props.restoreScrollTop > 0) {
            nextTick(() => {
                if (bodyEl.value && bodyEl.value.scrollHeight > props.restoreScrollTop) {
                    bodyEl.value.scrollTop = props.restoreScrollTop;
                }
            });
        }
    },
    { immediate: true }
);

onActivated(() => {
    if (props.restoreScrollTop > 0) {
        nextTick(() => {
            if (bodyEl.value && bodyEl.value.scrollHeight > props.restoreScrollTop) {
                bodyEl.value.scrollTop = props.restoreScrollTop;
            }
        });
    }
});

onDeactivated(() => {
    const scrollTop = bodyEl.value?.scrollTop ?? 0;
    if (scrollTop > 0 && props.filePath) {
        window.dispatchEvent(new CustomEvent('simple-write:save-scroll', {
            detail: { filePath: props.filePath, scrollTop },
        }));
    }
});

const handleMenuClick = ({ key }) => {
    if (key !== "txt-read-makeLink") return;
    const selectedText = window.getSelection?.()?.toString?.().trim?.() || "";
    window.dispatchEvent(new CustomEvent("simple-write:open-make-link", {
        detail: {
            filePath: props.filePath,
            targetText: selectedText || props.fileName || "",
            selectedText,
        },
    }));
};

const updatePreviewPosition = (event) => {
    preview.value = {
        ...preview.value,
        x: Math.min(window.innerWidth - 280, event.clientX + 18),
        y: Math.min(window.innerHeight - 140, event.clientY + 18),
    };
};

const showPreviewForTarget = (target, event) => {
    const title = target?.getAttribute?.("data-preview-title") || "";
    const content = target?.getAttribute?.("data-preview-content") || "";
    if (!title && !content) return false;
    preview.value = {
        visible: true,
        title: title || t('txtEditor.detail'),
        content: content || title,
        x: preview.value.x,
        y: preview.value.y,
    };
    updatePreviewPosition(event);
    return true;
};

const handleBodyMouseOver = (event) => {
    const target = event.target?.closest?.(".txt-read-view__insert, .txt-read-view__mark");
    if (!target) return;
    showPreviewForTarget(target, event);
};

const handleBodyMouseMove = (event) => {
    if (!preview.value.visible) return;
    updatePreviewPosition(event);
};

const handleBodyMouseOut = (event) => {
    const related = event.relatedTarget;
    if (related?.closest?.(".txt-read-view__insert, .txt-read-view__mark")) return;
    preview.value.visible = false;
};
</script>

<style scoped>
.txt-read-view {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
}

.txt-read-view__empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
}

.txt-read-view__body {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 32px 40px;
    border-radius: 8px;
    background: var(--bg-base, #fff);
    border: 1px solid var(--border-color, #e8e8e8);
    color: var(--text-primary, #262626);
    font-size: var(--read-font-size, 16px);
    line-height: 1.8;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    white-space: pre-wrap;
    word-break: break-word;
}

.txt-read-view__body :deep(.txt-read-view__mark) {
    border-radius: 3px;
    padding: 0 1px;
}

.txt-read-view__body :deep(.txt-read-view__insert) {
    margin: 0 1px;
    vertical-align: sub;
    font-size: 0.78em;
    font-weight: 700;
    cursor: help;
}

.txt-read-preview {
    position: fixed;
    z-index: 1060;
    width: 260px;
    max-height: 220px;
    background: var(--bg-elevated, #fff);
    border-radius: 8px;
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08), 0 3px 6px rgba(0, 0, 0, 0.12);
    overflow: hidden;
    pointer-events: none;
}

.txt-read-preview__header {
    padding: 6px 12px 8px;
    font-size: 12px;
    color: var(--text-tertiary, #8c8c8c);
    border-bottom: 1px solid var(--border-secondary, #f0f0f0);
}

.txt-read-preview__body {
    padding: 12px 16px;
    color: var(--text-primary, #262626);
    font-size: 13px;
    line-height: 1.6;
    white-space: pre-wrap;
}
</style>
