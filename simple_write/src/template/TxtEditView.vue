<template>
    <a-dropdown :trigger="['contextmenu']" placement="bottomLeft">
        <div class="txt-edit-view">
            <div
                class="txt-edit-view__wrap"
                ref="editorWrap"
            >
                <div ref="mirrorLayer" class="txt-edit-view__mirror" aria-hidden="true" v-html="mirrorHtml"></div>
                <textarea
                    ref="editorTextarea"
                    :value="editorTextRef"
                    class="txt-edit-view__textarea"
                    spellcheck="false"
                    :disabled="disabled"
                    @input="handleInput"
                    @scroll="syncMirrorScroll"
                    @mouseup="onMouseUp"
                    @keyup="onMouseUp"
                />
            </div>
            <div class="txt-edit-view__status" :class="{ 'txt-edit-view__status--dirty': isDirty }">
                <span class="txt-edit-view__char-count">{{ contentRef.length }} {{ t('content.charCount') }}</span>
                <span v-if="isSaving">{{ t('content.saving') }}</span>
                <span v-else-if="isDirty">{{ t('content.notSaved') }}</span>
                <span v-else>{{ t('content.saved') }}</span>
            </div>
        </div>

        <template #overlay>
            <a-menu class="dropdown-menu-bordered" @click="handleMenuClick">
                <a-menu-item key="txt-insert" :icon="h(PlusCircleOutlined)">{{ t('txtEditor.insert') }}</a-menu-item>
                <a-menu-item key="txt-mark" :icon="h(HighlightOutlined)" :disabled="!hasSelection">{{ t('txtEditor.markSelection') }}</a-menu-item>
                <a-menu-item v-if="cursorInMark" key="txt-unmark" :icon="h(ClearOutlined)">{{ t('txtEditor.unmark') }}</a-menu-item>
                <a-menu-item key="txt-make-link" :icon="h(LinkOutlined)" :disabled="!hasSelection">{{ t('txtEditor.makeLink') }}</a-menu-item>
                <a-menu-divider />
                <a-menu-item key="txt-cut" :icon="h(ScissorOutlined)" :disabled="!hasSelection">{{ t('content.cut') }}</a-menu-item>
                <a-menu-item key="txt-copy" :icon="h(CopyOutlined)" :disabled="!hasSelection">{{ t('content.copy') }}</a-menu-item>
                <a-menu-item key="txt-paste" :icon="h(CarryOutOutlined)">{{ t('content.paste') }}</a-menu-item>
                <a-menu-item key="txt-selectAll" :icon="h(ExpandOutlined)">{{ t('content.selectAll') }}</a-menu-item>
            </a-menu>
        </template>
    </a-dropdown>

    <!-- 插入弹窗 -->
    <a-modal v-model:open="insertModalOpen" :title="t('txtEditor.insert')" :footer="null" width="380px">
        <div class="ins-modal">
            <a-select v-model:value="insertTypeId" :placeholder="t('txtEditor.selectInsertType')" style="width:100%">
                <a-select-option v-for="t in enabledInsertTypes" :key="t.id" :value="t.id">
                    <span class="ins-dot" :style="{background:t.color}"></span>{{ t.name }}
                </a-select-option>
            </a-select>
            <a-textarea v-model:value="insertPayload" :placeholder="t('txtEditor.enterContent')" :rows="3" style="margin-top:10px" />
            <a-select
                v-model:value="insertLinks"
                :placeholder="t('txtEditor.selectLinkOptional')"
                style="width:100%;margin-top:10px"
                :options="customLinkOptions"
                :disabled="insertTypeId !== 'hidden-link'"
            />
            <a-button type="primary" block style="margin-top:12px" @click="doInsert" :disabled="!insertTypeId">{{ t('txtEditor.confirmInsert') }}</a-button>
        </div>
    </a-modal>

    <!-- 标记弹窗 -->
    <a-modal v-model:open="markModalOpen" :title="t('txtEditor.selectMarkType')" :footer="null" width="320px">
        <div v-if="enabledMarkTypes.length === 0" style="color:var(--text-tertiary,#8c8c8c);text-align:center;padding:16px;">{{ t('txtEditor.noMarkTypes') }}</div>
        <div v-for="t in enabledMarkTypes" :key="t.id" class="mark-type-item" @click="doMark(t)">
            <span class="mark-swatch" :style="{background:t.bgColor,color:t.textColor}">Abc</span>
            <span class="mark-type-item__name">{{ t.name }}</span>
        </div>
    </a-modal>
