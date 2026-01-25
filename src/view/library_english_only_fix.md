// 修改 library_view.rs 中的输入框部分

// 在 render_library_header 函数中，找到输入框的占位符文本
// 将 "Library name..." 改为：

.child(
    div()
        .text_xs()
        .text_color(rgb(0x646473))
        .child("Library name (English only)...")  // ← 修改这里
)

// 同时，在点击 "+ New" 按钮时，添加提示
// 可以在状态栏显示一个消息：

// 在 CanViewApp 的状态中添加：
// pub struct CanViewApp {
//     ...
//     pub status_msg: Option<String>,
//     ...
// }

// 然后在创建输入框时：
view.update(cx, |this, cx| {
    this.new_library_name = String::from(" ");
    this.library_cursor_position = 1;
    this.status_msg = Some("注意：当前版本仅支持英文名称。中文支持正在开发中。".to_string());
    cx.notify();
});
