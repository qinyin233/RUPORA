<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save, confirm } from "@tauri-apps/plugin-dialog";
import Vditor from "vditor";
import "vditor/dist/index.css";

// --- Types ---
interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[];
  content?: string;
  is_dirty?: boolean;
}

// --- State ---
const vditor = ref<Vditor | null>(null);
const currentFilePath = ref<string | null>(null);
const statusMsg = ref("未命名文件");

const openedFiles = ref<FileNode[]>([]);
const isSidebarOpen = ref(true);
const sidebarWidth = ref(280);
const isDark = ref(false);
const themeStorageKey = "rupora-theme";

let exportToolbarHandler: ((e: Event) => void) | null = null;
let unlistenFileDrop: (() => void) | null = null;
let unlistenAppFileDrop: (() => void) | null = null;

// --- Lifecycle ---
onMounted(() => {
  initTheme();
  vditor.value = new Vditor("vditor", {
    height: "100%",
    width: "100%",
    mode: "ir",
    outline: {
      enable: true,
      position: "right",
    },
    toolbarConfig: {
      pin: true,
    },
    cache: {
      enable: false,
    },
    after: () => {
      setupOutlineResizer();
      applyVditorStyling();
      applyVditorTheme();
      setupExportOverrides();
    },
    input: () => {
      const path = currentFilePath.value;
      if (!path) return;
      const node = openedFiles.value.find(f => f.path === path);
      if (node) {
        node.content = vditor.value?.getValue() ?? "";
        node.is_dirty = true;
      }
    }
  });

  window.addEventListener("keydown", handleKeydown);

  setupTauriFileDrop();
});

async function handleKeydown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === "s") {
    e.preventDefault();
    e.stopPropagation();
    await saveFile();
  }
  if ((e.ctrlKey || e.metaKey) && e.key === "o") {
    e.preventDefault();
    e.stopPropagation();
    openFileDialog();
  }
}

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
  cleanupTauriFileDrop();
  cleanupOutlineResizer();
  cleanupExportOverrides();
  // 清理自定义 tooltip 元素
  const tooltipEl = document.getElementById("custom-toolbar-tooltip");
  if (tooltipEl) tooltipEl.remove();
});

async function setupTauriFileDrop() {
  if (unlistenFileDrop) return;
  const unlisten = await listen<unknown>("tauri://file-drop", async (event) => {
    await handleDropPaths(normalizeDropPayload(event.payload));
  });
  unlistenFileDrop = unlisten;

  const unlistenApp = await listen<string[]>("app-file-drop", async (event) => {
    await handleDropPaths(Array.isArray(event.payload) ? event.payload : []);
  });
  unlistenAppFileDrop = unlistenApp;
}

function cleanupTauriFileDrop() {
  if (unlistenFileDrop) {
    unlistenFileDrop();
    unlistenFileDrop = null;
  }
  if (unlistenAppFileDrop) {
    unlistenAppFileDrop();
    unlistenAppFileDrop = null;
  }
}

async function handleDropPaths(paths: string[]) {
  if (!paths.length) return;
  await openFilesByPaths(paths);
}

function normalizeDropPayload(payload: unknown) {
  if (Array.isArray(payload)) {
    return payload.filter((p): p is string => typeof p === "string" && p.length > 0);
  }
  if (payload && typeof payload === "object") {
    const paths = (payload as { paths?: unknown }).paths;
    if (Array.isArray(paths)) {
      return paths.filter((p): p is string => typeof p === "string" && p.length > 0);
    }
  }
  return [] as string[];
}


async function openFilesByPaths(paths: string[]) {
  for (const p of paths) {
    if (!openedFiles.value.find(f => f.path === p)) {
      const name = p.split(/[/\\]/).pop() || p;
      openedFiles.value.push({
        name,
        path: p,
        is_dir: false,
        content: "",
        is_dirty: false,
      });
    }
  }
  if (paths.length > 0) {
    await openFile(paths[paths.length - 1]);
  }
}

