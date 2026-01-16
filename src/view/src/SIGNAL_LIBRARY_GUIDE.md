// 示例：如何在 CanViewApp 中使用信号库管理功能

// 1. 在 main.rs 中添加模块声明
mod library;

// 2. 导入必要的类型
use crate::library::{LibraryManager, DatabaseValidation, Database};
use crate::models::{SignalLibrary, LibraryVersion, ChannelType};

// 3. 在 CanViewApp 中添加库管理相关字段
impl CanViewApp {
    // 添加到结构体定义中：
    // pub library_manager: LibraryManager,
    // pub selected_library_id: Option<String>,
    // pub show_library_dialog: bool,
    // pub new_library_name: SharedString,

    /// 创建新库
    fn create_library(&mut self, name: String, channel_type: ChannelType, cx: &mut Context<Self>) {
        match self.library_manager.create_library(name.clone(), channel_type) {
            Ok(_) => {
                self.status_msg = format!("Created library '{}'", name).into();
                self.selected_library_id = self.library_manager.find_library_id(&name);
                self.save_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to create library: {}", e).into();
            }
        }
        cx.notify();
    }

    /// 删除库
    fn delete_library(&mut self, library_id: &str, cx: &mut Context<Self>) {
        match self.library_manager.delete_library(library_id, &self.app_config.mappings) {
            Ok(_) => {
                self.status_msg = "Library deleted successfully".into();
                if self.selected_library_id.as_ref() == Some(library_id) {
                    self.selected_library_id = None;
                }
                self.save_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to delete library: {}", e).into();
            }
        }
        cx.notify();
    }

    /// 添加新版本
    fn add_library_version(
        &mut self,
        library_id: &str,
        name: String,
        path: String,
        description: String,
        cx: &mut Context<Self>
    ) {
        match self.library_manager.add_version(library_id, name.clone(), path.clone(), description) {
            Ok(_) => {
                // 验证数据库
                match self.library_manager.validate_database(&path) {
                    Ok(validation) => {
                        let msg = if validation.is_valid {
                            format!(
                                "Added version {} - {} messages, {} signals",
                                name, validation.message_count, validation.signal_count
                            )
                        } else {
                            format!("Added version {} (with warnings)", name)
                        };
                        self.status_msg = msg.into();
                    }
                    Err(e) => {
                        self.status_msg = format!("Added version {} (validation failed: {})", name, e).into();
                    }
                }
                self.save_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to add version: {}", e).into();
            }
        }
        cx.notify();
    }

    /// 删除版本
    fn delete_library_version(&mut self, library_id: &str, version_name: &str, cx: &mut Context<Self>) {
        match self.library_manager.remove_version(library_id, version_name, &self.app_config.mappings) {
            Ok(_) => {
                self.status_msg = format!("Deleted version {}", version_name).into();
                self.save_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to delete version: {}", e).into();
            }
        }
        cx.notify();
    }

    /// 加载并激活库版本
    fn load_library_version(&mut self, library_id: &str, version_name: &str, cx: &mut Context<Self>) {
        // 查找库和版本
        let library = match self.library_manager.find_library(library_id) {
            Some(lib) => lib,
            None => {
                self.status_msg = "Library not found".into();
                cx.notify();
                return;
            }
        };

        let version = match library.get_version(version_name) {
            Some(ver) => ver,
            None => {
                self.status_msg = "Version not found".into();
                cx.notify();
                return;
            }
        };

        // 加载数据库
        match self.library_manager.load_database(&version.path, library.channel_type) {
            Ok(db) => {
                match db {
                    Database::Dbc(dbc_db) => {
                        // 加载到所有CAN通道
                        for channel in 1..=16u16 {
                            self.dbc_channels.insert(channel, dbc_db.clone());
                        }
                    }
                    Database::Ldf(ldf_db) => {
                        // 加载到所有LIN通道
                        for channel in 1..=16u16 {
                            self.ldf_channels.insert(channel, ldf_db.clone());
                        }
                    }
                }

                // 更新激活状态
                self.app_config.active_library_id = Some(library_id.to_string());
                self.app_config.active_version_name = Some(version_name.to_string());

                self.status_msg = format!(
                    "Loaded {} v{}",
                    library.name,
                    version_name
                ).into();

                self.save_config(cx);
            }
            Err(e) => {
                self.status_msg = format!("Failed to load database: {}", e).into();
            }
        }
        cx.notify();
    }

    /// 验证数据库文件
    fn validate_database_file(&mut self, path: String) {
        match self.library_manager.validate_database(&path) {
            Ok(validation) => {
                if validation.is_valid {
                    self.status_msg = format!(
                        "Valid - {} messages, {} signals",
                        validation.message_count,
                        validation.signal_count
                    ).into();
                } else {
                    self.status_msg = format!("Invalid: {:?}", validation.error).into();
                }
            }
            Err(e) => {
                self.status_msg = format!("Validation error: {}", e).into();
            }
        }
    }

