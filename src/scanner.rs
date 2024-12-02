use std::sync::mpsc::Sender;
use std::thread;
use std::{fs, path::PathBuf};
use std::path::Path;

use dirs_next as dirs;
use crate::logger; // 引入日志模块

pub fn scan_appdata(tx: Sender<(String, u64)>, folder_type: &str) {
    println!("开始扫描 {} 类型的文件夹", folder_type);
    // 记录日志
    logger::log_info(&format!("开始扫描 {} 类型的文件夹", folder_type));

    // 根据 folder_type 确定要扫描的目录
    let appdata_dir = match folder_type {
        "Roaming" => dirs::data_dir(),
        "Local" => dirs::cache_dir(),
        "LocalLow" => Some(PathBuf::from("C:/Users/Default/AppData/LocalLow")), // 手动设置路径
        _ => None,
    };

    // 如果找到有效的目录，开始扫描
    if let Some(appdata_dir) = appdata_dir {
        thread::spawn(move || {
        let total_files = count_files(&appdata_dir); // 计算所有文件数量
        let mut processed_files = 0;
            if let Ok(entries) = fs::read_dir(&appdata_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_dir() {
                            let folder_name = entry.file_name().to_string_lossy().to_string();
                            let size = calculate_folder_size(&entry.path());
                            // 发送文件夹大小数据
                            tx.send((folder_name, size)).unwrap();
                        }
                        processed_files += 1;
                        // 计算已扫描的文件数量并发送进度数据
                        let progress = (processed_files as f64 / total_files as f64) * 100.0;
                        tx.send((format!("progress_{:.2}", progress), 0)).unwrap(); // 以字符串形式发送进度
                    }
                }
            }
            tx.send(("scan_complete".to_string(), 0)).unwrap(); // 扫描结束 
        });
    }
}
//统计目录及其子目录中的文件总数
fn count_files(folder: &Path) -> u64 {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_files(&path);
            } else if path.is_file() {
                count += 1;
            }
        }
    }
    count
}

// 计算文件夹的总大小（递归）
fn calculate_folder_size(folder: &Path) -> u64 {
    let mut size = 0;

    // 遍历文件夹中的所有条目
    if let Ok(entries) = fs::read_dir(folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // 递归计算子文件夹的大小
                size += calculate_folder_size(&path);
            } else if path.is_file() {
                // 计算文件大小
                if let Ok(metadata) = entry.metadata() {
                    size += metadata.len();
                }
            }
        }
    }

    size
}