function initTheme() {
  const saved = localStorage.getItem(themeStorageKey);
  if (saved === "dark" || saved === "light") {
    isDark.value = saved === "dark";
  } else {
    isDark.value = window.matchMedia?.("(prefers-color-scheme: dark)")?.matches ?? false;
  }
  applyTheme();
}

function applyTheme() {
  const root = document.documentElement;
  root.dataset.theme = isDark.value ? "dark" : "light";
  localStorage.setItem(themeStorageKey, isDark.value ? "dark" : "light");
  applyVditorTheme();
}

function toggleTheme() {
  isDark.value = !isDark.value;
  applyTheme();
}

function applyVditorTheme() {
  const instance = vditor.value as any;
  if (!instance?.setTheme) return;
  const editorTheme = isDark.value ? "dark" : "classic";
  const contentTheme = isDark.value ? "dark" : "light";
  const codeTheme = isDark.value ? "dracula" : "github";
  instance.setTheme(editorTheme, contentTheme, codeTheme);
}

// --- Vditor Styling ---
function applyVditorStyling() {
  // 覆写工具栏样式：更圆润、毛玻璃效果
  const toolbar = document.querySelector(".vditor-toolbar") as HTMLElement;
  if (toolbar) {
    toolbar.style.backgroundColor = "var(--toolbar-bg)";
    toolbar.style.backdropFilter = "blur(20px)";
    toolbar.style.borderBottom = "1px solid var(--border-color)";
    toolbar.style.padding = "8px 12px";
  }

  // 创建全局 tooltip 容器（fixed 定位，不受 overflow 裁切）
  let tooltipEl = document.getElementById("custom-toolbar-tooltip") as HTMLElement;
  if (!tooltipEl) {
    tooltipEl = document.createElement("div");
    tooltipEl.id = "custom-toolbar-tooltip";
    tooltipEl.style.cssText = `
      position: fixed;
      z-index: 99999;
      pointer-events: none;
      background: rgba(0, 0, 0, 0.85);
      color: #fff;
      font-size: 12px;
      line-height: 1.2;
      padding: 6px 10px;
      border-radius: 6px;
      white-space: nowrap;
      opacity: 0;
      transition: opacity 0.15s ease;
      box-shadow: 0 2px 8px rgba(0,0,0,0.15);
    `;
    document.body.appendChild(tooltipEl);
  }

  // 工具栏按钮样式：圆润简洁风格 + 自定义 tooltip
  const buttons = document.querySelectorAll(".vditor-toolbar button");
  buttons.forEach((btn: Element) => {
    const element = btn as HTMLElement;
    element.style.borderRadius = "8px";
    element.style.margin = "2px 4px";
    element.style.padding = "6px 10px";
    element.style.transition = "all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1)";
    element.style.backgroundColor = "transparent";
    element.style.border = "none";
    element.style.color = "var(--toolbar-text)";
    element.style.fontSize = "13px";
    element.style.fontWeight = "500";

    // 获取 tooltip 文本（aria-label 或 data-type）
    const tipText = element.getAttribute("aria-label")
      || element.getAttribute("data-type")
      || element.getAttribute("title")
      || "";

    // 清除原生 tooltip 属性，防止浏览器默认 tooltip
    if (element.hasAttribute("title")) {
      element.removeAttribute("title");
    }

    let hoverTimer: ReturnType<typeof setTimeout> | null = null;

    element.addEventListener("mouseenter", () => {
      element.style.backgroundColor = "rgba(0, 0, 0, 0.05)";
      element.style.transform = "scale(1.05)";

      if (!tipText) return;
      hoverTimer = setTimeout(() => {
        const rect = element.getBoundingClientRect();
        tooltipEl.textContent = tipText;
        tooltipEl.style.opacity = "1";
        // 先显示以获取尺寸
        const tipRect = tooltipEl.getBoundingClientRect();
        let left = rect.left + rect.width / 2 - tipRect.width / 2;
        let top = rect.bottom + 6;
        // 防止超出右边界
        if (left + tipRect.width > window.innerWidth - 8) {
          left = window.innerWidth - tipRect.width - 8;
        }
        // 防止超出左边界
        if (left < 8) left = 8;
        tooltipEl.style.left = left + "px";
        tooltipEl.style.top = top + "px";
      }, 300);
    });

    element.addEventListener("mouseleave", () => {
      element.style.backgroundColor = "transparent";
      element.style.transform = "scale(1)";
      if (hoverTimer) { clearTimeout(hoverTimer); hoverTimer = null; }
      tooltipEl.style.opacity = "0";
    });

    element.addEventListener("mousedown", () => {
      element.style.backgroundColor = "rgba(0, 0, 0, 0.12)";
      element.style.transform = "scale(0.98)";
      if (hoverTimer) { clearTimeout(hoverTimer); hoverTimer = null; }
      tooltipEl.style.opacity = "0";
    });
  });

  // 覆写大纲样式：更柔和的边框
  const outline = document.querySelector(".vditor-outline") as HTMLElement;
  if (outline) {
    outline.style.borderLeft = "1px solid var(--border-color)";
    outline.style.backgroundColor = "var(--outline-bg)";
  }

  // 大纲标题链接
  const outlineLinks = document.querySelectorAll(".vditor-outline a");
  outlineLinks.forEach((link: Element) => {
    const element = link as HTMLElement;
    element.style.borderRadius = "4px";
    element.style.transition = "all 0.15s ease";
    element.addEventListener("mouseover", () => {
      element.style.backgroundColor = "var(--outline-hover-bg)";
    });
    element.addEventListener("mouseout", () => {
      element.style.backgroundColor = "transparent";
    });
  });

  // 编辑器容器背景
  const editor = document.querySelector(".vditor-ir") as HTMLElement;
  if (editor) {
    editor.style.backgroundColor = "var(--editor-bg)";
  }
}

