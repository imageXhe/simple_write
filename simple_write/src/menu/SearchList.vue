<template>
  <div class="search-container">
    <!-- 搜索历史 popover -->
    <a-popover
      v-model:open="historyVisible"
      :trigger="[]"
      placement="bottomLeft"
      overlayClassName="search-history-popover"
    >
      <template #content>
        <div class="history-panel">
          <div class="history-header">
            <span class="history-title">{{ t('file.searchHistory') }}</span>
            <a-button type="text" size="small" @click="clearHistory" class="clear-btn">
              <CloseOutlined />
            </a-button>
          </div>
          <div v-if="searchHistory.length > 0" class="history-list">
            <div
              v-for="item in searchHistory"
              :key="item"
              class="history-item"
              @mousedown.prevent="handleHistoryClick(item)"
            >
              {{ item }}
            </div>
          </div>
        </div>
      </template>

      <!-- 搜索栏 -->
      <div class="search-row">
        <a-input
          :placeholder="t('file.search')"
          v-model:value="searchText"
          @pressEnter="handleSearch"
          @focus="onInputFocus"
          @blur="onInputBlur"
          class="search-input"
          allow-clear
        />
        <a-dropdown v-model:open="dropdownOpen" :trigger="['click']">
          <a-tooltip :title="t('file.searchSettings')">
            <a-button type="text" class="setting-btn">
              <SettingOutlined />
            </a-button>
          </a-tooltip>
          <template #overlay>
            <div class="dropdown-panel">
              <div class="option-item">
                <span class="option-label">{{ t('file.searchCaseSensitive') }}</span>
                <a-switch v-model:checked="searchOptions.caseSensitive" size="small" />
              </div>
              <div class="option-item">
                <span class="option-label">{{ t('file.searchWholeWord') }}</span>
                <a-switch v-model:checked="searchOptions.wholeWord" size="small" />
              </div>
              <div class="option-item">
                <span class="option-label">{{ t('file.searchUseRegex') }}</span>
                <a-switch v-model:checked="searchOptions.useRegex" size="small" />
              </div>
              <div class="option-item option-select">
                <span class="option-label">{{ t('file.searchScope') }}</span>
                <a-select
                  v-model:value="searchOptions.searchScope"
                  size="small"
                  class="scope-select"
                >
                  <a-select-option value="content">{{ t('file.searchScopeContent') }}</a-select-option>
                  <a-select-option value="path">{{ t('file.searchScopePath') }}</a-select-option>
                  <a-select-option value="file">{{ t('file.searchScopeFile') }}</a-select-option>
                </a-select>
              </div>
            </div>
          </template>
        </a-dropdown>
      </div>
    </a-popover>

    <!-- 搜索结果统计 + 排序 -->
    <div v-if="hasSearched" class="search-stats">
      <span class="result-count">{{ resultCountText }}</span>
      <a-dropdown :trigger="['click']" :disabled="sortedResults.length === 0">
        <a-tooltip :title="t('file.sort')">
          <a-button
            type="text"
            size="small"
            :disabled="sortedResults.length === 0"
            class="sort-btn"
          >
            <SortDescendingOutlined />
          </a-button>
        </a-tooltip>
        <template #overlay>
          <a-menu
            v-model:selectedKeys="selectedSortKeys"
            @click="handleSortChange"
          >
            <a-menu-item key="name_asc">{{ t('file.nameAsc') }}</a-menu-item>
            <a-menu-item key="name_desc">{{ t('file.nameDesc') }}</a-menu-item>
            <a-menu-item key="modify_time_asc">{{ t('file.modifyTimeAsc') }}</a-menu-item>
            <a-menu-item key="modify_time_desc">{{ t('file.modifyTimeDesc') }}</a-menu-item>
            <a-menu-item key="create_time_asc">{{ t('file.createTimeAsc') }}</a-menu-item>
            <a-menu-item key="create_time_desc">{{ t('file.createTimeDesc') }}</a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>
    </div>

    <!-- 搜索结果列表 -->
    <div class="search-results">
      <!-- 搜索中 -->
      <div v-if="searching" class="search-loading">
        <a-spin />
      </div>

      <!-- 无结果 -->
      <a-empty
        v-else-if="hasSearched && sortedResults.length === 0"
        :description="t('file.noSearchResult')"
        :image="simpleImage"
      />

      <!-- 折叠面板结果 -->
      <a-collapse
        v-else-if="sortedResults.length > 0"
        v-model:activeKey="activeKeys"
        :bordered="false"
        class="result-collapse"
      >
        <a-collapse-panel
          v-for="item in sortedResults"
          :key="item.path"
          :collapsible="searchOptions.searchScope === 'content' ? 'header' : 'disabled'"
          :showArrow="searchOptions.searchScope === 'content'"
        >
          <template #header>
            <div class="result-header-row">
              <span class="result-header-text">{{ item.name }}</span>
              <a-button type="text" size="small" class="locate-btn" @click.stop="navigateToFile(item)">
                <AimOutlined />
              </a-button>
            </div>
          </template>
          <div v-if="item.matches && item.matches.length > 0" class="match-list">
            <div
              v-for="(match, idx) in item.matches"
              :key="idx"
              class="match-item"
              @click.stop="handleMatchClick(item, match)"
            >
              <div class="match-line match-context">{{ match.beforeText }}</div>
              <div class="match-line">
                <span>{{ match.matchText.slice(0, match.matchStart) }}</span>
                <span class="match-highlight">{{ match.matchText.slice(match.matchStart, match.matchEnd) }}</span>
                <span>{{ match.matchText.slice(match.matchEnd) }}</span>
              </div>
              <div class="match-line match-context">{{ match.afterText }}</div>
            </div>
          </div>
        </a-collapse-panel>
      </a-collapse>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive, watch, computed, onMounted, inject, nextTick } from 'vue'
