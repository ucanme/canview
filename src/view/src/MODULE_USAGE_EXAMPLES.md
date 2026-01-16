// 示例：如何在现有 main.rs 中使用新模块

// 1. 在 main.rs 顶部添加模块声明
mod models;
mod config;
mod filters;

// 2. 导入需要的类型和函数
use models::{ChannelType, AppConfig, AppView};
use filters::{filter_by_id, get_unique_ids, format_id};

// 3. 替换原有的数据结构定义
// 之前在 main.rs 中定义的这些结构现在应该删除，因为它们在 models 模块中：
// - enum ChannelType { CAN, LIN }
// - struct ChannelMapping { ... }
// - struct AppConfig { ... }
// - enum AppView { LogView, ConfigView, ChartView }

// 4. 在代码中使用模块中的函数

// 示例：在 CanViewApp 实现中使用过滤函数
impl CanViewApp {
    fn get_filtered_messages(&self) -> Vec<LogObject> {
        match (self.id_filter, self.channel_filter) {
            (None, None) => self.messages.clone(),
            (Some(filter_id), None) => filter_by_id(&self.messages, filter_id),
            (None, Some(filter_ch)) => {
                // 可以添加 filter_by_channel 函数到 filters 模块
                self.messages.iter()
                    .filter(|msg| msg.channel() == filter_ch)
                    .cloned()
                    .collect()
            }
            (Some(filter_id), Some(filter_ch)) => {
                // 先按ID过滤，再按通道过滤
                let id_filtered = filter_by_id(&self.messages, filter_id);
                id_filtered.into_iter()
                    .filter(|msg| msg.channel() == filter_ch)
                    .collect()
            }
        }
    }

    // 示例：使用 format_id 函数
    fn render_id(&self, id: u32) -> String {
        format_id(id, self.id_display_decimal)
    }

    // 示例：使用 get_unique_ids 函数
    fn get_available_filters(&self) -> Vec<u32> {
        get_unique_ids(&self.messages)
    }
}

// 5. 配置管理示例
impl CanViewApp {
    fn save_my_config(&self) {
        use config::save_config_to_path;
        use std::path::PathBuf;

        let config_path = PathBuf::from("my_config.json");
        if let Err(e) = save_config_to_path(&self.app_config, &config_path) {
            self.status_msg = format!("Failed to save: {}", e).into();
        } else {
            self.status_msg = "Configuration saved!".into();
        }
    }

    fn load_my_config(&mut self) {
        use config::load_config_from_path;

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Config Files", &["json"])
            .pick_file()
        {
            match load_config_from_path(path.clone()) {
                Ok(config) => {
                    self.app_config = config;
                    self.status_msg = "Config loaded successfully".into();
                }
                Err(e) => {
                    self.status_msg = format!("Load error: {}", e).into();
                }
            }
        }
    }
}

// 6. 创建新的配置示例
fn create_example_config() -> AppConfig {
    use models::ChannelMapping;

    AppConfig {
        mappings: vec![
            ChannelMapping {
                channel_type: ChannelType::CAN,
                channel_id: 1,
                path: "/path/to/engine.dbc".to_string(),
                description: "Engine CAN Bus".to_string(),
            },
            ChannelMapping {
                channel_type: ChannelType::LIN,
                channel_id: 2,
                path: "/path/to/lin.ldf".to_string(),
                description: "LIN Network".to_string(),
            },
        ],
    }
}

// 7. 使用模型类型
fn process_channel_type(channel_type: &ChannelType) -> String {
    match channel_type {
        ChannelType::CAN => "CAN Bus".to_string(),
        ChannelType::LIN => "LIN Network".to_string(),
    }
}

// 8. 测试模块功能的辅助函数
#[cfg(test)]
mod tests {
    use super::*;
    use models::*;

    #[test]
    fn test_channel_type() {
        let can = ChannelType::CAN;
        assert!(can.is_can());
        assert!(!can.is_lin());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config);
        assert!(json.is_ok());
    }
}