function setupExportOverrides() {
  const toolbar = document.querySelector(".vditor-toolbar");
  if (!toolbar || exportToolbarHandler) return;
  exportToolbarHandler = async (event: Event) => {
    const target = event.target as HTMLElement | null;
    if (!target || target.tagName !== "BUTTON") return;
    const dataType = target.getAttribute("data-type");
    if (!dataType) return;
    if (dataType !== "html" && dataType !== "pdf") return;
    if (!target.closest(".vditor-hint")) return;

    event.preventDefault();
    event.stopPropagation();
    if (dataType === "html") {
      await exportHtmlFile();
    } else if (dataType === "pdf") {
      await exportPdfFile();
    }
  };
  toolbar.addEventListener("click", exportToolbarHandler, true);
}

function cleanupExportOverrides() {
  const toolbar = document.querySelector(".vditor-toolbar");
  if (toolbar && exportToolbarHandler) {
    toolbar.removeEventListener("click", exportToolbarHandler, true);
  }
  exportToolbarHandler = null;
}

function getExportBaseName() {
  const path = currentFilePath.value;
  if (!path) return "untitled";
  const name = path.split(/[/\\]/).pop() || "untitled";
  return name.replace(/\.[^/.]+$/, "");
}

async function getRenderedHtml() {
  const instance = vditor.value as any;
  if (instance?.getHTML) {
    return instance.getHTML() as string;
  }
  const markdown = instance?.getValue?.() ?? "";
  const container = document.createElement("div");
  await Promise.resolve(
    Vditor.preview(container, markdown, {
      mode: "light",
      theme: { current: isDark.value ? "dark" : "light" },
    })
  );
  const html = container.innerHTML;
  container.remove();
  return html;
}

function buildExportHtml(bodyHtml: string) {
  const themeClass = isDark.value ? "vditor-theme-dark" : "vditor-theme-light";
  const background = isDark.value ? "#0f172a" : "#ffffff";
  const text = isDark.value ? "#e5e7eb" : "#1f2937";
  return `<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <link rel="stylesheet" href="https://unpkg.com/vditor@3.11.2/dist/index.css" />
  <style>
    body { margin: 0; padding: 24px; background: ${background}; color: ${text}; }
    .vditor-reset { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
  </style>
</head>
<body class="${themeClass}">
  <div class="vditor-reset">${bodyHtml}</div>
</body>
</html>`;
}