</template>

<script setup>
import { computed, inject, ref, watch, h } from "vue";
import {
    HighlightOutlined,
    ClearOutlined,
    LinkOutlined,
    ScissorOutlined,
    CopyOutlined,
    CarryOutOutlined,
    ExpandOutlined,
    PlusCircleOutlined,
} from "@ant-design/icons-vue";
import { enabledInsertTypes, enabledMarkTypes, getFileMeta, addInsert, addMark, removeMark, removeInsert, generateId, saveTxtMeta } from "../stores/novelStore";
import { readCustomLinks } from "../menu/novelActions";
import { useI18n } from "../locales";

const { t } = useI18n();

const props = defineProps({
    initialContent: { type: String, default: "" },
    originalContent: { type: String, default: "" },
    disabled: { type: Boolean, default: false },
    isDirty: { type: Boolean, default: false },
    isSaving: { type: Boolean, default: false },
    filePath: { type: String, default: "" },
});

const updateTab = inject("updateTab", () => {});
const activeTabId = inject("activeTabId", ref(""));
const editorTextarea = ref(null);
const mirrorLayer = ref(null);
const editorWrap = ref(null);
const contentRef = ref(props.initialContent);
const editorTextRef = ref(props.initialContent);
let lastContent = props.initialContent || "";
const customLinks = ref([]);
const editingInsertId = ref("");

watch(() => props.initialContent, (val) => {
    const v = val ?? "";
    if (v !== contentRef.value) {
        contentRef.value = v;
        lastContent = v;
        rebuildEditorText(v);
    }
});

const buildTextPosition = (offset) => ({ line: 0, column: 0, utf16Offset: Math.max(0, offset) });

const sortInsertsByOffset = (inserts = []) => {
    return [...inserts].sort((a, b) => (a?.anchor?.utf16Offset ?? 0) - (b?.anchor?.utf16Offset ?? 0));
};

const buildEditorText = (text, inserts = []) => {
    const source = text || "";
    const ordered = sortInsertsByOffset(inserts);
    let result = "";
    let cursor = 0;
    for (const ins of ordered) {
        const offset = Math.max(0, Math.min(source.length, ins?.anchor?.utf16Offset ?? 0));
        result += source.slice(cursor, offset);
        result += `▾${ins?.payload || ins?.linkedText || ins?.typeName || ""}▾`;
        cursor = offset;
    }
    result += source.slice(cursor);
    return result;
};

const rebuildEditorText = (text = contentRef.value) => {
    const meta = getFileMeta(props.filePath);
    editorTextRef.value = buildEditorText(text, meta.inserts || []);
};

const getTokenRanges = (displayText) => {
    const ranges = [];
    const text = displayText || "";
    let pureOffset = 0;
    let i = 0;
    while (i < text.length) {
        if (text[i] === "▾") {
            const close = text.indexOf("▾", i + 1);
            if (close > i + 1) {
                ranges.push({
                    start: i,
                    end: close + 1,
                    pureOffset,
                    payload: text.slice(i + 1, close),
                });
                i = close + 1;
                continue;
            }
        }
        pureOffset += 1;
        i += 1;
    }
    return ranges;
};

const displayOffsetToPureOffset = (displayText, offset) => {
    const text = displayText || "";
    const clamped = Math.max(0, Math.min(offset, text.length));
    let pureOffset = 0;
    let i = 0;
    while (i < clamped) {
        if (text[i] === "▾") {
            const close = text.indexOf("▾", i + 1);
            if (close > i + 1) {
                if (clamped <= close + 1) {
                    return pureOffset;
                }
                i = close + 1;
                continue;
            }
        }
        pureOffset += 1;
        i += 1;
    }
    return pureOffset;
};

