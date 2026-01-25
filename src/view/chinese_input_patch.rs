// 直接可用的中文输入补丁
//
// 将这个代码复制到 library_view.rs 中的输入框部分

// 在 render_library_header 函数中，找到输入框的 div()
// 它应该有 .on_key_down({ ...