async function exportHtmlFile() {
  const bodyHtml = await getRenderedHtml();
  if (!bodyHtml) return;
  const defaultName = `${getExportBaseName()}.html`;
  const selected = await save({
    defaultPath: defaultName,
    filters: [{ name: "HTML", extensions: ["html", "htm"] }],
  });
  if (!selected) return;
  const htmlContent = buildExportHtml(bodyHtml);
  await invoke("export_html_file", { path: selected, html_content: htmlContent });
  statusMsg.value = `已导出: ${selected}`;
}

async function exportPdfFile() {
  const bodyHtml = await getRenderedHtml();
  if (!bodyHtml) return;
  const htmlContent = buildExportHtml(bodyHtml);
  const iframe = document.createElement("iframe");
  iframe.style.position = "fixed";
  iframe.style.right = "0";
  iframe.style.bottom = "0";
  iframe.style.width = "0";
  iframe.style.height = "0";
  iframe.style.border = "0";
  document.body.appendChild(iframe);
  const doc = iframe.contentDocument;
  if (doc) {
    doc.open();
    doc.write(htmlContent);
    doc.close();
    setTimeout(() => {
      iframe.contentWindow?.focus();
      iframe.contentWindow?.print();
      iframe.remove();
    }, 300);
  } else {
    iframe.remove();
  }
}

// --- File Operations ---

async function onFileClick(node: FileNode) {
    if (node.path !== currentFilePath.value) {
        await openFile(node.path);
    }
}

async function openFileDialog() {
    try {
        const selected = await open({
            multiple: true,
            filters: [
                { name: "Markdown Files", extensions: ["md", "markdown", "MD", "MARKDOWN"] },
                { name: "All Files", extensions: ["*"] }
            ]
        });
        
        if (selected) {
            const paths = Array.isArray(selected) ? selected : [selected];
            for (const p of paths) {
                if (!openedFiles.value.find(f => f.path === p)) {
                    const name = p.split(/[/\\]/).pop() || p;
                    openedFiles.value.push({
                        name,
                        path: p,
                  is_dir: false,
                  content: "",
                  is_dirty: false
                    });
                }
            }
            if (paths.length > 0) {
                await openFile(paths[paths.length - 1]);
            }
        }
    } catch (err: any) {
        alert(`打开文件对话框失败: ${err}`);
    }
}

async function openFile(path: string) {
    try {
    const node = openedFiles.value.find(f => f.path === path);
    if (node && node.is_dirty && typeof node.content === "string") {
      vditor.value?.setValue(node.content);
    } else {
      const content = await invoke<string>("read_markdown_file", { path });
      vditor.value?.setValue(content);
      if (node) {
        node.content = content;
        node.is_dirty = false;
      }
    }
        currentFilePath.value = path;
        statusMsg.value = `已打开: ${path}`;
    } catch (err: any) {
        alert(`打开失败: ${err}`);
    }
}

