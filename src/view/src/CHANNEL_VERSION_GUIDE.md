// 通道配置版本管理 - UI 实现示例

// 这个文件展示了如何实现支持多通道配置的版本管理UI

use gpui::*;
use crate::app::{CanViewApp, ChannelConfig};
use crate::models::{ChannelDatabase, DatabaseType};

// ==================== 1. 添加版本对话框（带通道配置） ====================

impl CanViewApp {
    /// 渲染添加版本对话框（增强版 - 支持通道配置）
    pub fn render_add_version_dialog_enhanced(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .inset_0()
            .bg(rgb(0x000000).with_alpha(0.5))
            .flex()
            .items_center()
            .justify_center()
            .z_index(1000)
            .child(
                div()
                    .w(px(800.))
                    .max_h(px(700.))
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(12.))
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .overflow_hidden()
                    .child(
                        // 标题
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .child("Add New Version")
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .bg(rgb(0xef4444))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0xdc2626)))
                                    .text_color(rgb(0xffffff))
                                    .text_sm()
                                    .child("✕")
                                    .on_click(cx.listener(|this, _event, cx| {
                                        this.close_add_channel_dialog(cx);
                                    }))
                            )
                    )
                    .child(
                        // 版本基本信息
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(self.render_version_input_form(cx))
                    )
                    .child(
                        // 通道配置列表
                        div()
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
                                            .child("Channel Configuration")
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .gap_2()
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
                                                    .child("+ Add Channel")
                                                    .on_click(cx.listener(|this, _event, cx| {
                                                        this.open_add_channel_dialog(cx);
                                                    }))
                                            )
                                            .when(!self.channel_configs.is_empty(), |div| {
                                                div.child(
                                                    div()
                                                        .px_3()
                                                        .py_1()
                                                        .bg(rgb(0xef4444))
                                                        .rounded(px(4.))
                                                        .cursor_pointer()
                                                        .hover(|style| style.bg(rgb(0xdc2626)))
                                                        .text_color(rgb(0xffffff))
                                                        .text_sm()
                                                        .child("Clear All")
                                                        .on_click(cx.listener(|this, _event, cx| {
                                                            this.clear_channel_configs(cx);
                                                        }))
                                                )
                                            })
                                    )
                            )
                            .child(self.render_channel_list(cx))
                    )
                    .child(
                        // 底部按钮
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(0x10b981))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x059669)))
                                    .text_color(rgb(0xffffff))
                                    .child("Save Version")
                                    .on_click(cx.listener(|this, _event, cx| {
                                        this.add_version_with_channels(cx);
                                    }))
                            )
                    )
            )
    }

    /// 渲染版本输入表单
    fn render_version_input_form(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            // 版本名称
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child("Version Name *")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .bg(rgb(0x374151))
                            .border_1()
                            .border_color(rgb(0x2a2a2a))
                            .rounded(px(4.))
                            .text_color(rgb(0xffffff))
                            .child(self.new_version_name.clone())
                    )
            )
            // 版本描述
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(rgb(0xffffff))
                            .child("Description")
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .bg(rgb(0x374151))
                            .border_1()
                            .border_color(rgb(0x2a2a2a))
                            .rounded(px(4.))
                            .text_color(rgb(0xffffff))
                            .child(self.new_version_description.clone())
                    )
            )
    }

    /// 渲染通道列表
    fn render_channel_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        if self.channel_configs.is_empty() {
            return div()
                .p_6()
                .bg(rgb(0x374151))
                .border_1()
                .border_color(rgb(0x2a2a2a))
                .rounded(px(8.))
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x9ca3af))
                        .child("No channels configured yet")
                );
        }

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .max_h(px(300.))
            .overflow_y_scroll()
            .children(
                self.channel_configs.iter().enumerate().map(|(index, config)| {
                    self.render_channel_item(index, config, cx)
                })
            )
    }

    /// 渲染单个通道项
    fn render_channel_item(&self, index: usize, config: &ChannelConfig, cx: &mut Context<Self>) -> impl IntoElement {
        let file_type = std::path::Path::new(&config.file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("unknown");

        div()
            .px_4()
            .py_3()
            .bg(rgb(0x1f1f1f))
            .border_1()
            .border_color(rgb(0x2a2a2a))
            .rounded(px(8.))
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .bg(if file_type == "dbc" { rgb(0x3b82f6) } else { rgb(0x10b981) })
                            .rounded(px(3.))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .child(config.channel_id.to_uppercase())
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xffffff))
                                    .child(format!("Channel {} - {}", config.channel_id, config.channel_name))
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x9ca3af))
                                    .child(
                                        config.file_path
                                            .split('\\')
                                            .last()
                                            .unwrap_or(&config.file_path)
                                            .to_string()
                                    )
                            )
                    )
            )
            .child(
                div()
                    .px_2()
                    .py_1()
                    .bg(rgb(0xef4444))
                    .rounded(px(3.))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0xdc2626)))
                    .text_color(rgb(0xffffff))
                    .text_xs()
                    .child("Remove")
                    .on_click(cx.listener(move |this, _event, cx| {
                        this.remove_channel_config(index, cx);
                    }))
            )
    }

    /// 渲染添加通道对话框
    pub fn render_add_channel_dialog(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .inset_0()
            .bg(rgb(0x000000).with_alpha(0.5))
            .flex()
            .items_center()
            .justify_center()
            .z_index(1100)
            .child(
                div()
                    .w(px(500.))
                    .bg(rgb(0x1f1f1f))
                    .border_1()
                    .border_color(rgb(0x2a2a2a))
                    .rounded(px(12.))
                    .p_6()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .child("Add Channel")
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            // 通道ID
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .child("Channel ID *")
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .bg(rgb(0x374151))
                                            .border_1()
                                            .border_color(rgb(0x2a2a2a))
                                            .rounded(px(4.))
                                            .text_color(rgb(0xffffff))
                                            .child(
                                                if self.new_channel_id.is_empty() {
                                                    "Enter channel ID (1-255)".into()
                                                } else {
                                                    self.new_channel_id.clone().into()
                                                }
                                            )
                                    )
                            )
                            // 通道名称
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .child("Channel Name *")
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .bg(rgb(0x374151))
                                            .border_1()
                                            .border_color(rgb(0x2a2a2a))
                                            .rounded(px(4.))
                                            .text_color(rgb(0xffffff))
                                            .child(
                                                if self.new_channel_name.is_empty() {
                                                    "Enter channel name".into()
                                                } else {
                                                    self.new_channel_name.clone().into()
                                                }
                                            )
                                    )
                            )
                            // 数据库文件
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xffffff))
                                            .child("Database File *")
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .px_3()
                                                    .py_2()
                                                    .bg(rgb(0x374151))
                                                    .border_1()
                                                    .border_color(rgb(0x2a2a2a))
                                                    .rounded(px(4.))
                                                    .text_color(rgb(0x9ca3af))
                                                    .child(
                                                        if self.new_channel_file.is_empty() {
                                                            "Select database file".into()
                                                        } else {
                                                            std::path::Path::new(&self.new_channel_file)
                                                                .file_name()
                                                                .and_then(|n| n.to_str())
                                                                .unwrap_or("Unknown file")
                                                                .to_string()
                                                                .into()
                                                        }
                                                    )
                                            )
                                            .child(
                                                div()
                                                    .px_3()
                                                    .py_2()
                                                    .bg(rgb(0x3b82f6))
                                                    .rounded(px(4.))
                                                    .cursor_pointer()
                                                    .hover(|style| style.bg(rgb(0x2563eb)))
                                                    .text_color(rgb(0xffffff))
                                                    .text_sm()
                                                    .child("Browse")
                                                    .on_click(cx.listener(|this, _event, cx| {
                                                        this.browse_channel_database_file(cx);
                                                    }))
                                            )
                                    )
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .justify_end()
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(0x6b7280))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x4b5563)))
                                    .text_color(rgb(0xffffff))
                                    .child("Cancel")
                                    .on_click(cx.listener(|this, _event, cx| {
                                        this.close_add_channel_dialog(cx);
                                    }))
                            )
                            .child(
                                div()
                                    .px_4()
                                    .py_2()
                                    .bg(rgb(0x10b981))
                                    .rounded(px(4.))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x059669)))
                                    .text_color(rgb(0xffffff))
                                    .child("Add")
                                    .on_click(cx.listener(|this, _event, cx| {
                                        this.add_channel_config(cx);
                                    }))
                            )
                    )
            )
    }
}

