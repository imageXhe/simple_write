// assets 下所有 SVG 的一次性导入，新增 SVG 文件后无需手动 import
const modules = import.meta.glob<string>("./*.svg", {
  query: "?raw",
  import: "default",
  eager: true,
});

const icons: Record<string, string> = {};
for (const [path, raw] of Object.entries(modules)) {
  // 从 "./code.svg" 提取 "code" 作为 key
  const name = path.replace("./", "").replace(".svg", "");
  icons[name] = raw;
}

export default icons;