async function saveFile() {
  const content = vditor.value?.getValue();
  if (!content) return;

  let path = currentFilePath.value;
  if (!path) {
      const selected = await save({
          filters: [{ name: "Markdown File", extensions: ["md", "markdown"] }]
      });
      if (!selected) return;
      path = selected as string;
  }

  try {
    await invoke("save_markdown_file", { path, content });
    currentFilePath.value = path;
    statusMsg.value = path;
    
    const node = openedFiles.value.find(f => f.path === path);
    if (!node) {
      const name = path!.split(/[/\\]/).pop() || path!;
      openedFiles.value.push({ name, path: path!, is_dir: false, content, is_dirty: false });
    } else {
      node.content = content;
      node.is_dirty = false;
    }
    
  } catch (err: any) {
    alert(`保存失败: ${err}`);
  }
}

  async function saveFileToPath(path: string, content: string) {
    try {
      await invoke("save_markdown_file", { path, content });
      const node = openedFiles.value.find(f => f.path === path);
      if (node) {
        node.content = content;
        node.is_dirty = false;
      }
      if (currentFilePath.value === path) {
        statusMsg.value = path;
      }
    } catch (err: any) {
      alert(`保存失败: ${err}`);
    }
  }

  async function closeFile(node: FileNode) {
    if (node.is_dirty) {
      const shouldSave = await confirm("文件已修改，是否保存？", {
        title: "RUPORA",
        okLabel: "保存",
        cancelLabel: "不保存"
      });
      if (shouldSave) {
        const content = node.content ?? vditor.value?.getValue() ?? "";
        await saveFileToPath(node.path, content);
      }
    }

    openedFiles.value = openedFiles.value.filter(f => f.path !== node.path);

    if (currentFilePath.value === node.path) {
      const next = openedFiles.value[openedFiles.value.length - 1];
      if (next) {
        await openFile(next.path);
      } else {
        currentFilePath.value = null;
        statusMsg.value = "未命名文件";
        vditor.value?.setValue("");
      }
    }
  }

// --- Outline Resizer Logic ---
let outlineMoveHandler: ((e: MouseEvent) => void) | null = null;
let outlineUpHandler: (() => void) | null = null;

function setupOutlineResizer() {
    const outline = document.querySelector('.vditor-outline') as HTMLElement;
    if (!outline) return;

    let isResizing = false;
    let startX = 0;
    let startWidth = 0;

    outline.addEventListener('mousedown', (e: MouseEvent) => {
        const rect = outline.getBoundingClientRect();
        if (e.clientX - rect.left > 6) return;

        isResizing = true;
        startX = e.clientX;
        startWidth = outline.offsetWidth;
        e.preventDefault();
        outline.style.cursor = 'col-resize';
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';
    });

    outlineMoveHandler = (e: MouseEvent) => {
        if (!isResizing) {
            const rect = outline.getBoundingClientRect();
            if (Math.abs(e.clientX - rect.left) < 6) {
                outline.style.cursor = 'col-resize';
            } else {
                outline.style.cursor = '';
            }
            return;
        }
        const diff = startX - e.clientX;
        const newWidth = Math.max(120, Math.min(startWidth + diff, 500));
        outline.style.width = newWidth + 'px';
    };

    outlineUpHandler = () => {
        if (isResizing) {
            isResizing = false;
            outline.style.cursor = '';
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }
    };

    document.addEventListener('mousemove', outlineMoveHandler);
    document.addEventListener('mouseup', outlineUpHandler);
}

function cleanupOutlineResizer() {
    if (outlineMoveHandler) {
        document.removeEventListener('mousemove', outlineMoveHandler);
        outlineMoveHandler = null;
    }
    if (outlineUpHandler) {
        document.removeEventListener('mouseup', outlineUpHandler);
        outlineUpHandler = null;
    }
}

// --- Sidebar Resizer Logic ---
const startResize = (e: PointerEvent) => {
  e.preventDefault();
  const startX = e.clientX;
  const startWidth = sidebarWidth.value;
  const target = e.currentTarget as HTMLElement | null;

  const onPointerMove = (evt: PointerEvent) => {
    const nextWidth = startWidth + (evt.clientX - startX);
    if (nextWidth > 150 && nextWidth < 600) {
      sidebarWidth.value = nextWidth;
    }
  };

  const onPointerUp = () => {
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    document.removeEventListener('pointermove', onPointerMove);
    document.removeEventListener('pointerup', onPointerUp);
  };

  if (target && e.pointerId !== undefined) {
    target.setPointerCapture(e.pointerId);
  }
  document.body.style.cursor = "col-resize";
  document.body.style.userSelect = "none";
  document.addEventListener('pointermove', onPointerMove);
  document.addEventListener('pointerup', onPointerUp);
}