import { Empty } from "ant-design-vue"
import { SettingOutlined, CloseOutlined, SortDescendingOutlined, AimOutlined } from "@ant-design/icons-vue"
import { invoke } from "@tauri-apps/api/core"
import { useI18n } from "../locales"
import { fileData, warehousePath } from "../stores/fileStore"

const { t } = useI18n()
const simpleImage = Empty.PRESENTED_IMAGE_SIMPLE
const openFile = inject("openFile", null)

const searchText = ref('')
const searchResults = ref([])
const searching = ref(false)
const hasSearched = ref(false)

// 搜索选项
const searchOptions = reactive({
  caseSensitive: false,
  wholeWord: false,
  useRegex: false,
  searchScope: 'content',
})

// 搜索历史
const HISTORY_KEY = 'search_history'
const MAX_HISTORY = 10
const searchHistory = ref([])

onMounted(() => {
  try {
    const stored = localStorage.getItem(HISTORY_KEY)
    if (stored) {
      searchHistory.value = JSON.parse(stored)
    }
  } catch {
    // 数据解析失败则忽略
  }
})

const saveHistory = () => {
  localStorage.setItem(HISTORY_KEY, JSON.stringify(searchHistory.value))
}

const addToHistory = (keyword) => {
  if (!keyword.trim()) return
  const idx = searchHistory.value.indexOf(keyword)
  if (idx > -1) {
    searchHistory.value.splice(idx, 1)
  }
  searchHistory.value.unshift(keyword)
  if (searchHistory.value.length > MAX_HISTORY) {
    searchHistory.value.length = MAX_HISTORY
  }
  saveHistory()
}

// UI 控制
const dropdownOpen = ref(false)
const historyVisible = ref(false)
const activeKeys = ref([])

// 搜索框清空时清除结果
watch(searchText, (val) => {
  if (!val) {
    searchResults.value = []
    hasSearched.value = false
    activeKeys.value = []
  }
})

// 切换搜索范围时清除结果
watch(() => searchOptions.searchScope, () => {
  searchResults.value = []
  hasSearched.value = false
  activeKeys.value = []
})

const onInputFocus = () => {
  if (searchHistory.value.length > 0) {
    historyVisible.value = true
  }
}

