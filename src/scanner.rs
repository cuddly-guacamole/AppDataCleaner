use dirs_next as dirs;
use std::fs::{self};
use std::sync::mpsc::Sender;

pub fn scan_appdata(target: &str, selected_target: &str, tx: Sender<(String, u64)>) {
    let appdata_dir = match dirs::data_dir() {
        Some(path) => match selected_target {
            "Local" => path.parent().unwrap().join("Local"),
            "Roaming" => path,
            "LocalLow" => path.parent().unwrap().join("LocalLow"),
            _ => return, // 如果路径不匹配，返回
        },
        None => return,
    };

    println!("开始扫描文件夹: {}", target);
    println!("扫描的路径: {}", appdata_dir.display());

    if appdata_dir.exists() {
        let entries = std::fs::read_dir(appdata_dir)
            .expect("Failed to read directory");

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let folder_name = path.file_name()
                        .and_then(|os_str| os_str.to_str())
                        .unwrap_or("<未知文件夹>")
                        .to_owned();
                    let size = calculate_folder_size(&path);
                    tx.send((folder_name, size)).unwrap();
                }
            }
        }
    }
}

fn calculate_folder_size(folder: &std::path::Path) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    size += metadata.len();
                } else if metadata.is_dir() {
                    size += calculate_folder_size(&entry.path());
                }
            }
        }
    }
    size
}
