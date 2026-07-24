# StatusBar 多文件展示重设计

## 背景

当前 StatusBar 左侧文件段只显示一个文件名(`current_file_name`,最近加载的),
多文件场景下其他文件只能通过右侧的 📁 Files(N) 按钮打开 popover 查看。问题:

- 左侧的"📂 文件名"在多文件时语义模糊,用户以为只有一个文件
- 右侧 Files(N) 是唯一入口,数字小、视觉权重低,容易被忽略
- 入口和数据展示分离,信息架构不直观

## 目标

- 多文件时,左侧文件段直接展示"N files"计数,信息更直观
- 左侧文件段本身可点击,作为打开 files popover 的唯一入口
- 解析失败时,左侧文件段追加黄色 ⚠️ 提示,与文件段一起点击打开 files popover(里面已显示每文件状态和错误详情)
- 移除右侧 Files 按钮和 status_msg 的 "› details" 链接,避免重复入口(函数保留以便未来恢复)

## 设计

### 左侧文件段(`render_file_segment`)新行为

按 `app.files.len()` × `app.blf_parse_errors.is_empty()` 分五种状态:

| 文件数 | 有解析错误 | 展示 | 可点击 |
|---|---|---|---|
| 0 | - | `📂 No file loaded — File > Open BLF...` | 否 |
| 1 | 否 | `📂 <file_name>` | 否 |
| 1 | 是 | `📂 <file_name> ⚠️` | 是 |
| ≥2 | 否 | `📂 N files` | 是 |
| ≥2 | 是 | `📂 N files ⚠️` | 是 |

⚠️ 样式:`text_color(colors::WARNING)`,与文件段整体一起渲染、一起点击。

可点击时的样式:
- `cursor_pointer`
- hover 反馈:`bg(colors::SURFACE0)`
- `on_mouse_down(MouseButton::Left, ...)` 中调用 `view_for_click.update(cx, |app, cx| { app.show_files_popover = !app.show_files_popover; cx.notify(); })`
- `cx.stop_propagation()` 防止事件冒泡到上层

签名变更:从 `fn(&App) -> impl IntoElement` 改为 `fn(&App, Entity<App>) -> impl IntoElement`(需要 view 来注册点击回调)。

### 右侧 Files 按钮(`render_files_button_segment`)

- 保留函数(便于未来快速恢复)
- 把原来的 `if app.files.is_empty() { return None; }` 改为 `return None;`(永远返回 None)
- 注释说明:多文件入口已迁移到左侧 `render_file_segment`
- `render_status_bar` 调用处的 `when_some(render_files_button_segment(...), |el, files_btn| { el.child(render_separator()).child(files_btn) })` 自然跳过

### 右侧 status_msg 段(`render_status_msg_segment`)

- 移除末尾的 "› details" 可点击链接(对应 `has_errors` 分支整体删除)
- 函数签名从 `fn(&App, Entity<App>) -> Option<impl IntoElement>` 改为 `fn(&App) -> Option<impl IntoElement>`(不再需要 view)
- `render_status_bar` 调用处相应改为 `render_status_msg_segment(app)`

### Popover(`render_files_popover`)

不动。已实现的滚动 + 动态宽度 + 紧凑高度逻辑保留:
- 1-8 行:高度随内容收缩,无滚动条
- 9+ 行:固定 8 行高度,出滚动条
- 宽度按文件名长度估算,夹在 [280, 460] 之间

唯一改变是触发入口:从右侧 Files 按钮变成左侧文件段。

## 影响范围

1. `src/view/src/ui/components/status_bar.rs`
   - `render_file_segment`:签名从 `fn(&App) -> impl IntoElement` 改为 `fn(&App, Entity<App>) -> impl IntoElement`(需要 view 来注册点击回调);加条件分支;有错时追加 ⚠️;多文件或有错时加 cursor/hover/click
   - `render_status_bar`:左侧 `render_file_segment(app)` 调用改为 `render_file_segment(app, view.clone())`
   - `render_files_button_segment`:首行 `if app.files.is_empty() { return None; }` 改为 `return None;`,更新注释
   - `render_status_msg_segment`:删除 `has_errors` 分支和 "› details" 链接,签名改为 `fn(&App) -> Option<impl IntoElement>`
   - `render_status_bar` 中右侧 status_msg 调用从 `render_status_msg_segment(app, view.clone())` 改为 `render_status_msg_segment(app)`

2. 不动:`render_files_popover`、`render_status_bar_popovers`、`render_blf_errors_popover`(虽然 status_msg 不再触发它,但 popover 函数本身保留)、状态字段 `show_files_popover`、`show_blf_errors_popover`

## 验证

手动测试:
1. 加载 1 个文件无错误:左侧显示 `?? <file_name>`,无 ⚠️,无可点击样式,右侧无 Files 按钮
2. 加载 1 个文件有错误:左侧显示 `?? <file_name> ⚠️`(⚠️ 黄色),点击打开 files popover,里面看到该文件的 ❌ 状态图标和错误列表
3. 加载 9 个文件无错误:左侧显示 `?? 9 files`,鼠标 hover 有底色反馈,点击打开 popover
4. 加载 9 个文件有错误:左侧显示 `?? 9 files ⚠️`,点击打开 popover,滚动条正常,Remove All / Done 可见
5. 关闭 popover:点击左侧文件段再次切换,点击 popover 外部也可关闭
6. 右侧 status_msg 不再有 "› details" 链接

## 不在本次范围

- Popover 内部布局调整
- 文件名截断策略调整
- 拖拽排序文件顺序
- 右键菜单
- 移除 `blf_errors_popover`(本次仅删除入口,函数保留)