const onInputBlur = () => {
  setTimeout(() => {
    historyVisible.value = false
  }, 200)
}

// ==================== 文件树扁平化（含时间字段） ====================
const flattenWithTime = (nodes, basePath) => {
  const result = []
  const walk = (list, segs) => {
    for (const node of list) {
      if (!node || !node.info) continue
      const fullSegs = [...segs, node.info.name]
      const dir = segs.length > 0 ? segs.join('/') + '/' : ''
      result.push({
        name: node.info.name,
        path: [basePath, ...fullSegs].filter(Boolean).join('/').replace(/\\/g, '/'),
        dir,
        isFolder: node.is_folder || false,
        modifyTime: node.info.modify_time || '',
        createTime: node.info.create_time || '',
      })
      if (node.children?.length) {
        walk(node.children, fullSegs)
      }
    }
  }
  walk(nodes, [])
  return result
}

// ==================== 正则构建 ====================
const escapeRegex = (str) => str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')

const buildTestRegex = (keyword, options) => {
  try {
    let pattern = options.useRegex ? keyword : escapeRegex(keyword)
    if (options.wholeWord && !options.useRegex) {
      pattern = '\\b' + pattern + '\\b'
    }
    return new RegExp(pattern, options.caseSensitive ? '' : 'i')
  } catch {
    return null
  }
}

const buildGlobalRegex = (keyword, options) => {
  try {
    let pattern = options.useRegex ? keyword : escapeRegex(keyword)
    if (options.wholeWord && !options.useRegex) {
      pattern = '\\b' + pattern + '\\b'
    }
    return new RegExp(pattern, options.caseSensitive ? 'g' : 'gi')
  } catch {
    return null
  }
}

// ==================== 上下文提取（行级：匹配行上方一行 + 匹配行 + 下方一行，不足时另一侧补） ====================
const extractContext = (text, matchStart, matchLength, matchIndex) => {
  const lines = text.split('\n')

  // 定位匹配所在行号
  let lineIdx = -1
  let charCount = 0
  for (let i = 0; i < lines.length; i++) {
    if (charCount + lines[i].length >= matchStart) {
      lineIdx = i
      break
    }
    charCount += lines[i].length + 1 // +1 for \n
  }
  if (lineIdx === -1) lineIdx = lines.length - 1

  const matchLineStart = charCount
  const matchInLineStart = matchStart - matchLineStart
  const matchInLineEnd = matchInLineStart + matchLength

  const matchText = lines[lineIdx] || ''

  // 提取上下行（不足时向另一侧补齐）
  const hasBefore = lineIdx > 0
  const hasAfter = lineIdx < lines.length - 1

  let beforeText = ''
  let afterText = ''

  if (hasBefore && hasAfter) {
    beforeText = lines[lineIdx - 1]
    afterText = lines[lineIdx + 1]
  } else if (!hasBefore && hasAfter) {
    afterText = lines[lineIdx + 1]
    if (lineIdx + 2 < lines.length) {
      afterText += '\n' + lines[lineIdx + 2]
    }
  } else if (hasBefore && !hasAfter) {
    if (lineIdx - 2 >= 0) {
      beforeText = lines[lineIdx - 2] + '\n'
    }
    beforeText += lines[lineIdx - 1]
  }

  return {
    beforeText,
    matchText,
    afterText,
    matchStart: matchInLineStart,
    matchEnd: matchInLineEnd,
    matchIndex,
    startPos: matchStart,  // 匹配在文件中的绝对位置，供 ReadView 等比滚动
  }
}

// ==================== 三种搜索模式 ====================
const searchByPath = (list, matcher) => {
  if (!matcher) return []
  return list.filter(item => matcher.test(item.path))
}

const searchByFileName = (list, matcher) => {
  if (!matcher) return []
  return list.filter(item => !item.isFolder && matcher.test(item.name))
}