const parseEditorText = (displayText, existingInserts = []) => {
    const ordered = sortInsertsByOffset(existingInserts);
    const nextInserts = [];
    let pureText = "";
    let pureOffset = 0;
    let i = 0;
    let insertIndex = 0;

    while (i < displayText.length) {
        if (displayText[i] === "▾") {
            const close = displayText.indexOf("▾", i + 1);
            if (close > i + 1) {
                const payload = displayText.slice(i + 1, close);
                const previous = ordered[insertIndex] || null;
                nextInserts.push({
                    ...(previous || {}),
                    id: previous?.id || generateId(),
                    typeId: previous?.typeId || enabledInsertTypes.value[0]?.id || "annotation",
                    anchor: buildTextPosition(pureOffset),
                    payload,
                });
                i = close + 1;
                insertIndex += 1;
                continue;
            }
        }
        pureText += displayText[i];
        pureOffset += 1;
        i += 1;
    }

    return { pureText, inserts: nextInserts };
};

const clampRange = (mark) => {
    mark.range.start.utf16Offset = Math.max(0, mark.range.start.utf16Offset ?? 0);
    mark.range.end.utf16Offset = Math.max(mark.range.start.utf16Offset, mark.range.end.utf16Offset ?? 0);
};

function transformPosDelete(pos, delStart, delEnd) {
    if (pos < delStart) return pos;
    if (pos > delEnd) return pos - (delEnd - delStart);
    return delStart;
}

function transformPosInsert(pos, insertPos, insertLen, affinity) {
    if (insertLen <= 0) return pos;
    if (pos < insertPos) return pos;
    if (pos > insertPos) return pos + insertLen;
    return affinity === "right" ? pos + insertLen : pos;
}

function transformPos(pos, start, oldEnd, newEnd, affinity) {
    const afterDelete = transformPosDelete(pos, start, oldEnd);
    return transformPosInsert(afterDelete, start, newEnd - start, affinity);
}

// ---- 位置同步：文本变更后更新标记和插入点的偏移 ----
function adjustPositions(oldText, newText) {
    if (oldText === newText) return;
    const meta = getFileMeta(props.filePath);
    let changed = false;

    // 找到第一个不同的字符位置
    let start = 0;
    while (start < oldText.length && start < newText.length && oldText[start] === newText[start]) start++;
    let oldEnd = oldText.length;
    let newEnd = newText.length;
    while (oldEnd > start && newEnd > start && oldText[oldEnd - 1] === newText[newEnd - 1]) { oldEnd--; newEnd--; }
    // 调整插入点
    for (const ins of meta.inserts) {
        const pos = ins.anchor?.utf16Offset ?? 0;
        const nextPos = transformPos(pos, start, oldEnd, newEnd, "right");
        if (nextPos !== pos) {
            ins.anchor = buildTextPosition(nextPos);
            changed = true;
        }
    }

    // 调整标记范围
    for (const mk of meta.marks) {
        const ms = mk.range?.start?.utf16Offset ?? 0;
        const me = mk.range?.end?.utf16Offset ?? 0;
        const nextStart = transformPos(ms, start, oldEnd, newEnd, "right");
        const nextEnd = transformPos(me, start, oldEnd, newEnd, "left");
        if (nextStart !== ms || nextEnd !== me) {
            mk.range.start = buildTextPosition(nextStart);
            mk.range.end = buildTextPosition(Math.max(nextStart, nextEnd));
            clampRange(mk);
            changed = true;
        }
    }
    return changed;
}

const handleInput = (event) => {
    const displayText = event.target.value;
    const meta = getFileMeta(props.filePath);
    const parsed = parseEditorText(displayText, meta.inserts || []);
    const changed = adjustPositions(lastContent, parsed.pureText);
    meta.inserts = parsed.inserts;
    contentRef.value = parsed.pureText;
    editorTextRef.value = displayText;
    lastContent = parsed.pureText;
    syncValue(parsed.pureText);
    if (changed) {
        saveTxtMeta();
    } else {
        saveTxtMeta();
    }
};

const syncValue = (value = contentRef.value) => {
    const tabId = activeTabId.value;
    if (!tabId) return;
    contentRef.value = value;
    updateTab(tabId, { draftContent: value, isDirty: value !== (props.originalContent ?? "") });
};

const loadCustomLinks = async () => {
    try {
        customLinks.value = await readCustomLinks();
    } catch {
        customLinks.value = [];
    }
};

const customLinkOptions = computed(() => {
    return (customLinks.value || []).map((link) => ({
        label: link.name,
        value: link.id,
    }));
});

