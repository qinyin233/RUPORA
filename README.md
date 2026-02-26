# RUPORA

> A modern, lightweight Markdown editor built with **Tauri 2** + **Vue 3** + **Vditor**.

## Features

- WYSIWYG Markdown Editing - Powered by Vditor with IR (Instant Rendering) mode
- Multi-file Support - Open and switch between multiple Markdown files
- Outline View - Resizable document outline for quick navigation
- Native File Dialogs - Open/Save files using system-native dialogs
- Smart Encoding - Auto-detect UTF-8, GBK, UTF-16 and more
- Collapsible Sidebar - Resizable file list panel
- Keyboard Shortcuts - Ctrl+O to open, Ctrl+S to save

## Tech Stack

- **Backend**: Rust + Tauri 2
- **Frontend**: Vue 3 + TypeScript
- **Editor**: Vditor
- **Encoding**: encoding_rs + chardetng

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## License

MIT
