use std::fs;

// 读取文件内容 (增强版编码探测)
#[tauri::command]
fn read_markdown_file(path: String) -> Result<String, String> {
    eprintln!("Request to read file: '{}'", path); // Log the path
    let bytes = fs::read(&path).map_err(|e| format!("无法读取文件 '{}': {}", path, e))?;

    // 1. 如果能直接解析 UTF-8，优先使用 (Rust 默认最快，也能处理 UTF-8 BOM)
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        if s.starts_with('\u{feff}') {
            return Ok(s.trim_start_matches('\u{feff}').to_string());
        }
        return Ok(s);
    }

    // 2. 也是很常见的 Windows 格式 UTF-16 LE (BOM: FF FE)
    if bytes.len() >= 2 && bytes[0] == 0xff && bytes[1] == 0xfe {
        let (cow, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return Ok(cow.into_owned());
    }
    
    // 3. 也是很常见的 Windows 格式 UTF-16 BE (BOM: FE FF)
    if bytes.len() >= 2 && bytes[0] == 0xfe && bytes[1] == 0xff {
        let (cow, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return Ok(cow.into_owned());
    }

    // 4. 尝试 GB18030 (包含 GBK/GB2312)
    // 这是最容易跟 UTF-8 混淆的地方，如果强制解码且没有错误，那就大概率是它
    let (cow, _, had_errors) = encoding_rs::GB18030.decode(&bytes);
    if !had_errors {
        return Ok(cow.into_owned());
    }

    // 5. 如果以上都不是，最后用 chardetng 猜一个可能性最大的
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, true); // true = 允许回退到常见编码 (如 Windows-1252)

    let (final_cow, _, _) = encoding.decode(&bytes);
    eprintln!("此文件编码识别困难，最终猜测为: {}", encoding.name());
    
    Ok(final_cow.into_owned())
}

// 保存文件内容
#[tauri::command]
fn save_markdown_file(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("无法保存文件: {}", e))
}

#[derive(serde::Serialize, Clone)]
struct FileNode {
    name: String,
    path: String,
    is_dir: bool,
    children: Option<Vec<FileNode>>, // For recursive walking, though we'll keep it simple first
}

// 遍历目录下的 Markdown 文件
#[tauri::command]
fn list_markdown_files(path: String) -> Result<Vec<FileNode>, String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(&path).map_err(|e| format!("无法读取目录 [{}]: {}", path, e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path_buf = entry.path();
            let is_dir = path_buf.is_dir();
            let file_name = entry.file_name().to_string_lossy().to_string();
            let filestr = path_buf.to_string_lossy().to_string();

            // 调试信息：打印每个扫描到的文件
            eprintln!("Scanning: {} (is_dir: {})", file_name, is_dir);

            // 宽松过滤：
            // 1. 是文件夹
            // 2. 也是 Markdown (.md, .markdown)
            // 3. 忽略隐藏文件 (以 . 开头)
            if !file_name.starts_with('.') {
                let lower_name = file_name.to_lowercase();
                if is_dir || lower_name.ends_with(".md") || lower_name.ends_with(".markdown") {
                    files.push(FileNode {
                        name: file_name,
                        path: filestr,
                        is_dir,
                        children: None,
                    });
                }
            }
        }
    }
    
    // Sort directories first
    files.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    eprintln!("Found {} files in {}", files.len(), path);
    Ok(files)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            read_markdown_file,
            save_markdown_file,
            list_markdown_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