// ==================== 2. 使用示例 ====================

// 在配置视图中集成
impl CanViewApp {
    pub fn render_library_management_with_channels(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_4()
            // 库列表
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xffffff))
                    .child("Signal Libraries")
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgb(0x3b82f6))
                            .rounded(px(4.))
                            .cursor_pointer()
                            .text_color(rgb(0xffffff))
                            .text_sm()
                            .child("+ Add Version with Channels")
                            .on_click(cx.listener(|this, _event, cx| {
                                // 打开增强的添加版本对话框
                                // this.open_add_version_dialog_enhanced(cx);
                            }))
                    )
            )
            // 显示通道配置统计
            .when(!self.channel_configs.is_empty(), |div| {
                div.child(
                    div()
                        .px_4()
                        .py_2()
                        .bg(rgb(0x374151))
                        .rounded(px(8.))
                        .text_color(rgb(0xffffff))
                        .text_sm()
                        .child(format!(
                            "Configured: {} channel(s)",
                            self.channel_configs.len()
                        ))
                )
            })
            // 显示添加通道对话框
            .when(self.show_add_channel_dialog, |div| {
                div.child(self.render_add_channel_dialog(cx))
            })
    }
}

// ==================== 3. 完整工作流示例 ====================

// 步骤1: 打开添加版本对话框
// app.open_add_version_dialog(cx);