    /// 获取数据库统计信息
    fn get_database_stats(&self, path: String) -> Option<crate::library::DatabaseStats> {
        self.library_manager.get_database_stats(&path).ok()
    }

    /// 导入数据库文件并添加为版本
    fn import_database_as_version(&mut self, cx: &mut Context<Self>) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Database Files", &["dbc", "ldf"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();

            // 如果有选中的库，直接添加版本
            if let Some(lib_id) = &self.selected_library_id {
                // 自动生成版本号
                use crate::library::extract_version_from_path;
                let version_name = extract_version_from_path(&path);

                self.add_library_version(
                    lib_id,
                    version_name,
                    path_str,
                    String::new(),
                    cx
                );
            } else {
                self.status_msg = "Please select a library first".into();
            }
        }
    }
}

// 4. 在配置视图中渲染库管理界面
impl CanViewApp {
    fn render_library_management(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .gap_4()
            .child(
                // 左侧：库列表
                div()
                    .w(px(300.))
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("Libraries")
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(rgb(0x3b82f6))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x2563eb)))
                                    .text_color(rgb(0xffffff))
                                    .text_sm()
                                    .child("+ New")
                                    .on_click(cx.listener(|this, _event, cx| {
                                        this.show_library_dialog = true;
                                        cx.notify();
                                    }))
                            )
                    )
                    .child(
                        self.library_manager.libraries(),
                        &self.selected_library_id,
                        &self.app_config.mappings,
                        cx
                    )
            )
            .child(
                // 右侧：版本详情
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .child("Versions")
                    )
                    .child({
                        if let Some(lib_id) = &self.selected_library_id {
                            if let Some(library) = self.library_manager.find_library(lib_id) {
                                render_version_details(library, &self.app_config.mappings, cx)
                            } else {
                                div()
                            }
                        } else {
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0x6b7280))
                                        .child("Select a library to view versions")
                                )
                        }
                    })
            )
    }
}

// 5. 配置持久化
impl CanViewApp {
    fn save_library_config(&self, cx: &mut Context<Self>) {
        // 将库管理器的状态保存到app_config
        self.app_config.libraries = self.library_manager.libraries().to_vec();
        self.save_config(cx);
    }

    fn load_library_config(&mut self) {
        // 从app_config加载到库管理器
        self.library_manager = LibraryManager::from_libraries(
            self.app_config.libraries.clone()
        );
    }
}

// 6. 示例：完整的库管理工作流程

// 6.1 创建CAN库
// this.create_library("Engine CAN".to_string(), ChannelType::CAN, cx);

// 6.2 添加DBC文件作为版本
// this.add_library_version(
//     library_id,
//     "v1.0".to_string(),
//     "/path/to/engine.dbc".to_string(),
//     "Initial version".to_string(),
//     cx
// );

// 6.3 激活版本（加载数据库）
// this.load_library_version(library_id, "v1.0", cx);

// 6.4 添加新版本
// this.add_library_version(
//     library_id,
//     "v2.0".to_string(),
//     "/path/to/engine_v2.dbc".to_string(),
//     "Updated signal definitions".to_string(),
//     cx
// );

// 6.5 切换版本
// this.load_library_version(library_id, "v2.0", cx);

// 6.6 删除旧版本（如果未被使用）
// this.delete_library_version(library_id, "v1.0", cx);

// 7. 高级功能

/// 批量导入多个库
fn import_multiple_libraries(&mut self, cx: &mut Context<Self>) {
    let libraries_to_create = vec![
        ("Engine CAN", ChannelType::CAN),
        ("Chassis CAN", ChannelType::CAN),
        ("Body LIN", ChannelType::LIN),
    ];

    for (name, channel_type) in libraries_to_create {
        if let Ok(_) = self.library_manager.create_library(name.to_string(), channel_type) {
            self.status_msg = format!("Created library '{}'", name).into();
        }
    }

    self.save_config(cx);
    cx.notify();
}

/// 克隆版本（创建副本）
fn clone_library_version(&mut self, library_id: &str, version_name: &str, cx: &mut Context<Self>) {
    if let Some(library) = self.library_manager.find_library(library_id) {
        if let Some(version) = library.get_version(version_name) {
            let new_name = format!("{}_copy", version_name);
            self.add_library_version(
                library_id,
                new_name,
                version.path.clone(),
                format!("Copy of {}", version_name),
                cx
            );
        }
    }
}

/// 导出库配置
fn export_library_config(&self, library_id: &str) -> Result<String, String> {
    let library = self.library_manager.find_library(library_id)
        .ok_or("Library not found")?;

    serde_json::to_string_pretty(library)
        .map_err(|e| format!("Failed to serialize: {}", e))
}
