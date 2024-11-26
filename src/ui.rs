use crate::about;
use crate::confirmation;
use crate::delete;
use crate::scanner;
use crate::utils;
use eframe::egui::{self, Grid, ScrollArea};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use dirs_next as dirs;

#[derive(Clone)]  // 添加 Clone trait 以支持克隆
pub struct AppDataCleaner {
    is_scanning: bool,
    current_folder: Option<String>,
    folder_data: Arc<Mutex<Vec<(String, u64)>>>, // 使用 Arc<Mutex> 包装数据
    show_about_window: bool,                // 确保字段存在
    confirm_delete: Option<(String, bool)>, // 保存要确认删除的文件夹状态
    selected_target: String,                // 当前选择的目标文件夹
}

impl Default for AppDataCleaner {
    fn default() -> Self {
        Self {
            is_scanning: false,
            current_folder: None,
            folder_data: Arc::new(Mutex::new(vec![])), // 初始化
            show_about_window: false, // 默认值
            confirm_delete: None,     // 初始化为 None
            selected_target: "Local".to_string(), // 默认扫描 Local 文件夹
        }
    }
}

impl AppDataCleaner {
    fn setup_custom_fonts(&self, ctx: &egui::Context) {
        use eframe::egui::{FontData, FontDefinitions};

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "custom_font".to_owned(),
            FontData::from_static(include_bytes!("../assets/SourceHanSansCN-Regular.otf")),
        );

        fonts.families.insert(
            egui::FontFamily::Proportional,
            vec!["custom_font".to_owned()],
        );
        fonts
            .families
            .insert(egui::FontFamily::Monospace, vec!["custom_font".to_owned()]);

        ctx.set_fonts(fonts);
    }

    // 添加 start_scan 函数作为 AppDataCleaner 结构体的一部分
    fn start_scan(&mut self) {
        let (tx, rx) = mpsc::channel();

        // 将 `self` 包装在 `Arc<Mutex<Self>>` 中，允许线程共享
        let appdata_cleaner: Arc<Mutex<AppDataCleaner>> = Arc::new(Mutex::new(self.clone()));

        std::thread::spawn({
            let appdata_cleaner = Arc::clone(&appdata_cleaner); // 克隆 `Arc`，允许线程访问
            move || {
                let appdata_cleaner = appdata_cleaner.lock().unwrap(); // 获取锁
                scanner::scan_appdata(&appdata_cleaner.selected_target, &appdata_cleaner.selected_target, tx);
            }
        });
    }
}

impl eframe::App for AppDataCleaner {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.setup_custom_fonts(ctx);

        if let Some((folder, _confirm)) = &self.confirm_delete {
            if let Some(result) =
                confirmation::show_confirmation(ctx, &format!("确定要删除文件夹 {} 吗？", folder))
            {
                if result {
                    if let Err(err) = delete::delete_folder(&folder) {
                        eprintln!("删除失败: {}", err);
                    }
                }
                self.confirm_delete = None;
            }
        }

        // 顶部菜单栏
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            if ui.button("关于").clicked() {
                self.show_about_window = true; // 打开关于窗口
            }
            ui.menu_button("扫描文件夹", |ui| {
                if ui.button("Local").clicked() {
                    self.selected_target = "Local".to_string();
                    self.start_scan(); // 切换文件夹后开始扫描
                    ui.close_menu();
                }
                if ui.button("Roaming").clicked() {
                    self.selected_target = "Roaming".to_string();
                    self.start_scan(); // 切换文件夹后开始扫描
                    ui.close_menu();
                }
                if ui.button("LocalLow").clicked() {
                    self.selected_target = "LocalLow".to_string();
                    self.start_scan(); // 切换文件夹后开始扫描
                    ui.close_menu();
                }
            });
            // 添加“立即扫描”按钮
            if ui.button("立即扫描").clicked() {
                self.start_scan(); // 直接启动扫描
            }
        });


        // 当前目标标签
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(format!("当前目标: {}", self.selected_target));
        });

        // 显示关于窗口
        if self.show_about_window {
            about::show_about_window(ctx, &mut self.show_about_window);
        }
    }
}