</script>

<template>
  <div class="app-layout" :class="{ dark: isDark }">
    <!-- Sidebar -->
    <aside class="sidebar" v-if="isSidebarOpen" :style="{ width: sidebarWidth + 'px' }">
      <div class="sidebar-header">
        <button class="action-btn" @click="openFileDialog" title="打开 Markdown 文件">
          <svg class="icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          <span>打开</span>
        </button>
      </div>

      <div class="file-section">
        <div class="file-section-title">已打开文件</div>
        <ul class="file-list" v-if="openedFiles.length > 0">
          <li 
            v-for="node in openedFiles" 
            :key="node.path" 
            @click="onFileClick(node)"
            :class="{ active: currentFilePath === node.path }"
            :title="node.path"
          >
            <span class="file-dot" :class="{ visible: node.is_dirty }"></span>
            <svg class="file-icon" viewBox="0 0 24 24" fill="currentColor">
              <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"></path>
              <polyline points="13 2 13 9 20 9"></polyline>
            </svg>
            <span class="file-name">{{ node.name }}</span>
            <button class="file-close" title="关闭" @click.stop="closeFile(node)">×</button>
          </li>
        </ul>
        <div v-else class="empty-state">暂无打开文件</div>
      </div>
    </aside>

    <!-- Resizer -->
    <div class="resizer" v-if="isSidebarOpen" @pointerdown="startResize"></div>

    <!-- Main Content -->
    <div class="main-content">
      <div id="vditor" class="editor"></div>
      
      <div class="statusbar">
        <button
          v-if="isSidebarOpen"
          class="sidebar-toggle"
          @click="isSidebarOpen = false"
          title="收起侧边栏"
        >
          <svg class="icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
        <button
          v-else
          class="sidebar-toggle"
          @click="isSidebarOpen = true"
          title="展开侧边栏"
        >
          <svg class="icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
        <span class="status-text">{{ statusMsg }}</span>
        <button class="status-action" @click="toggleTheme" :title="isDark ? '切换到浅色' : '切换到深色'">
          <svg v-if="isDark" class="icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="4"></circle>
            <line x1="12" y1="2" x2="12" y2="4"></line>
            <line x1="12" y1="20" x2="12" y2="22"></line>
            <line x1="2" y1="12" x2="4" y2="12"></line>
            <line x1="20" y1="12" x2="22" y2="12"></line>
            <line x1="4.9" y1="4.9" x2="6.3" y2="6.3"></line>
            <line x1="17.7" y1="17.7" x2="19.1" y2="19.1"></line>
            <line x1="4.9" y1="19.1" x2="6.3" y2="17.7"></line>
            <line x1="17.7" y1="6.3" x2="19.1" y2="4.9"></line>
          </svg>
          <svg v-else class="icon-svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3a7 7 0 0 0 9.79 9.79z"></path>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
:global(html, body, #app) {
  height: 100%;
  margin: 0;
  overflow: hidden;
}

:global(:root) {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  --primary-color: #3b82f6;
  --primary-light: #93c5fd;
  --text-primary: #1f2937;
  --text-secondary: #6b7280;
  --bg-secondary: #f3f4f6;
  --border-color: rgba(0, 0, 0, 0.08);
  --divider: rgba(0, 0, 0, 0.06);
  --app-bg: #ffffff;
  --sidebar-bg: rgba(255, 255, 255, 0.7);
  --sidebar-header-bg: rgba(243, 244, 246, 0.5);
  --editor-bg: #ffffff;
  --outline-bg: rgba(255, 255, 255, 0.4);
  --outline-hover-bg: rgba(59, 130, 246, 0.1);
  --statusbar-bg: rgba(243, 244, 246, 0.6);
  --toolbar-bg: rgba(255, 255, 255, 0.7);
  --toolbar-text: #555;
  --hover-bg: rgba(0, 0, 0, 0.05);
  --file-hover-bg: rgba(59, 130, 246, 0.08);
  --file-active-bg: rgba(59, 130, 246, 0.15);
  --action-bg: rgba(59, 130, 246, 0.1);
  --action-border: rgba(59, 130, 246, 0.2);
  --action-hover-bg: rgba(59, 130, 246, 0.15);
  --action-hover-border: rgba(59, 130, 246, 0.3);
  --action-active-bg: rgba(59, 130, 246, 0.25);
  --scrollbar-thumb: rgba(0, 0, 0, 0.15);
  --scrollbar-thumb-hover: rgba(0, 0, 0, 0.25);
}

