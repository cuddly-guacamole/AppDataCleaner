use dirs_next as dirs;
use std::fs;
use std::path::PathBuf;

/// 格式化大小显示
pub fn format_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", size, UNITS[unit])
}

/// 获取指定类型的 AppData 目录
pub fn get_appdata_dir(folder_type: &str) -> Option<PathBuf> {
    match folder_type {
        "Roaming" => dirs::data_dir(),
        "Local" => dirs::cache_dir(),
        "LocalLow" => Some(PathBuf::from("C:/Users/Default/AppData/LocalLow")),
        _ => None,
    }
}

/// 计算文件夹总大小
pub fn calculate_folder_size(folder: &PathBuf) -> u64 {
    fn dir_size(dir: &PathBuf) -> u64 {
        fs::read_dir(dir)
            .unwrap_or_else(|_| fs::read_dir(dir).unwrap())
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    dir_size(&path)
                } else {
                    path.metadata().map(|meta| meta.len()).unwrap_or(0)
                }
            })
            .sum()
    }
    dir_size(folder)
}

/// 计算子文件夹占父文件夹大小的百分比
pub fn calculate_percentage(child_size: u64, parent_size: u64) -> f64 {
    if parent_size == 0 {
        return 0.0;
    }
    (child_size as f64 / parent_size as f64) * 100.0
}

/// 绘制横状图表示百分比
pub fn draw_percentage_bar(percentage: f64, bar_width: usize) -> String {
    let filled_width = (percentage / 100.0 * bar_width as f64).round() as usize;
    let empty_width = bar_width - filled_width;

    format!(
        "[{}{}] {:.2}%",
        "#".repeat(filled_width),
        " ".repeat(empty_width),
        percentage
    )
}
