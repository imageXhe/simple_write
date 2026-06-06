<template>
    <a-dropdown :trigger="['contextmenu']" placement="bottomLeft">
        <div class="edit-view__wrap">
            <textarea
                ref="editorTextarea"
                :value="contentRef"
                class="edit-view__editor"
                spellcheck="false"
                :disabled="disabled"
                @input="handleInput"
                @click="checkLinkContext"
                @keyup="handleTextareaKeyup"
                @contextmenu="updateCurrentParagraphType"
            />
            <div
                v-if="showFilePicker"
                class="file-picker-overlay"
                :style="filePickerStyle"
                @click.stop
            >
                <div class="file-picker__title">{{ t("content.chooseLinkFile") }}</div>
                <div v-if="linkableFiles.length === 0" class="file-picker__empty">{{ t("content.noFiles") }}</div>
                <div
                    v-for="file in linkableFiles"
                    :key="file.path"
                    class="file-picker__item"
                    @click="handleFilePick(file)"
                >
                    <span class="file-picker__item-name">{{ file.isFolder ? '📁 ' : '' }}{{ file.name }}</span>
                    <span v-if="file.dir" class="file-picker__item-dir">{{ file.dir }}</span>
                </div>
            </div>
            <div class="edit-view__status" :class="{ 'edit-view__status--dirty': isDirty }">
                <span class="edit-view__char-count">{{ contentRef.length }} {{ t("content.charCount") }}</span>
                <span v-if="isSaving">{{ t("content.saving") }}</span>
                <span v-else-if="isDirty">{{ t("content.notSaved") }}</span>
                <span v-else>{{ t("content.saved") }}</span>
            </div>
        </div>

        <template #overlay>
                <a-menu class="dropdown-menu-bordered" @click="handleMenuClick">
                    <a-menu-item key="edit-newLink" :icon="h(LinkOutlined)">{{ t("content.newLink") }}</a-menu-item>
                    <a-menu-item key="edit-newOutLinkp" :icon="h(ExportOutlined)">{{ t("content.newOutLink") }}</a-menu-item>
                    <a-menu-divider />
                <a-sub-menu key="edit-textFormat" :icon="h(FontColorsOutlined)" :title="t('content.textFormat')">
                    <a-menu-item key="bold" :icon="h(BoldOutlined)">{{ t("content.bold") }}</a-menu-item>
                    <a-menu-item key="italic" :icon="h(ItalicOutlined)">{{ t("content.italic") }}</a-menu-item>
                    <a-menu-item key="strickThrough" :icon="h(StrikethroughOutlined)">{{ t("content.strickThrough") }}</a-menu-item>
                    <a-menu-item key="highlight" :icon="h(HighlightOutlined)">{{ t("content.highLight") }}</a-menu-item>
                    <a-menu-divider />
                    <a-menu-item key="code" :icon="h(SvgIcon, { raw: svgIcons.code })">{{ t("content.code") }}</a-menu-item>
                    <a-menu-divider />
                    <a-menu-item key="clearFormatting" :icon="h(ClearOutlined)">{{ t("content.clearFormatting") }}</a-menu-item>
                </a-sub-menu>
                <a-sub-menu key="edit-paragraphSettings" :icon="h(SvgIcon, { raw: svgIcons.paragraph })" :title="t('content.paragraphSettings')">
                    <a-menu-item key="bulletList" :icon="h(UnorderedListOutlined)">
                        {{ t("content.bulletList") }}
                        <CheckOutlined v-if="currentParagraphType === 'bulletList'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-item key="numberedList" :icon="h(OrderedListOutlined)">
                        {{ t("content.numberedList") }}
                        <CheckOutlined v-if="currentParagraphType === 'numberedList'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-item key="tasklist" :icon="h(CheckSquareOutlined)">
                        {{ t("content.tasklist") }}
                        <CheckOutlined v-if="currentParagraphType === 'tasklist'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-divider />
                    <a-menu-item key="heading1" :icon="h(SvgIcon, { raw: svgIcons.Heading_H1 })">
                        {{ t("content.heading1") }}
                        <CheckOutlined v-if="currentParagraphType === 'heading1'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-item key="heading2" :icon="h(SvgIcon, { raw: svgIcons.Heading_H2 })">
                        {{ t("content.heading2") }}
                        <CheckOutlined v-if="currentParagraphType === 'heading2'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-item key="heading3" :icon="h(SvgIcon, { raw: svgIcons.Heading_H3 })">
                        {{ t("content.heading3") }}
                        <CheckOutlined v-if="currentParagraphType === 'heading3'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-item key="heading4" :icon="h(SvgIcon, { raw: svgIcons.Heading_H4 })">
                        {{ t("content.heading4") }}
                        <CheckOutlined v-if="currentParagraphType === 'heading4'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-item key="heading5" :icon="h(SvgIcon, { raw: svgIcons.Heading_H5 })">
                        {{ t("content.heading5") }}
                        <CheckOutlined v-if="currentParagraphType === 'heading5'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-item key="heading6" :icon="h(SvgIcon, { raw: svgIcons.Heading_H6 })">
                        {{ t("content.heading6") }}
                        <CheckOutlined v-if="currentParagraphType === 'heading6'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-item key="body" :icon="h(AlignLeftOutlined)">
                        {{ t("content.body") }}
                        <CheckOutlined v-if="currentParagraphType === 'body'" class="paragraph-check" />
                    </a-menu-item>
                    <a-menu-divider />
                    <a-menu-item key="quote" :icon="h(SvgIcon, { raw: svgIcons.quote })">
                        {{ t("content.quote") }}
                        <CheckOutlined v-if="currentParagraphType === 'quote'" class="paragraph-check" />
                    </a-menu-item>
                </a-sub-menu>
                <a-sub-menu key="edit-insert" :icon="h(SvgIcon, { raw: svgIcons.insert })" :title="t('content.insert')">
                    <a-menu-item key="footNote" :icon="h(SvgIcon, { raw: svgIcons.footnote })">{{ t("content.footNote") }}</a-menu-item>
                    <a-menu-item key="table" :icon="h(TableOutlined)">{{ t("content.table") }}</a-menu-item>
                    <a-menu-item key="callOut" :icon="h(SvgIcon, { raw: svgIcons.quote })">{{ t("content.callOut") }}</a-menu-item>
                    <a-menu-divider />
                    <a-menu-item key="horizontalRule" :icon="h(LineOutlined)">{{ t("content.horizontalRule") }}</a-menu-item>
                    <a-menu-item key="codeBlock" :icon="h(SvgIcon, { raw: svgIcons.codeblock })">{{ t("content.codeBlock") }}</a-menu-item>
                </a-sub-menu>
                <a-menu-divider />
                <a-menu-item key="edit-cut" :icon="h(ScissorOutlined)" :disabled="!hasSelection">{{ t("content.cut") }}</a-menu-item>
                <a-menu-item key="edit-copy" :icon="h(CopyOutlined)" :disabled="!hasSelection">{{ t("content.copy") }}</a-menu-item>
                <a-menu-item key="edit-paste" :icon="h(CarryOutOutlined)">{{ t("content.paste") }}</a-menu-item>
                <a-menu-item key="edit-selectAll" :icon="h(ExpandOutlined)">{{ t("content.selectAll") }}</a-menu-item>
            </a-menu>
        </template>
    </a-dropdown>