const searchByContent = async (list, matcher) => {
  if (!matcher) return []
  const files = list.filter(item => !item.isFolder)
  const results = []

  for (const file of files) {
    matcher.lastIndex = 0
    let content
    try {
      content = await invoke('get_file_content', { filePath: file.path })
    } catch {
      continue
    }
    if (!content) continue

    let matchIdx = 0
    const matches = []
    let match
    while ((match = matcher.exec(content)) !== null) {
      matches.push(extractContext(content, match.index, match[0].length, matchIdx))
      matchIdx++
      // 防止空匹配死循环
      if (match[0].length === 0) {
        matcher.lastIndex++
      }
    }

    if (matches.length > 0) {
      results.push({ ...file, matches })
    }
  }

  return results
}

// ==================== 搜索主函数 ====================
const handleSearch = async () => {
  historyVisible.value = false
  const keyword = searchText.value.trim()
  if (!keyword) return

  addToHistory(keyword)
  searching.value = true
  hasSearched.value = true

  try {
    const list = flattenWithTime(fileData.value, warehousePath.value)

    let results = []
    if (searchOptions.searchScope === 'path') {
      const matcher = buildTestRegex(keyword, searchOptions)
      results = searchByPath(list, matcher)
    } else if (searchOptions.searchScope === 'file') {
      const matcher = buildTestRegex(keyword, searchOptions)
      results = searchByFileName(list, matcher)
    } else {
      const matcher = buildGlobalRegex(keyword, searchOptions)
      results = await searchByContent(list, matcher)
    }

    searchResults.value = results

    // content 搜索时全部展开
    if (searchOptions.searchScope === 'content') {
      activeKeys.value = results.map(r => r.path)
    } else {
      activeKeys.value = []
    }
  } finally {
    searching.value = false
  }
}

// ==================== 排序 ====================
const sortOrder = ref('name_asc')
const selectedSortKeys = ref(['name_asc'])

const SORT_CONFIG = {
  name_asc: { field: 'name', order: 1 },
  name_desc: { field: 'name', order: -1 },
  modify_time_asc: { field: 'modifyTime', order: 1 },
  modify_time_desc: { field: 'modifyTime', order: -1 },
  create_time_asc: { field: 'createTime', order: 1 },
  create_time_desc: { field: 'createTime', order: -1 },
}

const sortedResults = computed(() => {
  const config = SORT_CONFIG[sortOrder.value]
  if (!config) return searchResults.value

  const results = [...searchResults.value]
  results.sort((a, b) => {
    const va = a[config.field] || ''
    const vb = b[config.field] || ''
    return va.localeCompare(vb) * config.order
  })
  return results
})

const handleSortChange = ({ key }) => {
  sortOrder.value = key
  selectedSortKeys.value = [key]
}

// ==================== 结果统计 ====================
const resultCountText = computed(() => {
  return t('file.searchResultCount').replace('{n}', sortedResults.value.length)
})

// ==================== 点击匹配项 → 打开文件阅读视图 → 定位到匹配位置 ====================
const handleMatchClick = (item, match) => {
  if (typeof openFile !== "function" || !item?.path) return
  openFile({ filePath: item.path, fileName: item.name })
  // 同步派发事件，ReadView 侧 watch content 变化后自动重试
  window.dispatchEvent(new CustomEvent('simple-write:scroll-to-match', {
    detail: {
      filePath: item.path,
      matchText: match.matchText.slice(match.matchStart, match.matchEnd),  // 匹配子串，用于高亮
      startPos: match.startPos,  // 匹配在文件中的绝对位置，用于等比滚动
    }
  }))
}

// ==================== 导航到文件（切换到 FileList 并展开闪烁） ====================
const navigateToFile = (item) => {
  const lastSlash = item.path.lastIndexOf('/')
  const parentPath = lastSlash > -1 ? item.path.substring(0, lastSlash) : item.path

  window.dispatchEvent(new CustomEvent('simple-write:switch-view', {
    detail: { view: 'filelist' }
  }))
  window.dispatchEvent(new CustomEvent('simple-write:expand-folder', {
    detail: { path: parentPath }
  }))
}

// ==================== 历史交互 ====================
const handleHistoryClick = (item) => {
  searchText.value = item
  historyVisible.value = false
  handleSearch()
}