const mirrorHtml = computed(() => {
    const text = editorTextRef.value || "";
    const meta = getFileMeta(props.filePath);
    const marks = meta?.marks || [];
    const tokenRanges = getTokenRanges(text);
    const tokenMap = new Map(tokenRanges.map((range) => [range.start, range]));

    const charStyles = [];
    for (const mark of marks) {
        const start = mark.range?.start?.utf16Offset ?? 0;
        const end = mark.range?.end?.utf16Offset ?? 0;
        const mt = (enabledMarkTypes.value || []).find(t => t.id === mark.typeId);
        if (mt) {
            for (let i = Math.max(0, start); i < end; i++) {
                charStyles[i] = { bg: mt.bgColor, fg: mt.textColor, title: mt.name };
            }
        }
    }

    const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

    let html = "";
    let inSpan = false;
    let currentStyle = null;

    let pureOffset = 0;
    for (let i = 0; i < text.length;) {
        const tokenRange = tokenMap.get(i);
        if (tokenRange) {
            if (inSpan) { html += "</span>"; inSpan = false; currentStyle = null; }
            const insert = (meta.inserts || []).find((item) => (item.anchor?.utf16Offset ?? -1) === tokenRange.pureOffset && (item.payload || "") === tokenRange.payload);
            const insertType = (enabledInsertTypes.value || []).find((item) => item.id === insert?.typeId);
            const color = insertType?.color || "#1890ff";
            html += `<span class="txtev-ins-inline" style="color:${color}">▾${esc(tokenRange.payload)}▾</span>`;
            i = tokenRange.end;
            continue;
        }

        const cs = charStyles[pureOffset];
        const styleStr = cs ? `background-color:${cs.bg};color:${cs.fg}` : null;
        if (styleStr !== currentStyle) {
            if (inSpan) { html += "</span>"; inSpan = false; }
            if (styleStr) {
                html += `<span class="txtev-mark" data-tip="${esc(cs.title || '')}" style="${styleStr}">`;
                inSpan = true;
            }
            currentStyle = styleStr;
        }
        html += text[i] === "\n" ? "<br>" : esc(text[i]);
        pureOffset += 1;
        i += 1;
    }
    if (inSpan) html += "</span>";
    return html || "<br>";
});

// ---- 选区 ----
const selStart = ref(0);
const selEnd = ref(0);
const hasSelection = computed(() => selStart.value !== selEnd.value);

const onMouseUp = () => {
    const ta = editorTextarea.value;
    if (!ta) return;
    selStart.value = ta.selectionStart;
    selEnd.value = ta.selectionEnd;
};

// ---- 插入 ----
const insertModalOpen = ref(false);
const insertTypeId = ref("");
const insertPayload = ref("");
const insertLinks = ref(undefined);

const openInsertModal = () => {
    const ta = editorTextarea.value;
    if (!ta) return;
    insertTypeId.value = enabledInsertTypes.value[0]?.id || "";
    insertPayload.value = "";
    insertLinks.value = undefined;
    editingInsertId.value = "";
    loadCustomLinks();
    insertModalOpen.value = true;
};

const findInsertNearCursor = () => {
    const meta = getFileMeta(props.filePath);
    const pos = selStart.value;
    const displayText = editorTextRef.value || "";
    const tokenRanges = getTokenRanges(displayText);
    const matchedRange = tokenRanges.find((range) => pos >= range.start && pos <= range.end);
    if (!matchedRange) return null;
    return (meta.inserts || []).find((ins) => {
        const anchor = ins.anchor?.utf16Offset ?? -999;
        return anchor === matchedRange.pureOffset && (ins.payload || "") === matchedRange.payload;
    }) || null;
};

