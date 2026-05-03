<template>
  <span class="anticon svg-icon" v-html="svgContent"></span>
</template>

<script setup>
import { computed } from "vue";

const props = defineProps({
  // SVG 原始字符串（通过 ?raw 导入）
  raw: { type: String, required: true },
});

const svgContent = computed(() => {
  return props.raw
    // 移除硬编码宽高，改用 CSS 控制
    .replace(/\bwidth="[^"]*"/g, "")
    .replace(/\bheight="[^"]*"/g, "")
    // 将硬编码 fill 替换为 currentColor，使其继承父级颜色
    .replace(/\bfill="(?!none|currentColor)[^"]*"/g, 'fill="currentColor"');
});
</script>

<style scoped>
.svg-icon :deep(svg) {
  display: block;
  width: 14px;
  height: 14px;
  fill: currentColor;
}
</style>
