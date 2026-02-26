<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import Vditor from "vditor";
import "vditor/dist/index.css";

// --- Types ---
interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[];
}

// --- State ---
const vditor = ref<Vditor | null>(null);
const currentFilePath = ref<string | null>(null);
const statusMsg = ref("未命名文件");

const openedFiles = ref<FileNode[]>([]);
const isSidebarOpen = ref(true);
const sidebarWidth = ref(250); // Initial width

// --- Lifecycle ---
onMounted(() => {
  vditor.value = new Vditor("vditor", {
    height: "100%", // Changed from 100vh so it fits inside the flex container
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
      statusMsg.value = "准备就绪";
      // 添加大纲拖拽调整宽度功能
      setupOutlineResizer();
    },
    input: () => {
        statusMsg.value = "编辑中...";
    }
  });

  window.addEventListener("keydown", async (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "s") {
      e.preventDefault();
      await saveFile();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === "o") {
        e.preventDefault();
        openFileDialog(); // 快捷键打开文件对话框
    }
  });
});

// --- File Operations ---

async function onFileClick(node: FileNode) {
    if (node.path !== currentFilePath.value) {
        await openFile(node.path);
    }
}

async function openFileDialog() {
    try {
        const selected = await open({
            multiple: true, // 允许选择多个文件
            filters: [
                { name: "Markdown Files", extensions: ["md", "markdown", "MD", "MARKDOWN"] },
                { name: "All Files", extensions: ["*"] }
            ]
        });
        
        if (selected) {
            const paths = Array.isArray(selected) ? selected : [selected];
            for (const p of paths) {
                 // Check if already in list
                 if (!openedFiles.value.find(f => f.path === p)) {
                     // Extract filename. 
                     // Simple extraction, assumes Windows/Linux separator.
                     const name = p.split(/[/\\]/).pop() || p;
                     openedFiles.value.push({
                         name,
                         path: p,
                         is_dir: false
                     });
                 }
            }
            // Open the last selected file
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
        const content = await invoke<string>("read_markdown_file", { path });
        vditor.value?.setValue(content);
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
    invoke("save_markdown_file", { path, content });
    currentFilePath.value = path;
    statusMsg.value = `已保存: ${path}`;
    
    // Add to list if not present (e.g. new file)
    if (!openedFiles.value.find(f => f.path === path)) {
        const name = path!.split(/[/\\]/).pop() || path!;
        openedFiles.value.push({ name, path: path!, is_dir: false });
    }
    
  } catch (err: any) {
    alert(`保存失败: ${err}`);
  }
}

// --- Outline Resizer Logic ---
function setupOutlineResizer() {
    const outline = document.querySelector('.vditor-outline') as HTMLElement;
    if (!outline) return;

    let isResizing = false;
    let startX = 0;
    let startWidth = 0;

    // 监听鼠标在大纲左边缘的按下事件
    outline.addEventListener('mousedown', (e: MouseEvent) => {
        // 只在左边缘 6px 内触发拖拽
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

    document.addEventListener('mousemove', (e: MouseEvent) => {
        if (!isResizing) {
            // 当鼠标在大纲左边缘时显示 resize 光标
            const rect = outline.getBoundingClientRect();
            if (Math.abs(e.clientX - rect.left) < 6) {
                outline.style.cursor = 'col-resize';
            } else {
                outline.style.cursor = '';
            }
            return;
        }
        const diff = startX - e.clientX; // 向左拖 = 增大宽度
        const newWidth = Math.max(120, Math.min(startWidth + diff, 500));
        outline.style.width = newWidth + 'px';
    });

    document.addEventListener('mouseup', () => {
        if (isResizing) {
            isResizing = false;
            outline.style.cursor = '';
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }
    });
}

// --- Sidebar Resizer Logic ---
const startResize = (e: MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = sidebarWidth.value;

    const onMouseMove = (e: MouseEvent) => {
        const newWidth = startWidth + (e.clientX - startX);
        if (newWidth > 150 && newWidth < 600) { // Min/Max constraints
            sidebarWidth.value = newWidth;
        }
    }

    const onMouseUp = () => {
        document.removeEventListener('mousemove', onMouseMove);
        document.removeEventListener('mouseup', onMouseUp);
    }

    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
}

</script>

<template>
  <div class="app-layout">
    <!-- Sidebar -->
    <aside class="sidebar" v-if="isSidebarOpen" :style="{ width: sidebarWidth + 'px' }">
      <div class="sidebar-header">
        <button class="action-btn" @click="openFileDialog">
            <span class="icon">+</span> 打开文件
        </button>
        <button class="close-sidebar-btn" @click="isSidebarOpen = false" title="收起侧边栏">
            &lt;&lt;
        </button>
      </div>
      <!-- Removed current-path div -->
      <ul class="file-list">
        <li 
            v-for="node in openedFiles" 
            :key="node.path" 
            @click="onFileClick(node)"
            :class="{ active: currentFilePath === node.path }"
        >
          <span class="file-icon">📄</span> {{ node.name }}
        </li>
      </ul>
    </aside>

    <!-- Resizer -->
    <div class="resizer" v-if="isSidebarOpen" @mousedown="startResize"></div>

    <!-- Main Content -->
    <div class="main-content">
      <!-- Ensure editor doesn't push statusbar out -->
      <div id="vditor" class="editor"></div>
      
      <div class="statusbar" :class="{ 'sidebar-closed': !isSidebarOpen }">
          <button v-if="!isSidebarOpen" class="toggle-btn" @click="isSidebarOpen = true">
              📂 展开侧边栏
          </button>
          <span>{{ statusMsg }}</span>
      </div>
    </div>
  </div>
</template>

<style>
:root {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  height: 100%;
  margin: 0;
  overflow: hidden;
}

body { margin: 0; }

.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden; /* Ensure layout doesn't break */
}

/* Sidebar Styling */
.sidebar {
  width: 250px;
  background: #f8f9fa;
  border-right: 1px solid #e9ecef;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: width 0.2s ease; /* Smooth toggle */
}

/* Resizer Handle */
.resizer {
  width: 5px;
  cursor: col-resize;
  background-color: transparent;
  transition: background-color 0.2s;
  flex-shrink: 0;
}
.resizer:hover, .resizer:active {
    background-color: #0d6efd;
}

.sidebar-header {
  padding: 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid #e9ecef;
  background: #f1f3f5;
}

.action-btn {
    flex: 1;
    padding: 6px 12px;
    cursor: pointer;
    background: #fff;
    border: 1px solid #ced4da;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    font-weight: 500;
    margin-right: 8px;
}

.action-btn:hover {
    background-color: #f8f9fa;
    border-color: #adb5bd;
}

.close-sidebar-btn {
    padding: 6px 10px;
    cursor: pointer;
    background: transparent;
    border: none;
    font-size: 16px;
    color: #868e96;
    border-radius: 4px;
}

.close-sidebar-btn:hover {
    background-color: #e9ecef;
    color: #495057;
}

.file-list {
  list-style: none;
  padding: 0;
  margin: 0;
  overflow-y: auto;
  flex: 1;
}

.file-list li {
  padding: 8px 12px;
  cursor: pointer;
  font-size: 14px;
  color: #333;
  display: flex;
  align-items: center;
  gap: 8px;
  user-select: none;
  border-left: 3px solid transparent;
}

.file-list li:hover {
  background: #e9ecef;
}

.file-list li.active {
  background: #e7f1ff;
  color: #0d6efd;
  border-left-color: #0d6efd;
}

.file-icon {
    font-size: 16px;
}

/* Main Area */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0; /* Prevent flex overflow */
  overflow: hidden;
}

.editor {
  flex: 1;
  overflow: hidden;
  min-height: 0; /* Critical: allows flex child to shrink */
}

/* Fix Vditor to respect flex container */
.editor .vditor {
  height: 100% !important;
}

.statusbar {
    height: 28px;
    min-height: 28px; /* Prevent being squeezed out */
    background: #f1f3f5;
    color: #495057;
    font-size: 12px;
    display: flex;
    align-items: center;
    padding: 0 8px;
    border-top: 1px solid #dee2e6;
    gap: 10px;
    flex-shrink: 0; /* Never shrink the statusbar */
}

.toggle-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 12px;
    padding: 2px 8px;
    color: #0d6efd; 
    font-weight: 500;
    white-space: nowrap;
}

.toggle-btn:hover {
    background: #dee2e6;
    border-radius: 3px;
}

.sidebar-closed {
    background: #e9ecef;
    color: #333;
}

/* Vditor outline resizer - make the border between content and outline draggable */
.vditor-outline {
    border-left: 3px solid #e9ecef !important;
    transition: none !important;
}
.vditor-outline:hover {
    border-left-color: #0d6efd !important;
}
</style>
