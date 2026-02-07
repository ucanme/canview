# Domain 层使用指南

## 快速开始

Domain层包含纯业务逻辑，可以在任何地方使用，无需依赖GPUI。

### 1. 时间处理

```rust
use crate::domain::{TimeHandler, TimestampFormatter, TimestampFormat};

// 创建时间处理器
let mut handler = TimeHandler::new();
handler.set_start_time(start_time);

// 格式化时间戳
let formatted = handler.format_timestamp(nanos);
println!("时间: {}", formatted);

// 使用不同的格式化器
let formatter = TimestampFormatter::new(TimestampFormat::HMS);
let time_str = formatter.format(123.456, None);
```

### 2. 日志处理

```rust
use crate::domain::{LogProcessor, MessageFilter};

// 创建日志处理器
let mut processor = LogProcessor::new();
processor.add_messages(messages);

// 应用过滤器
let filter = MessageFilter::new()
    .with_id(0x123)
    .with_channel(1);
let filtered = processor.apply_filter(filter);

// 获取统计信息
let stats = processor.calculate_statistics();
println!("总消息数: {}", stats.total_messages);
println!("CAN消息数: {}", stats.can_messages);

// 获取唯一ID列表
let unique_ids = processor.unique_can_ids();
```

### 3. 信号解码

```rust
use crate::domain::SignalDecoder;

// 创建解码器
let mut decoder = SignalDecoder::new();
decoder.add_dbc_channel(1, dbc_database);

// 解码CAN消息
let signals = decoder.decode_can_message(
    channel,
    can_id,
    &data,
    timestamp
);

for signal_value in signals {
    println!("{}: {}", 
        signal_value.signal.name,
        signal_value.signal.physical_value
    );
}
```

### 4. 配置管理

```rust
use crate::domain::ConfigManager;

// 创建配置管理器
let mut manager = ConfigManager::new(config_dir);
manager.load()?;

// 获取配置
let config = manager.config();

// 修改配置
manager.config_mut().add_channel(channel_config);
manager.save()?;

// 导入/导出
manager.import_from_file(path)?;
manager.export_to_file(export_path)?;
```

## 编写测试

Domain层的代码都是纯逻辑，很容易测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_log_processor() {
        let mut processor = LogProcessor::new();
        processor.add_messages(test_messages());
        
        assert_eq!(processor.message_count(), 10);
        
        let filtered = processor.apply_filter(
            MessageFilter::new().with_id(0x123)
        );
        assert!(filtered.len() > 0);
    }
}
```

## 运行测试

```bash
# 运行所有Domain层测试
cargo test -p view domain

# 运行特定模块的测试
cargo test -p view time_handler
cargo test -p view log_processor
cargo test -p view signal_decoder
cargo test -p view config_manager

# 查看测试输出
cargo test -p view domain -- --nocapture
```

## 下一步

- 查看每个模块的文档注释了解详细API
- 阅读单元测试了解使用示例
- 在你的代码中使用这些Domain层API
