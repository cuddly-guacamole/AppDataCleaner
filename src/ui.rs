use crate::about;
use crate::confirmation;
use crate::delete;
use crate::scanner;
use crate::utils;
use eframe::egui::{self, Grid, ScrollArea};

pub struct AppDataCleaner {
    is_scanning: bool,
    current_folder: Option<String>,
    folder_data: Vec<(String, u64)>,
    show_about_window: bool,                // 确保字段存在
    confirm_delete: Option<(String, bool)>, // 保存要确认删除的文件夹状态
}

impl Default for AppDataCleaner {
    fn default() -> Self {
        Self {
            is_scanning: false,
            current_folder: None,
            folder_data: vec![],
            show_about_window: false, // 默认值
            confirm_delete: None,     // 初始化为 None
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

        // 顶部菜单
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.menu_button("菜单", |ui| {
                if ui.button("关于").clicked() {
                    self.show_about_window = true; // 打开关于窗口
                    ui.close_menu();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("立即扫描").clicked() && !self.is_scanning {
                self.is_scanning = true;
                self.folder_data.clear();

                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || scanner::scan_appdata(tx));

                while let Ok((folder, size)) = rx.recv() {
                    self.folder_data.push((folder, size));
                }

                self.is_scanning = false;
                self.current_folder = None;
            }

            if self.is_scanning {
                ui.label("扫描中...");
            }

            ScrollArea::vertical().show(ui, |ui| {
                Grid::new("folders_table").striped(true).show(ui, |ui| {
                    ui.label("文件夹");
                    ui.label("大小");
                    ui.label("使用软件");
                    ui.label("操作");
                    ui.end_row();

                    for (folder, size) in &self.folder_data {
                        ui.label(folder);
                        ui.label(utils::format_size(*size));
                        ui.label("未知");

                        if ui.button("彻底删除").clicked() {
                            self.confirm_delete = Some((folder.clone(), false));
                            let folder_name = folder.clone();
                            let full_path = format!("{}/{}", utils::get_appdata_dir(), folder_name);
                            if let Err(err) = delete::delete_folder(&full_path) {
                                eprintln!("Error: {}", err);
                            }
                        }
                        if ui.button("移动").clicked() {
                            // 移动逻辑
                        }
                        ui.end_row();
                    }
                });
            });
        });

        // 关于窗口
        if self.show_about_window {
            about::show_about_window(ctx, &mut self.show_about_window);
        }
    }
}
