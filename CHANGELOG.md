# Changelog

本文件记录 RUPORA 的所有版本变更，格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [1.1.0] - 2026-02-27

### ✨ Apple 风格 UI 重构

#### 新增
- **毛玻璃设计（Glassmorphism）** - 侧边栏和工具栏采用毛玻璃效果，视觉更加现代
- **圆润的 UI 元素** - 所有按钮、列表项改为圆角设计（border-radius: 8px），更符合 Apple 风格
- **平滑的过渡动画** - 按钮及交互元素使用 cubic-bezier 曲线实现平滑、弹性的动画效果
- **SVG 图标** - 使用精美的 SVG 图标替代 emoji，设计统一、专业
- **优化的工具栏** - Vditor 工具栏按钮样式重新设计，鼠标悬停时有缩放效果
- **改进的颜色系统** - 引入 CSS 变量，统一管理主题色、文本色、边框色等
- **增强的大纲面板** - 大纲与正文之间的拖拽边框更易被发现，视觉反馈更强
- **文件列表改进** - 侧边栏文件列表显示改进，添加「已打开文件」分组标题
- **自定义滚动条** - 美化 Webkit 滚动条样式，与整体 UI 风格统一

#### 优化
- 提升工具栏按钮的交互体验，增加 hover、active 状态反馈
- 调整侧边栏默认宽度至 280px，提供更好的内容呈现空间
- 优化大纲面板背景，采用半透明效果与整体风格协调
- 改进状态栏显示，更清晰地展示编辑状态和文件信息

#### 技术
- 新增 `applyVditorStyling()` 函数，动态覆写 Vditor 样式
- 使用 CSS 作用域样式（scoped），避免样式冲突
- 引入 CSS 变量，便于后续主题定制

## [1.0.0] - 2026-02-27

### 🎉 首次发布

#### 新增
- **所见即所得编辑** - 基于 Vditor IR（即时渲染）模式，输入 Markdown 语法即时渲染
- **多文件支持** - 可同时打开多个 Markdown 文件，通过侧边栏快速切换
- **文档大纲导航** - 右侧自动生成标题大纲，支持点击跳转
- **智能编码检测** - 自动识别 UTF-8、UTF-8 BOM、UTF-16 LE/BE、GBK/GB18030 等编码
- **原生文件对话框** - 使用系统原生的打开/保存对话框
- **可折叠侧边栏** - 侧边栏可折叠，支持拖拽调整宽度（150px ~ 600px）
- **可拖拽大纲面板** - 大纲面板支持拖拽调整宽度（120px ~ 500px）
- **快捷键支持** - `Ctrl+O` 打开文件，`Ctrl+S` 保存文件
- **GFM 语法支持** - 完整支持 GitHub Flavored Markdown
- **代码高亮** - 内置代码语法高亮，支持数百种编程语言
- **数学公式** - 支持 KaTeX 数学公式渲染

#### 技术
- 使用 Tauri 2 + Rust 构建后端
- 使用 Vue 3 + TypeScript 构建前端
- 使用 Vditor 作为编辑器引擎
- 使用 encoding_rs + chardetng 进行编码检测
- Windows x64 安装包（NSIS + MSI）

[1.1.0]: https://github.com/qinyin233/RUPORA/releases/tag/v1.1.0
[1.0.0]: https://github.com/qinyin233/RUPORA/releases/tag/v1.0.0

### 🎉 首次发布

#### 新增
- **所见即所得编辑** - 基于 Vditor IR（即时渲染）模式，输入 Markdown 语法即时渲染
- **多文件支持** - 可同时打开多个 Markdown 文件，通过侧边栏快速切换
- **文档大纲导航** - 右侧自动生成标题大纲，支持点击跳转
- **智能编码检测** - 自动识别 UTF-8、UTF-8 BOM、UTF-16 LE/BE、GBK/GB18030 等编码
- **原生文件对话框** - 使用系统原生的打开/保存对话框
- **可折叠侧边栏** - 侧边栏可折叠，支持拖拽调整宽度（150px ~ 600px）
- **可拖拽大纲面板** - 大纲面板支持拖拽调整宽度（120px ~ 500px）
- **快捷键支持** - `Ctrl+O` 打开文件，`Ctrl+S` 保存文件
- **GFM 语法支持** - 完整支持 GitHub Flavored Markdown
- **代码高亮** - 内置代码语法高亮，支持数百种编程语言
- **数学公式** - 支持 KaTeX 数学公式渲染

#### 技术
- 使用 Tauri 2 + Rust 构建后端
- 使用 Vue 3 + TypeScript 构建前端
- 使用 Vditor 作为编辑器引擎
- 使用 encoding_rs + chardetng 进行编码检测
- Windows x64 安装包（NSIS + MSI）

[1.0.0]: https://github.com/qinyin233/RUPORA/releases/tag/v1.0.0