const clearHistory = () => {
  searchHistory.value = []
  localStorage.removeItem(HISTORY_KEY)
  historyVisible.value = false
}
</script>

<style scoped>
.search-container {
  height: 100%;
  padding: 15px;
  display: flex;
  flex-direction: column;
}

/* 搜索栏 */
.search-row {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-bottom: 4px;
}

.search-input {
  flex: 1;
}

.setting-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  color: var(--text-secondary, #595959);
  flex-shrink: 0;
}

.setting-btn:hover {
  color: #1677ff;
  background: var(--bg-tertiary, #f5f5f5);
}

/* 设置下拉面板 */
.dropdown-panel {
  background: var(--bg-elevated, #fff);
  border-radius: 8px;
  padding: 8px 12px;
  min-width: 220px;
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08);
}

.option-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
}

.option-item + .option-item {
  border-top: 1px solid var(--border-secondary, #f0f0f0);
}

.option-select {
  padding: 8px 0 4px;
}

.option-label {
  font-size: 13px;
  color: var(--text-primary, #262626);
  white-space: nowrap;
  margin-right: 12px;
}

.scope-select {
  width: 110px;
}

/* 搜索历史面板 */
.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  padding-bottom: 6px;
  border-bottom: 1px solid var(--border-secondary, #f0f0f0);
}

.history-title {
  font-size: 13px;
  color: var(--text-tertiary, #8c8c8c);
}

.clear-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  color: var(--text-tertiary, #8c8c8c);
}

.clear-btn:hover {
  color: #ff4d4f;
}

.history-list {
  min-width: 180px;
  max-width: 300px;
}

.history-item {
  padding: 6px 8px;
  cursor: pointer;
  border-radius: 4px;
  font-size: 13px;
  color: var(--text-primary, #262626);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-item:hover {
  background: var(--bg-tertiary, #f5f5f5);
}

/* 搜索结果统计行 */
.search-stats {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0 8px;
  flex-shrink: 0;
}

.result-count {
  font-size: 13px;
  color: var(--text-tertiary, #8c8c8c);
}

.sort-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  color: var(--text-secondary, #595959);
}

.sort-btn:hover:not(:disabled) {
  color: #1677ff;
}

/* 搜索结果 */
.search-results {
  flex: 1;
  overflow-y: auto;
}

.search-loading {
  display: flex;
  justify-content: center;
  padding: 40px 0;
}

/* 折叠面板 */
.result-collapse {
  background: transparent;
}

.result-collapse :deep(.ant-collapse-item) {
  border-bottom: 1px solid var(--border-secondary, #f0f0f0);
}

.result-collapse :deep(.ant-collapse-header) {
  padding: 8px 0 !important;
  font-size: 14px;
}

.result-collapse :deep(.ant-collapse-header-text) {
  flex: 1 !important;
}

.result-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.result-header-text {
  color: var(--text-primary, #262626);
  user-select: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.locate-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  color: var(--text-tertiary, #8c8c8c);
  flex-shrink: 0;
  margin-left: 4px;
}

.locate-btn:hover {
  color: #1677ff;
}

/* 内容匹配列表 */
.match-list {
  padding: 4px 0 8px;
  border: 1px solid var(--border-color, #e8e8e8);
  border-radius: 6px;
  overflow: hidden;
}

.match-item {
  padding: 6px 10px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-color, #e8e8e8);
  transition: background 0.15s;
  font-size: 12px;
  line-height: 1.7;
  word-break: break-all;
  white-space: pre-wrap;
}

.match-item:last-child {
  border-bottom: none;
}

.match-item:hover {
  background: var(--bg-tertiary, #f5f5f5);
}

.match-line {
  white-space: pre-wrap;
  word-break: break-all;
}

.match-context {
  color: var(--text-tertiary, #8c8c8c);
}

.match-highlight {
  background: #ffd666;
  color: var(--text-primary, #262626);
  font-weight: 600;
  border-radius: 2px;
  padding: 0 1px;
}
</style>