const doInsert = () => {
    const ta = editorTextarea.value;
    if (!ta || !insertTypeId.value) return;
    const it = enabledInsertTypes.value.find(t => t.id === insertTypeId.value);
    const meta = getFileMeta(props.filePath);
    const existing = editingInsertId.value
        ? (meta.inserts || []).find((item) => item.id === editingInsertId.value)
        : null;
    const ins = existing || {
        id: generateId(),
        typeId: insertTypeId.value,
        anchor: { line: 0, column: 0, utf16Offset: ta.selectionStart },
        payload: insertPayload.value.trim(),
    };
    ins.typeId = insertTypeId.value;
    ins.payload = insertPayload.value.trim();
    if (!existing) {
        ins.anchor = { line: 0, column: 0, utf16Offset: displayOffsetToPureOffset(editorTextRef.value, ta.selectionStart) };
    }
    delete ins.linkedFilePath;
    delete ins.linkedText;
    delete ins.customLinkId;
    if (it?.id === "hidden-link" && insertLinks.value) {
        const link = (customLinks.value || []).find((item) => item.id === insertLinks.value);
        if (link) {
            ins.payload = link.name;
            ins.linkedFilePath = link.targetPath;
            ins.linkedText = link.targetText || link.targetPath.split("/").pop() || link.targetPath;
            ins.customLinkId = link.id;
        }
    }
    if (!existing) {
        addInsert(props.filePath, ins);
    } else {
        saveTxtMeta();
    }
    rebuildEditorText(contentRef.value);
    editingInsertId.value = "";
    insertModalOpen.value = false;
};

// ---- 标记 ----
const markModalOpen = ref(false);

const doMark = (type) => {
    const start = displayOffsetToPureOffset(editorTextRef.value, Math.min(selStart.value, selEnd.value));
    const end = displayOffsetToPureOffset(editorTextRef.value, Math.max(selStart.value, selEnd.value));
    if (start === end) return;
    addMark(props.filePath, {
        id: generateId(),
        typeId: type.id,
        range: {
            start: { line: 0, column: 0, utf16Offset: start },
            end: { line: 0, column: 0, utf16Offset: end },
        },
    });
    markModalOpen.value = false;
};

// ---- 取消标记检测 ----
const cursorInMark = computed(() => {
    const pos = displayOffsetToPureOffset(editorTextRef.value, selStart.value);
    const meta = getFileMeta(props.filePath);
    for (const mk of meta.marks || []) {
        const ms = mk.range?.start?.utf16Offset ?? 0;
        const me = mk.range?.end?.utf16Offset ?? 0;
        if (pos > ms && pos < me) return true;
        if (selStart.value !== selEnd.value) {
            const s = displayOffsetToPureOffset(editorTextRef.value, Math.min(selStart.value, selEnd.value));
            const e = displayOffsetToPureOffset(editorTextRef.value, Math.max(selStart.value, selEnd.value));
            if (s < me && e > ms) return true;
        }
    }
    return false;
});

const cursorOnInsert = computed(() => {
    const pos = selStart.value;
    return getTokenRanges(editorTextRef.value || "").some((range) => pos >= range.start && pos <= range.end);
});

const cancelMarkAtCursor = () => {
    const meta = getFileMeta(props.filePath);
    const s = displayOffsetToPureOffset(editorTextRef.value, Math.min(selStart.value, selEnd.value));
    const e = displayOffsetToPureOffset(editorTextRef.value, Math.max(selStart.value, selEnd.value));
    const hasSel = s !== e;
    const nextMarks = [];

    for (const mk of meta.marks || []) {
        const ms = mk.range?.start?.utf16Offset ?? 0;
        const me = mk.range?.end?.utf16Offset ?? 0;
        if (!hasSel) {
            if (s > ms && s < me) {
                continue;
            }
            nextMarks.push(mk);
            continue;
        }

        if (e <= ms || s >= me) {
            nextMarks.push(mk);
            continue;
        }

        if (s > ms) {
            nextMarks.push({
                ...mk,
                id: generateId(),
                range: {
                    start: buildTextPosition(ms),
                    end: buildTextPosition(s),
                },
            });
        }

        if (e < me) {
            nextMarks.push({
                ...mk,
                id: generateId(),
                range: {
                    start: buildTextPosition(e),
                    end: buildTextPosition(me),
                },
            });
        }
    }

    meta.marks = nextMarks;
    saveTxtMeta();
};

const removeInsertAtCursor = () => {
    const insert = findInsertNearCursor();
    if (!insert) return;
    removeInsert(props.filePath, insert.id);
    rebuildEditorText(contentRef.value);
};

const emitMakeLink = () => {
    const ta = editorTextarea.value;
    const start = Math.min(selStart.value, selEnd.value);
    const end = Math.max(selStart.value, selEnd.value);
    const selectedText = ta && start !== end ? ta.value.slice(start, end) : "";
    window.dispatchEvent(new CustomEvent("simple-write:open-make-link", {
        detail: {
            filePath: props.filePath,
            targetText: selectedText || props.filePath.split("/").pop() || "",
            selectedText,
        },
    }));
};