:global(:root[data-theme="dark"]) {
  --text-primary: #e5e7eb;
  --text-secondary: #9ca3af;
  --bg-secondary: #111827;
  --border-color: rgba(255, 255, 255, 0.08);
  --divider: rgba(255, 255, 255, 0.06);
  --app-bg: #0b1220;
  --sidebar-bg: rgba(17, 24, 39, 0.85);
  --sidebar-header-bg: rgba(17, 24, 39, 0.7);
  --editor-bg: #0f172a;
  --outline-bg: rgba(15, 23, 42, 0.6);
  --outline-hover-bg: rgba(59, 130, 246, 0.2);
  --statusbar-bg: rgba(17, 24, 39, 0.7);
  --toolbar-bg: rgba(17, 24, 39, 0.7);
  --toolbar-text: #e5e7eb;
  --hover-bg: rgba(255, 255, 255, 0.08);
  --file-hover-bg: rgba(59, 130, 246, 0.2);
  --file-active-bg: rgba(59, 130, 246, 0.25);
  --action-bg: rgba(59, 130, 246, 0.2);
  --action-border: rgba(59, 130, 246, 0.35);
  --action-hover-bg: rgba(59, 130, 246, 0.3);
  --action-hover-border: rgba(59, 130, 246, 0.45);
  --action-active-bg: rgba(59, 130, 246, 0.4);
  --scrollbar-thumb: rgba(255, 255, 255, 0.2);
  --scrollbar-thumb-hover: rgba(255, 255, 255, 0.35);
}

* {
  box-sizing: border-box;
}

.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--app-bg);
}

/* ============ SIDEBAR ============ */
.sidebar {
  width: 280px;
  background: var(--sidebar-bg);
  backdrop-filter: blur(20px);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: opacity 0.2s ease;
}

.sidebar-header {
  padding: 12px 10px;
  display: flex;
  align-items: center;
  gap: 8px;
  border-bottom: 1px solid var(--divider);
  background: var(--sidebar-header-bg);
}