// 步骤2: 填写版本信息
// app.new_version_name = "v1.0";
// app.new_version_description = "Initial version";

// 步骤3: 点击"Add Channel"按钮
// app.open_add_channel_dialog(cx);

// 步骤4: 在弹出的对话框中填写通道信息：
// - Channel ID: 1
// - Channel Name: "Engine CAN"
// - 点击Browse选择DBC文件: /path/to/engine.dbc
// - 点击Add

// 步骤5: 继续添加更多通道（如果需要）
// app.open_add_channel_dialog(cx);
// - Channel ID: 2
// - Channel Name: "Chassis CAN"
// - 选择DBC文件: /path/to/chassis.dbc
// - 点击Add

// 步骤6: 点击"Save Version"保存版本
// app.add_version_with_channels(cx);

// 结果：创建了一个包含两个CAN通道配置的版本

// ==================== 4. 配置示例 ====================

// JSON 配置示例
/*
{
  "libraries": [
    {
      "id": "lib_abc123",
      "name": "Vehicle CAN Database",
      "channel_type": "CAN",
      "versions": [
        {
          "name": "v1.0",
          "path": "/path/to/engine.dbc",
          "date": "2024-01-15",
          "description": "Multi-channel version",
          "channel_databases": [
            {
              "channel_id": 1,
              "channel_name": "Engine CAN",
              "database_path": "/path/to/engine.dbc"
            },
            {
              "channel_id": 2,
              "channel_name": "Chassis CAN",
              "database_path": "/path/to/chassis.dbc"
            },
            {
              "channel_id": 3,
              "channel_name": "Body CAN",
              "database_path": "/path/to/body.dbc"
            }
          ]
        }
      ]
    }
  ]
}
*/

// ==================== 5. 验证和错误处理 ====================

impl CanViewApp {
    /// 验证通道配置
    pub fn validate_channel_config(&self, config: &ChannelConfig) -> Result<(), String> {
        // 检查通道ID
        let channel_id = config.channel_id.parse::<u16>()
            .map_err(|_| "Invalid channel ID format".to_string())?;

        if channel_id == 0 || channel_id > 255 {
            return Err("Channel ID must be between 1 and 255".to_string());
        }

        // 检查通道名称
        if config.channel_name.trim().is_empty() {
            return Err("Channel name cannot be empty".to_string());
        }

        // 检查文件路径
        if config.file_path.trim().is_empty() {
            return Err("Database file path cannot be empty".to_string());
        }

        // 检查文件是否存在
        if !std::path::Path::new(&config.file_path).exists() {
            return Err(format!("File not found: {}", config.file_path));
        }

        Ok(())
    }

    /// 检查通道ID冲突
    pub fn check_channel_conflicts(&self, channel_id: &str) -> bool {
        self.channel_configs.iter().any(|c| c.channel_id == channel_id)
    }

    /// 获取通道类型统计
    pub fn get_channel_type_stats(&self) -> (usize, usize) {
        let mut can_count = 0;
        let mut lin_count = 0;

        for config in &self.channel_configs {
            let ext = std::path::Path::new(&config.file_path)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            if ext.eq_ignore_ascii_case("dbc") {
                can_count += 1;
            } else if ext.eq_ignore_ascii_case("ldf") {
                lin_count += 1;
            }
        }

        (can_count, lin_count)
    }
}