// ---- 右键菜单 ----
const handleMenuClick = ({ key }) => {
    if (key === "txt-insert") { openInsertModal(); }
    else if (key === "txt-mark") { markModalOpen.value = true; }
    else if (key === "txt-unmark") { cancelMarkAtCursor(); }
    else if (key === "txt-make-link") { emitMakeLink(); }
    else if (key === "txt-cut") { document.execCommand("cut"); syncValue(); }
    else if (key === "txt-copy") { document.execCommand("copy"); }
    else if (key === "txt-paste") {
        navigator.clipboard?.readText?.().then(t => {
            if (t) { document.execCommand("insertText", false, t); syncValue(); }
        }).catch(() => {});
    }
    else if (key === "txt-selectAll") {
        const ta = editorTextarea.value;
        if (ta) { ta.focus(); ta.select(); onMouseUp(); }
    }
};

// ---- 滚动同步 ----
const syncMirrorScroll = () => {
    const wrap = editorWrap.value;
    const mirror = mirrorLayer.value;
    const ta = editorTextarea.value;
    if (!wrap || !mirror || !ta) return;
    mirror.scrollTop = ta.scrollTop;
    mirror.scrollLeft = ta.scrollLeft;
};

rebuildEditorText(contentRef.value);

const getPlainContent = () => contentRef.value || "";

defineExpose({ editorTextarea, openInsertModal, getPlainContent });
</script>

<style scoped>
.txt-edit-view { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 8px; }

.txt-edit-view__wrap {
    position: relative; flex: 1; min-height: 240px; overflow: hidden;
    border: 1px solid var(--border-color, #d9d9d9); border-radius: 8px;
    background: var(--bg-base, #fff);
}

.txt-edit-view__mirror {
    position: absolute; top: 0; left: 0; width: 100%; height: 100%;
    color: var(--text-primary, #262626);
    pointer-events: none; z-index: 1; box-sizing: border-box;
    font-family: Consolas, "SFMono-Regular", Menlo, Monaco, "Liberation Mono", monospace;
    font-size: var(--edit-font-size, 14px); line-height: 1.6; tab-size: 4;
    padding: 16px; overflow: hidden;
    white-space: pre-wrap; word-break: break-word; overflow-wrap: break-word;
}

.txt-edit-view__textarea {
    position: relative; width: 100%; height: 100%;
    border: none; outline: none; resize: none;
    background: transparent; caret-color: var(--text-primary, #262626);
    z-index: 2; box-sizing: border-box;
    font-family: Consolas, "SFMono-Regular", Menlo, Monaco, "Liberation Mono", monospace;
    font-size: var(--edit-font-size, 14px); line-height: 1.6; tab-size: 4;
    padding: 16px; overflow: auto;
    white-space: pre-wrap; word-break: break-word; overflow-wrap: break-word;
    color: transparent;
}
.txt-edit-view__textarea::selection { background: rgba(24, 144, 255, 0.3); }

.txt-edit-view__status {
    display: flex; align-items: center; justify-content: space-between;
    min-height: 20px; font-size: 12px; color: var(--text-tertiary, #8c8c8c);
}
.txt-edit-view__char-count { color: var(--text-disabled, #bfbfbf); }
.txt-edit-view__status--dirty { color: #d46b08; }

/* 插入弹窗 */
.ins-dot { display: inline-block; width: 10px; height: 10px; border-radius: 50%; margin-right: 6px; vertical-align: middle; }

.mark-type-item {
    padding: 10px 12px; cursor: pointer; border-radius: 6px;
    display: flex; align-items: center; gap: 10px; transition: background 0.15s;
}
.mark-type-item:hover { background: var(--bg-tertiary, #f5f5f5); }
.mark-swatch { display: inline-flex; align-items: center; justify-content: center; width: 36px; height: 22px; border-radius: 4px; font-size: 11px; flex-shrink: 0; }
.mark-type-item__name { color: var(--text-primary, #262626); }
</style>

<style>
.txtev-ins-inline {
    display: inline;
    margin: 0 2px;
    font-size: 0.92em;
    font-weight: 600;
    vertical-align: baseline;
}
</style>