</template>

<script setup>
import { inject, ref, watch, h, computed, nextTick, onMounted, onUnmounted } from "vue";
import { message } from "ant-design-vue";
import { fileData, warehousePath, flattenTree } from "../stores/fileStore";
import {
    CopyOutlined, LinkOutlined, ScissorOutlined, ExportOutlined,
    CarryOutOutlined, FontColorsOutlined, AlignLeftOutlined,
    TableOutlined, ExpandOutlined, BoldOutlined, ItalicOutlined,
    StrikethroughOutlined, HighlightOutlined, ClearOutlined,
    OrderedListOutlined, CheckSquareOutlined, LineOutlined,
    CheckOutlined, UnorderedListOutlined,
} from "@ant-design/icons-vue";
import svgIcons from "../assets/icons";
import SvgIcon from "../components/SvgIcon.vue";
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

const emit = defineEmits(["save"]);

// ---- 注入父级状态 ----
const updateTab = inject("updateTab", () => {});
const activeTabId = inject("activeTabId", ref(""));

// ---- 编辑器引用 ----
const editorTextarea = ref(null);

// ---- 本地内容管理（关键：避免 Vue :value 绑定重设 DOM 破坏撤销栈） ----
const contentRef = ref(props.initialContent);

// 外部（如文件加载）同步到本地，内部变更（输入/格式化）比对后跳过
watch(
    () => props.initialContent,
    (val) => {
        const v = val ?? "";
        // 仅当 prop 值确实与本地不同时才更新（内部变更已通过 syncValue 同步了本地 ref）
        if (v !== contentRef.value) {
            contentRef.value = v;
        }
    },
);