.action-btn {
  flex: 1;
  padding: 8px 12px;
  cursor: pointer;
  background: var(--action-bg);
  border: 1px solid var(--action-border);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-weight: 500;
  font-size: 13px;
  color: var(--text-primary);
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.action-btn:hover {
  background: var(--action-hover-bg);
  border-color: var(--action-hover-border);
  transform: scale(1.02);
}

.action-btn:active {
  background: var(--action-active-bg);
  transform: scale(0.98);
}

.collapse-btn {
  padding: 6px 10px;
  cursor: pointer;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--text-secondary);
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.collapse-btn:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.icon-svg {
  width: 16px;
  height: 16px;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.file-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 12px 0;
}

.file-section-title {
  padding: 0 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.file-list {
  list-style: none;
  padding: 0;
  margin: 0;
  overflow-y: auto;
  overflow-x: hidden;
  flex: 1;
  padding: 0 6px;
}

.file-list::-webkit-scrollbar {
  width: 4px;
}

.file-list::-webkit-scrollbar-thumb {
  background: var(--scrollbar-thumb);
  border-radius: 4px;
}

.file-list::-webkit-scrollbar-thumb:hover {
  background: var(--scrollbar-thumb-hover);
}

.file-list::-webkit-scrollbar-track {
  background: transparent;
}

.file-list li {
  padding: 8px 10px;
  margin: 2px 0;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 8px;
  user-select: none;
  border-radius: 8px;
  transition: all 0.15s ease;
  white-space: nowrap;
  overflow: hidden;
}

.file-dot {
  width: 6px;
  height: 6px;
  border-radius: 999px;
  background: transparent;
  flex-shrink: 0;
}

.file-dot.visible {
  background: #ef4444;
}

.file-list li:hover {
  background: var(--file-hover-bg);
  color: var(--primary-color);
}

.file-list li.active {
  background: var(--file-active-bg);
  color: var(--primary-color);
  font-weight: 500;
}

.file-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.file-close {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  padding: 2px 4px;
  border-radius: 6px;
  flex-shrink: 0;
}

.file-close:hover {
  background: var(--hover-bg);
  color: var(--text-primary);
}

.empty-state {
  padding: 24px 12px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

/* ============ RESIZER ============ */
.resizer {
  width: 4px;
  cursor: col-resize;
  background-color: transparent;
  transition: background-color 0.2s;
  flex-shrink: 0;
}

.resizer:hover {
  background-color: var(--primary-light);
}

.resizer:active {
  background-color: var(--primary-color);
}

/* ============ MAIN CONTENT ============ */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

.editor {
  flex: 1;
  overflow: visible;
  min-height: 0;
  background: var(--editor-bg);
}

.editor .vditor {
  height: 100% !important;
}

/* ============ STATUSBAR ============ */
.statusbar {
  height: 32px;
  min-height: 32px;
  background: var(--statusbar-bg);
  backdrop-filter: blur(10px);
  color: var(--text-secondary);
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding: 0 12px 6px;
  border-top: 1px solid var(--border-color);
  gap: 8px;
  flex-shrink: 0;
  line-height: 1;
}

.sidebar-toggle {
  padding: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  border-radius: 6px;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 22px;
  width: 22px;
  flex-shrink: 0;
}

.sidebar-toggle:hover {
  background: var(--hover-bg);
  color: var(--primary-color);
}

.status-action {
  padding: 0;
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--text-secondary);
  border-radius: 6px;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 22px;
  width: 22px;
  flex-shrink: 0;
}

.status-action:hover {
  background: var(--hover-bg);
  color: var(--primary-color);
}

.status-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 32px;
}

/* ============ VDITOR CUSTOMIZATION ============ */
:deep(.vditor-toolbar) {
  background: var(--toolbar-bg) !important;
  backdrop-filter: blur(20px) !important;
  border-bottom: 1px solid var(--border-color) !important;
  padding: 8px 12px !important;
  position: relative !important;
  z-index: 30;
  overflow: visible !important;
}

/* 隐藏 Vditor 内置的 tooltip 伪元素（使用 JS 自定义 fixed tooltip 替代） */
:deep(.vditor-tooltipped)::before,
:deep(.vditor-tooltipped)::after {
  display: none !important;
}

:deep(.vditor-tooltip) {
  display: none !important;
}

:deep(.vditor-toolbar button) {
  border-radius: 8px !important;
  margin: 2px 4px !important;
  padding: 6px 10px !important;
  transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1) !important;
  background: transparent !important;
  border: none !important;
  color: var(--toolbar-text) !important;
  font-size: 13px !important;
  font-weight: 500 !important;
}

:deep(.vditor-toolbar button:hover) {
  background: var(--hover-bg) !important;
  transform: scale(1.05) !important;
}

:deep(.vditor-toolbar button:active) {
  background: rgba(0, 0, 0, 0.12) !important;
  transform: scale(0.98) !important;
}

:deep(.vditor-outline) {
  border-left: 1px solid var(--border-color) !important;
  background: var(--outline-bg) !important;
  transition: none !important;
}

:deep(.vditor-outline a) {
  border-radius: 4px;
  transition: all 0.15s ease;
}

:deep(.vditor-outline a:hover) {
  background: var(--outline-hover-bg) !important;
}

:deep(.vditor-ir) {
  background: var(--editor-bg) !important;
}
</style>