// ---- 输入处理 ----
const handleInput = (event) => {
    contentRef.value = event.target.value;
    syncValue();
    checkLinkContext();
};

// 将 textarea 当前值同步到 tab 状态（直接调 updateTab，不通过 v-model emit 触发 DOM 重设）
const syncValue = () => {
    const textarea = editorTextarea.value;
    const tabId = activeTabId.value;
    if (!textarea || !tabId) return;
    const val = textarea.value;
    // 先同步本地 ref，确保后续 prop → watch 比对时值一致，跳过无用更新
    contentRef.value = val;
    updateTab(tabId, {
        draftContent: val,
        isDirty: val !== (props.originalContent ?? ""),
    });
};

// ---- 句子终止符 ----
const SENTENCE_TERMINATORS = new Set(['.', '!', '?', '。', '！', '？', '\n']);

// 获取光标所在的句子范围
const getSentenceAtCursor = (text, cursorPos) => {
    if (!text || cursorPos <= 0) return null;
    let start = cursorPos - 1;
    while (start >= 0 && !SENTENCE_TERMINATORS.has(text[start])) start--;
    start++;
    if (start >= cursorPos) {
        let prevEnd = cursorPos - 1;
        while (prevEnd >= 0 && /\s/.test(text[prevEnd]) && !SENTENCE_TERMINATORS.has(text[prevEnd])) prevEnd--;
        if (prevEnd < 0) return null;
        start = prevEnd;
        while (start >= 0 && !SENTENCE_TERMINATORS.has(text[start])) start--;
        start++;
    }
    let end = cursorPos;
    while (end < text.length && !SENTENCE_TERMINATORS.has(text[end])) end++;
    if (end < text.length && SENTENCE_TERMINATORS.has(text[end])) end++;
    while (start < end && /\s/.test(text[start])) start++;
    while (end > start && /\s/.test(text[end - 1])) end--;
    if (start >= end) return null;
    return { start, end, text: text.slice(start, end) };
};

// 获取格式化范围
const getFormatRange = () => {
    const textarea = editorTextarea.value;
    if (!textarea) return null;
    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    if (start !== end) return { start, end, text: textarea.value.slice(start, end), isSelection: true };
    const sentence = getSentenceAtCursor(textarea.value, start);
    if (!sentence) return null;
    return { ...sentence, isSelection: false };
};

// ---- 内联格式 ----
const INLINE_FORMAT_WRAPPERS = {
    bold: '**$1**',
    italic: '*$1*',
    strickThrough: '~~$1~~',
    highlight: '==$1==',
    code: '`$1`',
};

const applyInlineFormat = (formatKey) => {
    const range = getFormatRange();
    if (!range) return;
    const wrapper = INLINE_FORMAT_WRAPPERS[formatKey];
    if (!wrapper) return;
    const textarea = editorTextarea.value;
    const formatted = wrapper.replace('$1', range.text);
    textarea.focus();
    textarea.setSelectionRange(range.start, range.end);
    // execCommand('insertText') 会推入浏览器撤销栈，使操作可被 Ctrl+Z 撤销
    const ok = document.execCommand('insertText', false, formatted);
    if (!ok) {
        // 降级：极少数环境下 execCommand 可能失败
        textarea.setRangeText(formatted, range.start, range.end, 'end');
    }
    syncValue();
    const prefixLen = wrapper.indexOf('$1');
    textarea.setSelectionRange(range.start + prefixLen, range.start + prefixLen + range.text.length);
};

const applyClearFormatting = () => {
    const range = getFormatRange();
    if (!range) return;
    const textarea = editorTextarea.value;
    let cleaned = range.text;
    cleaned = cleaned.replace(/\*\*(.+?)\*\*/g, '$1');
    cleaned = cleaned.replace(/(?<!\*)\*(?!\*)(.+?)(?<!\*)\*(?!\*)/g, '$1');
    cleaned = cleaned.replace(/~~(.+?)~~/g, '$1');
    cleaned = cleaned.replace(/==(.+?)==/g, '$1');
    cleaned = cleaned.replace(/`(.+?)`/g, '$1');
    textarea.focus();
    textarea.setSelectionRange(range.start, range.end);
    const ok = document.execCommand('insertText', false, cleaned);
    if (!ok) {
        textarea.setRangeText(cleaned, range.start, range.end, 'end');
    }
    syncValue();
    textarea.setSelectionRange(range.start, range.start + cleaned.length);
};

// ---- 文件列表（从共享 store 展平） ----
const allFiles = computed(() => flattenTree(fileData.value, warehousePath.value));

// 按用户输入匹配文件名
const linkableFiles = computed(() => {
    let files = allFiles.value;
    const q = filePickerQuery.value.trim().toLowerCase();
    if (q) {
        files = files.filter((f) => f.name.toLowerCase().includes(q));
    }
    return files;
});

// ---- 段落格式 ----
const currentParagraphType = ref('body');

const PARAGRAPH_PREFIXES = {
    heading1: '# ', heading2: '## ', heading3: '### ',
    heading4: '#### ', heading5: '##### ', heading6: '###### ',
    bulletList: '- ', numberedList: '1. ', tasklist: '- [ ] ',
    quote: '> ', body: '',
};

const PARAGRAPH_PREFIX_RE = /^(#{1,6}\s|>\s|[-*]\s(?:\[[ x]\]\s)?|\d+\.\s)/;

const detectParagraphType = (line) => {
    const trimmed = line.trimStart();
    if (/^#{6}\s/.test(trimmed)) return 'heading6';
    if (/^#{5}\s/.test(trimmed)) return 'heading5';
    if (/^#{4}\s/.test(trimmed)) return 'heading4';
    if (/^#{3}\s/.test(trimmed)) return 'heading3';
    if (/^#{2}\s/.test(trimmed)) return 'heading2';
    if (/^#{1}\s/.test(trimmed)) return 'heading1';
    if (/^>\s/.test(trimmed)) return 'quote';
    if (/^[-*]\s\[[ x]\]\s/.test(trimmed)) return 'tasklist';
    if (/^\d+\.\s/.test(trimmed)) return 'numberedList';
    if (/^[-*]\s/.test(trimmed)) return 'bulletList';
    return 'body';
};

const getCurrentLine = () => {
    const textarea = editorTextarea.value;
    if (!textarea) return '';
    const text = textarea.value;
    const pos = textarea.selectionStart;
    const lineStart = text.lastIndexOf('\n', pos - 1) + 1;
    let lineEnd = text.indexOf('\n', pos);
    if (lineEnd === -1) lineEnd = text.length;
    return text.slice(lineStart, lineEnd);
};

const hasSelection = ref(false);

// ---- 内部链接文件选择器 ----
const showFilePicker = ref(false);
const linkInsertStart = ref(0); // [[]] 插入时的起始位置
const filePickerStyle = ref({ left: '0px', top: '0px' }); // 弹窗定位
const filePickerQuery = ref(''); // 用户在 [[]] 内输入的过滤文本

const updateCurrentParagraphType = () => {
    const textarea = editorTextarea.value;
    if (textarea) {
        hasSelection.value = textarea.selectionStart !== textarea.selectionEnd;
    }
    const line = getCurrentLine();
    currentParagraphType.value = detectParagraphType(line);
};

const applyParagraphFormat = (formatKey) => {
    const textarea = editorTextarea.value;
    if (!textarea) return;
    const text = textarea.value;
    const selStart = textarea.selectionStart;
    const selEnd = textarea.selectionEnd;
    const lineStart = text.lastIndexOf('\n', selStart - 1) + 1;
    let lineEnd = text.indexOf('\n', selEnd - 1);
    if (lineEnd === -1) lineEnd = text.length;
    if (text[selEnd - 1] === '\n') lineEnd = selEnd;
    const selectedText = text.slice(lineStart, lineEnd);
    const lines = selectedText.split('\n');
    const newPrefix = PARAGRAPH_PREFIXES[formatKey] ?? '';
    const transformedLines = lines.map((line, index) => {
        if (formatKey === 'numberedList') {
            const match = line.match(PARAGRAPH_PREFIX_RE);
            const content = match ? line.slice(match[0].length) : line;
            return `${index + 1}. ${content}`;
        }
        const match = line.match(PARAGRAPH_PREFIX_RE);
        if (match) {
            const content = line.slice(match[0].length);
            if (formatKey !== 'body' && detectParagraphType(line) === formatKey) return line;
            return newPrefix ? newPrefix + content : content;
        }
        return newPrefix ? newPrefix + line : line;
    });
    const newText = transformedLines.join('\n');
    textarea.focus();
    textarea.setSelectionRange(lineStart, lineEnd);
    const ok = document.execCommand('insertText', false, newText);
    if (!ok) {
        textarea.setRangeText(newText, lineStart, lineEnd, 'end');
    }
    syncValue();
    currentParagraphType.value = formatKey;
};

// ---- 插入功能 ----

// 获取脚注序号：扫描文档中已有的 [^N]，返回下一个可用编号
const getNextFootnoteIndex = (text) => {
    const matches = text.matchAll(/\[\^(\d+)\]/g);
    let max = 0;
    for (const m of matches) {
        const n = parseInt(m[1], 10);
        if (n > max) max = n;
    }
    return max + 1;
};

const applyInsert = (key) => {
    const textarea = editorTextarea.value;
    if (!textarea) return;

    const text = textarea.value;
    const selStart = textarea.selectionStart;
    const selEnd = textarea.selectionEnd;
    const hasSelection = selStart !== selEnd;
    const selectedText = text.slice(selStart, selEnd);

    let insertText = '';
    let cursorOffset = 0; // 插入后光标相对于 insertText 起始的位置

    switch (key) {
        case 'horizontalRule': {
            // 确保分割线在独立行上
            const needBefore = selStart > 0 && text[selStart - 1] !== '\n';
            const needAfter = selStart < text.length && text[selStart] !== '\n';
            insertText = `${needBefore ? '\n' : ''}---${needAfter ? '\n' : ''}`;
            // 光标放在分割线之后
            cursorOffset = insertText.length;
            break;
        }

        case 'codeBlock': {
            if (hasSelection) {
                // 选中文本的话去掉首尾空白换行包装
                const trimmed = selectedText.replace(/^\n+|\n+$/g, '');
                insertText = `\n\`\`\`\n${trimmed}\n\`\`\`\n`;
                cursorOffset = 5 + trimmed.length; // 放在代码块之后
            } else {
                insertText = `\n\`\`\`\n\n\`\`\`\n`;
                cursorOffset = 5; // 放在 ``` 之后的空行上
            }
            break;
        }

        case 'table': {
            insertText = `\n| 列1 | 列2 | 列3 |\n| --- | --- | --- |\n|     |     |     |\n`;
            cursorOffset = 3; // 光标放在"列1"位置
            break;
        }

        case 'callOut': {
            insertText = `\n> **${t('content.callOut')}**\n> \n`;
            cursorOffset = insertText.length - 1; // 光标放在第二行 > 后面
            break;
        }

        case 'footNote': {
            const index = getNextFootnoteIndex(text);
            // 在文档末尾追加脚注定义
            const fnDef = text.length > 0 && text[text.length - 1] !== '\n'
                ? `\n[^${index}]: `
                : `[^${index}]: `;
            const marker = `[^${index}]`;
            insertText = marker;
            // 需要同时在末尾添加定义，用两次 setRangeText 不太好，
            // 改为整体替换：选中位置插入标记 + 末尾追加定义
            const before = text.slice(0, selStart);
            const after = text.slice(selEnd);
            let ok = document.execCommand('insertText', false, marker);
            if (!ok) { textarea.setRangeText(marker, selStart, selEnd, 'end'); }
            // 在末尾追加脚注定义
            const endPos = textarea.value.length;
            textarea.setSelectionRange(endPos, endPos);
            ok = document.execCommand('insertText', false, fnDef);
            if (!ok) { textarea.setRangeText(fnDef, endPos, endPos, 'end'); }
            syncValue();
            // 光标回到标记之后
            textarea.setSelectionRange(selStart + marker.length, selStart + marker.length);
            return;
        }

        default:
            return;
    }

    textarea.focus();
    textarea.setSelectionRange(selStart, selEnd);
    const ok = document.execCommand('insertText', false, insertText);
    if (!ok) {
        textarea.setRangeText(insertText, selStart, selEnd, 'end');
    }
    syncValue();

    // 调整光标位置
    const newPos = selStart + cursorOffset;
    textarea.setSelectionRange(newPos, newPos);
};

// ---- 插入链接 ----
const applyExternalLink = () => {
    const textarea = editorTextarea.value;
    if (!textarea) return;

    const selStart = textarea.selectionStart;
    const selEnd = textarea.selectionEnd;
    const hasSelection = selStart !== selEnd;

    textarea.focus();
    textarea.setSelectionRange(selStart, selEnd);

    if (hasSelection) {
        const selectedText = textarea.value.slice(selStart, selEnd);
        const linkText = `[${selectedText}]()`;
        const ok = document.execCommand('insertText', false, linkText);
        if (!ok) { textarea.setRangeText(linkText, selStart, selEnd, 'end'); }
        // 光标放在 () 中间
        textarea.setSelectionRange(selStart + linkText.length - 1, selStart + linkText.length - 1);
    } else {
        const linkText = '[]()';
        const ok = document.execCommand('insertText', false, linkText);
        if (!ok) { textarea.setRangeText(linkText, selStart, selEnd, 'end'); }
        // 光标放在方括号中间
        textarea.setSelectionRange(selStart + 1, selStart + 1);
    }
    syncValue();
};

const applyInternalLink = () => {
    const textarea = editorTextarea.value;
    if (!textarea) return;

    const selStart = textarea.selectionStart;
    const selEnd = textarea.selectionEnd;
    const hasSelection = selStart !== selEnd;

    textarea.focus();
    textarea.setSelectionRange(selStart, selEnd);

    const linkText = hasSelection
        ? `[[${textarea.value.slice(selStart, selEnd)}]]`
        : `[[]]`;
    const ok = document.execCommand('insertText', false, linkText);
    if (!ok) { textarea.setRangeText(linkText, selStart, selEnd, 'end'); }

    // 记录插入位置，光标放在 [[]] 中间
    linkInsertStart.value = selStart;
    const cursorPos = hasSelection ? selStart + linkText.length : selStart + 2;
    textarea.setSelectionRange(cursorPos, cursorPos);
    syncValue();

    // 将弹窗定位到 ]] 下方（空间不足时翻转到上方），然后显示
    nextTick(() => {
        const closePos = textarea.value.indexOf(']]', linkInsertStart.value);
        if (closePos !== -1) {
            const pos = getCaretPixelPos(textarea, closePos + 1);
            const h = 200;
            const spaceBelow = window.innerHeight - pos.bottom - 8;
            const top = spaceBelow >= h ? pos.bottom + 4 : pos.top - h - 4;
            filePickerStyle.value = constrainPickerPos(pos.left, top);
        }
        showFilePicker.value = true;
    });
};

const handleFilePick = (file) => {
    const textarea = editorTextarea.value;
    if (!textarea) return;

    // 用 getLinkContext 准确获取当前 [[]] 范围
    const ctx = getLinkContext(textarea);
    if (!ctx) { showFilePicker.value = false; return; }

    textarea.focus();
    // 未闭合的 [[：替换光标到 ]] 不存在的位置，并自动补全 ]]
    const replaceEnd = ctx.end !== -1 ? ctx.end : textarea.selectionStart;
    const insertText = ctx.end !== -1 ? file.name : file.name + ']]';
    textarea.setSelectionRange(ctx.start + 2, replaceEnd);
    const ok = document.execCommand('insertText', false, insertText);
    if (!ok) { textarea.setRangeText(insertText, ctx.start + 2, replaceEnd, 'end'); }
    syncValue();

    // 光标放到 ]] 之后
    const newEnd = ctx.start + 2 + file.name.length + 2;
    textarea.setSelectionRange(newEnd, newEnd);

    showFilePicker.value = false;
    filePickerQuery.value = '';
};

const closeFilePicker = () => {
    showFilePicker.value = false;
    filePickerQuery.value = '';
};

// 将弹窗位置约束在视口内
const constrainPickerPos = (left, top, w = 280, h = 200) => {
    const m = 8;
    if (left + w > window.innerWidth - m) left = window.innerWidth - w - m;
    if (left < m) left = m;
    if (top + h > window.innerHeight - m) top = window.innerHeight - h - m;
    if (top < m) top = m;
    return { left: left + 'px', top: top + 'px' };
};

// 检测光标是否在 [[链接]] 内（支持未闭合的 [[），返回 { start, end, query } 或 null
const getLinkContext = (textarea) => {
    const pos = textarea.selectionStart;
    const text = textarea.value;
    if (pos !== textarea.selectionEnd || pos <= 1) return null;
    const openPos = text.lastIndexOf('[[', pos - 1);
    if (openPos === -1 || pos <= openPos + 1) return null;
    // 如果 [[ 和光标之间有 ]]，说明该链接已闭合且光标在其后，不在链接内
    const closeBeforeCursor = text.indexOf(']]', openPos + 2);
    if (closeBeforeCursor !== -1 && closeBeforeCursor < pos) return null;
    // 仅在本行内查找 ]]，避免跨行匹配到文档末尾的 ]]
    const lineEnd = text.indexOf('\n', pos);
    const searchEnd = lineEnd !== -1 ? lineEnd : Math.min(text.length, openPos + 120);
    const closeAfter = text.indexOf(']]', pos);
    const hasClose = closeAfter !== -1 && closeAfter <= searchEnd;
    return {
        start: openPos,
        end: hasClose ? closeAfter : -1,
        query: text.slice(openPos + 2, pos),
    };
};

// textarea keyup 处理：Escape 关闭弹窗，其他键检查链接上下文
const handleTextareaKeyup = (e) => {
    if (e.key === 'Escape') {
        showFilePicker.value = false;
        filePickerQuery.value = '';
        return;
    }
    checkLinkContext();
};

// 检查链接上下文并显示/隐藏文件弹窗
const checkLinkContext = () => {
    const textarea = editorTextarea.value;
    if (!textarea) return;
    const ctx = getLinkContext(textarea);
    if (ctx) {
        linkInsertStart.value = ctx.start;
        filePickerQuery.value = ctx.query;
        const pos = getCaretPixelPos(textarea, textarea.selectionStart);
        const h = 200;
        const spaceBelow = window.innerHeight - pos.bottom - 8;
        const top = spaceBelow >= h ? pos.bottom + 4 : pos.top - h - 4;
        filePickerStyle.value = constrainPickerPos(pos.left, top);
        showFilePicker.value = true;
    } else {
        showFilePicker.value = false;
        filePickerQuery.value = '';
    }
};

// 点击弹窗外关闭
const handleOutsideMouseDown = (e) => {
    if (!showFilePicker.value) return;
    const pickerEl = document.querySelector('.file-picker-overlay');
    if (pickerEl && !pickerEl.contains(e.target)) {
        showFilePicker.value = false;
    }
};

onMounted(() => {
    document.addEventListener('mousedown', handleOutsideMouseDown, true);
});

onUnmounted(() => {
    document.removeEventListener('mousedown', handleOutsideMouseDown, true);
});

// 计算 textarea 中指定字符位置的像素坐标（视口坐标）
const getCaretPixelPos = (textarea, pos) => {
    const textareaRect = textarea.getBoundingClientRect();
    const computed = window.getComputedStyle(textarea);
    const div = document.createElement('div');
    const props = [
        'boxSizing', 'width', 'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
        'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth',
        'fontSize', 'fontFamily', 'fontWeight', 'fontStyle', 'lineHeight', 'letterSpacing',
        'whiteSpace', 'tabSize', 'textIndent',
    ];
    for (const p of props) div.style[p] = computed[p];
    // 关键：固定定位到 textarea 的视口位置，确保 mirror div 与 textarea 坐标对齐
    div.style.position = 'fixed';
    div.style.visibility = 'hidden';
    div.style.top = textareaRect.top + 'px';
    div.style.left = textareaRect.left + 'px';
    div.textContent = textarea.value.substring(0, pos);
    const marker = document.createElement('span');
    marker.textContent = textarea.value.charAt(pos) || '.';
    div.appendChild(marker);
    document.body.appendChild(div);
    const mr = marker.getBoundingClientRect();
    document.body.removeChild(div);
    return { left: mr.left, top: mr.top, bottom: mr.bottom };
};

// ---- 剪贴板 ----
const handleClipboard = (key) => {
    const textarea = editorTextarea.value;
    if (!textarea) return;

    const selStart = textarea.selectionStart;
    const selEnd = textarea.selectionEnd;

    if (key === "edit-cut") {
        if (selStart === selEnd) return; // 无选中内容则忽略
        textarea.focus();
        textarea.setSelectionRange(selStart, selEnd);
        document.execCommand("cut");
        syncValue();
        return;
    }

    if (key === "edit-copy") {
        if (selStart === selEnd) return; // 无选中内容则忽略
        textarea.focus();
        textarea.setSelectionRange(selStart, selEnd);
        document.execCommand("copy");
        return;
    }

    if (key === "edit-paste") {
        textarea.focus();
        // 将光标/选区恢复到右键时的位置
        textarea.setSelectionRange(selStart, selEnd);
        // execCommand("paste") 在部分环境下会失败，降级到 Clipboard API
        const ok = document.execCommand("paste");
        if (ok) {
            syncValue();
        } else {
            navigator.clipboard?.readText?.()
                .then((text) => {
                    if (text != null) {
                        textarea.setSelectionRange(selStart, selEnd);
                        const ok = document.execCommand('insertText', false, text);
                        if (!ok) {
                            textarea.setRangeText(text, selStart, selEnd, "end");
                        }
                        syncValue();
                    }
                })
                .catch(() => {}); // 权限不足时静默失败，用户可用 Ctrl+V
        }
        return;
    }
};

// ---- 菜单点击 ----
const handleMenuClick = ({ key }) => {
    if (!key) return;
    // 文本格式
    if (key === "bold" || key === "italic" || key === "strickThrough" || key === "highlight" || key === "code") {
        applyInlineFormat(key);
        return;
    }
    if (key === "clearFormatting") {
        applyClearFormatting();
        return;
    }
    // 段落设置
    if (
        key === "bulletList" || key === "numberedList" || key === "tasklist" ||
        key === "heading1" || key === "heading2" || key === "heading3" ||
        key === "heading4" || key === "heading5" || key === "heading6" ||
        key === "body" || key === "quote"
    ) {
        applyParagraphFormat(key);
        return;
    }
    // 插入
    if (
        key === "footNote" || key === "table" || key === "callOut" ||
        key === "horizontalRule" || key === "codeBlock"
    ) {
        applyInsert(key);
        return;
    }
    // 剪贴板
    if (key === "edit-cut" || key === "edit-copy" || key === "edit-paste") {
        handleClipboard(key);
        return;
    }
    if (key === "edit-selectAll") {
        const textarea = editorTextarea.value;
        if (textarea) { textarea.focus(); textarea.select(); }
        return;
    }
    // 链接
    if (key === "edit-newLink") {
        applyInternalLink();
        return;
    }
    if (key === "edit-newOutLinkp") {
        applyExternalLink();
        return;
    }
    message.info("功能待定");
};

// 暴露 textarea 引用，供父组件在 Ctrl+S 时读取值
defineExpose({ editorTextarea });
</script>

<style scoped>
.edit-view__wrap {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.edit-view__editor {
    flex: 1;
    min-height: 240px;
    width: 100%;
    box-sizing: border-box;
    padding: 16px;
    border: 1px solid var(--border-color, #d9d9d9);
    border-radius: 8px;
    background: var(--bg-base, #fff);
    color: var(--text-primary, #262626);
    resize: none;
    outline: none;
    font-size: var(--edit-font-size, 14px);
    line-height: 1.6;
    font-family: Consolas, "SFMono-Regular", Menlo, Monaco, "Liberation Mono", monospace;
    white-space: pre-wrap;
    word-break: break-word;
    overflow-wrap: break-word;
    tab-size: 4;
}

.edit-view__editor:focus {
    border-color: #1890ff;
    box-shadow: 0 0 0 2px rgba(24, 144, 255, 0.16);
}

.edit-view__editor:disabled {
    background: var(--bg-secondary, #fafafa);
    color: var(--text-tertiary, #8c8c8c);
    cursor: not-allowed;
}

.edit-view__status {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 20px;
    font-size: 12px;
    color: var(--text-tertiary, #8c8c8c);
}

.edit-view__char-count {
    color: var(--text-disabled, #bfbfbf);
}

.edit-view__status--dirty {
    color: #d46b08;
}

</style>

<style>
/* 段落格式菜单中的勾选标记（dropdown 渲染在 body 层，必须全局样式） */
.paragraph-check {
    margin-left: auto;
    color: #1890ff;
    font-size: 12px;
}

/* 文件选择弹窗（固定定位在页面中，必须全局样式） */
.file-picker-overlay {
    position: fixed;
    z-index: 1050;
    background: var(--bg-elevated, #fff);
    border-radius: 8px;
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08), 0 3px 6px rgba(0, 0, 0, 0.12);
    padding: 8px 12px;
    max-height: 180px;
    overflow-y: auto;
    min-width: 280px;
}

.file-picker__title {
    padding: 4px 0 8px;
    font-size: 12px;
    color: var(--text-tertiary, #8c8c8c);
    border-bottom: 1px solid var(--border-secondary, #f0f0f0);
    margin-bottom: 4px;
}

.file-picker__empty {
    padding: 16px 0;
    text-align: center;
    color: #bfbfbf;
    font-size: 13px;
}

.file-picker__item {
    padding: 4px 8px;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s;
}

.file-picker__item-name {
    font-size: 13px;
    color: var(--text-primary, #262626);
}

.file-picker__item-dir {
    display: block;
    font-size: 11px;
    color: var(--text-disabled, #bfbfbf);
    margin-top: 1px;
}

.file-picker__item:hover {
    background: #e6f7ff;
}

.file-picker__item:hover .file-picker__item-name {
    color: #1890ff;
}
</style>
