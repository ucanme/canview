# 多文件解析与回放 — Design Spec

- **日期**:2026-07-24
- **目标**:支持同时加载多个 BLF 文件,按原始绝对时间全局合并,UI 提供折线图展示和可滚动的日志列表
- **范围**:`view` crate 的 `CanViewApp` 状态结构 + File 菜单 + StatusBar + 文件管理弹框;`blf` crate 不变,复用 `read_blf_from_file`

---

## 0. 需求与决定

| 维度 | 决定 |
|---|---|
| 核心场景 | 多文件合并视图(按原始绝对时间全局排序) |
| "回放"含义 | 折线图展示 + 日志列表可滚动;无播放头/时间游标动画 |
| 时间轴基准 | 原始绝对时间(`measurement_start_time + object_time_stamp`) |
| 打开方式 | 多选 + 追加 |
| 单选 vs 多选语义 | 单选(`Open BLF...`)= 替换;多选(`Open Multiple BLF...`)= 追加 |
| 追加入口 | 只保留多选追加(`Open Multiple BLF...`),不单独提供 `Add BLF...` 单选追加 |
| 性能 | 并行解析 + 流式加载,最高优先 |
| 文件管理 UI | StatusBar 入口 + 弹框管理已加载文件 |
| 信号区分 | 不区分来源,同名信号直接合并拼接为一条波形 |
| 文件路径持久化 | 不保存到 `multi_channel_config.json`,重启后 `files` 为空 |
| `is_streaming_mode` | 删除(未使用) |

---

## 1. 整体架构

引入"文件段 + 全局合并视图"两层结构,对 UI 大部分代码透明(仍以 `&[LogObject]` 访问)。

```
用户多选 N 个 BLF 文件
  → File → Open Multiple BLF...
  → LoadBlfFiles { paths, mode: Append }
  → 对每个 path spawn 后台任务 read_blf_from_file (并发,GPUI background_executor)
  → 每个任务完成:
      → cx.update 把 FileSegment 推进 app.files
      → 重建 app.merged (增量 k-way merge,不整体重排序)
      → StatusBar 流式更新 "Loaded K/N files, M messages"
  → 所有任务完成:
      → StatusBar "✅ Loaded N files, total M messages"
      → 文件管理 popover 可查看/移除每个文件
  → LogView / Signal Plot 从 app.merged.messages 渲染,无文件来源区分
```

---

## 2. 数据模型

### 2.1 FileSegment

一个已加载的 BLF 文件段。

```rust
pub struct FileSegment {
    pub file_id: u32,                          // 自增 id,用于移除/管理
    pub path: PathBuf,
    pub file_name: String,                     // 用于 StatusBar/popover 显示
    pub start_time: Option<NaiveDateTime>,     // 该文件 measurement_start_time
    pub messages: Arc<[LogObject]>,            // 该文件解析后的全部消息(按文件内时间序)
    pub errors: Vec<String>,                   // 解析错误(保留 popover 显示)
    pub bytes_total: u64,
    pub bytes_consumed: u64,
    pub object_count: usize,
    pub time_min: Option<f64>,                 // 全局秒,该文件最早消息时间
    pub time_max: Option<f64>,                 // 全局秒,该文件最晚消息时间
}
```

- `file_id` 由全局 `AtomicU32` 计数器生成,从 1 开始,跨 session 不持久化
- `messages` 用 `Arc<[LogObject]>` 共享 segment 内部消息
- `time_min`/`time_max` 用全局秒(相对于 UNIX epoch),用于排序和绘图。计算公式:`global_seconds = (measurement_start_time - UNIX_EPOCH).num_seconds() as f64 + object_time_stamp / 1_000_000_000.0`

### 2.2 MergedView

全局合并视图(惰性构建、缓存、Arc 共享)。

```rust
pub struct MergedView {
    /// 拼接后的全局消息视图(按时间升序),每条 LogObject 是从各 segment 借用的克隆
    pub messages: Arc<[LogObject]>,
    /// 每条消息来自哪个 file_id,长度与 messages 相同,用于 popover/调试溯源(不用于绘图)
    pub source_file_ids: Arc<[u32]>,
    pub time_min: Option<f64>,
    pub time_max: Option<f64>,
    pub version: u64,                          // segment 集合变化时 +1,用于缓存失效
}
```

- `merged.messages` 严格按全局绝对时间升序
- 时间戳相同时按 `(file_id, msg_idx)` 稳定排序
- `version` 用于让 plot_data 缓存判断是否需要重算
- `source_file_ids[i]` 记录 `messages[i]` 来自哪个 segment,用于文件管理 popover 中"移除某文件后定位受影响消息"。**绘图不使用此字段**(按需求"不区分来源")

**关于"不复制 LogObject"的澄清:**

`LogObject` 是 `#[derive(Clone)]` 的枚举,合并时 `merged.messages` 中的每条消息是从各 segment 克隆过来的。`Arc<[LogObject]>` 共享的是合并后数组本身(多个视图共享),**不是跨 segment 共享单个 LogObject**。这意味着:

- 单文件加载:0 拷贝(segment.messages 直接作为 merged.messages)
- 多文件加载:N 个 segment 的 messages 逐条 clone 进 merged.messages(O(总消息数) 一次性拷贝)
- 移除一个文件:重建 merged,从剩余 segment 重新 clone(O(剩余消息数))

这是性能取舍:选择简单可控的全局数组,而非引用图。若消息总量 > 5M,考虑后续优化为 `Vec<Arc<LogObject>>` 共享单条消息。

### 2.3 CanViewApp 新增字段

```rust
pub struct CanViewApp {
    // 新增
    pub files: Vec<Arc<FileSegment>>,
    pub merged: MergedView,
    pub show_files_popover: bool,              // 控制文件管理弹框
    pub loading_progress: Option<LoadingProgress>,  // None=未加载,Some=加载中

    // 保留(语义变更)
    pub current_file_name: Option<String>,    // 改为"最近加载的文件名"
    pub blf_bytes_total: u64,                  // 改为汇总所有文件总和
    pub blf_bytes_consumed: u64,               // 改为汇总所有文件总和
    pub blf_parse_errors: Vec<String>,        // 改为当前 popover 选中文件的错误

    // 保留(不变)
    pub messages: Vec<LogObject>,              // 改为 merged.messages.to_vec() 兼容快照
    pub plot_data: Arc<[Series]>,
    pub plot_full_data: Arc<[Series]>,
    pub selected_signals: Vec<String>,
    pub start_time: Option<NaiveDateTime>,
    pub dbc_channels: HashMap<u16, DbcDatabase>,
    pub ldf_channels: HashMap<u16, LdfDatabase>,
    // ... 其余字段不变

    // 删除
    // pub is_streaming_mode: bool,
}

pub struct LoadingProgress {
    pub total_files: usize,
    pub completed_files: usize,
    pub current_file_name: Option<String>,
    pub total_messages_so_far: usize,
    pub is_cancelled: bool,
}
```

### 2.4 关键不变量

- `merged.messages` 严格按全局绝对时间升序(时间戳相同时按 (file_id, msg_idx) 稳定排序)
- `plot_data`/`plot_full_data` 从 `merged.messages` 解码,**不区分文件来源**
- 移除一个文件 = `files.retain(|f| f.file_id != id)` + 重建 `merged`(O(N) 合并,N = 总消息数)
- `files` 中失败文件 `messages` 为空,`errors` 非空,仍占位用于 popover 显示

---

## 3. 加载入口与流程

### 3.1 两个菜单入口

| 菜单项 | rfd 调用 | 语义 |
|---|---|---|
| `File → Open BLF...`(现有) | `pick_file()` 单选 | **替换**:清空 `files`、`merged`、`plot_data`、`selected_signals`,然后加载新文件 |
| `File → Open Multiple BLF...`(新增) | `pick_files()` 多选 | **追加**:不清空,把每个文件解析后追加进 `files`,重建 `merged` |

实现位置:`src/view/src/app/impls_rendering.rs:1880` 附近 File dropdown,在现有 `Open BLF...` child 下方追加第二个 child,结构完全镜像,差别只在 `pick_file()` → `pick_files()` 和 mode 标记。

### 3.2 共用加载流程

1. UI 调用 `LoadBlfFiles { paths: Vec<PathBuf>, mode: Append|Replace }`
2. 根据 mode 决定是否清空 `files`(Replace 清空,Append 不清空)
3. 对每个 path spawn 一个后台任务(`background_executor().spawn`)调用 `read_blf_from_file`,**并发执行**
4. 每个任务完成后通过 `cx.update` 把 `FileSegment` 推进 `files` 并触发 `merged` 重建
5. 流式更新:
   - StatusBar 显示 `Loading N/M files... (file_name)`
   - 每个文件完成后立即更新:`Loaded 2/3 → messages: 12,345`
   - **`merged` 每个文件完成就增量重建一次**,UI 在加载过程中已能看到部分数据
6. 所有完成后 StatusBar 显示 `✅ Loaded N files, total M messages`
7. 解析失败的单个文件:保留在 `files` 中带 `errors`,但 `messages` 为空,UI 文件管理弹框标红;**不影响其他文件加载**

### 3.3 单选 vs 多选的额外考虑

- 单选走 Replace 路径,等价于"清空 + 加载 1 个文件"
- 现有 `apply_blf_result` 改名为 `apply_blf_result_single`(内部调用 Replace 路径),保留对外接口签名尽量不变,避免破坏现有控制器代码
- 多选入口 `apply_blf_results_append` 是新接口

---

## 4. 并行解析与流式加载性能保证

### 4.1 并发模型

GPUI 的 `background_executor().spawn` 基于 smol 异步执行器,天然并发。每个 `read_blf_from_file(path)` 是独立的 async 任务,多个任务同时跑,充分利用多核。

```rust
// 伪代码
let tasks: Vec<_> = paths.into_iter().map(|path| {
    let path = path.clone();
    cx.background_executor().spawn(async move {
        read_blf_from_file(&path)
    })
}).collect();

// 顺序 await + 增量更新(每完成一个就 cx.update 推进 files,重建 merged)
```

### 4.2 流式合并优化

- 首个文件完成 → 直接构建 `merged`(O(N),无合并开销,直接借用 segment.messages 的 clone)
- 第 K 个文件完成 → **双指针归并**:把新 segment 的 messages 与现有 `merged.messages` 做经典双指针归并(O(N+M)),生成新的 `merged.messages`,而非整体重排序(O((N+M) log(N+M)))
- 单文件内部消息已按时间排序(BLF 文件本身按时间写入),所以两两合并就是经典的有序数组归并
- 移除一个文件 → 从剩余 segments **重新构建** `merged`(不是从现有 merged 中删除该文件的消息,而是用剩余 segments 重新归并,O(剩余消息数 log 剩余文件数) 或顺序归并 O(剩余消息数 × 剩余文件数) 取决于实现;简单实现用顺序归并即可,因为文件数通常 < 20)

### 4.3 大文件保护

- 单个 BLF 文件 > 100MB 时,解析过程中分块回调进度(已有 `bytes_consumed` 机制),UI 显示进度条
- 若内存压力(`messages.len() > 5_000_000`):加载完所有文件后做一次 `Arc::from(messages_vec)` 让 Vec 退化为切片引用,释放多余 capacity

### 4.4 取消语义

- 用户在加载过程中点击"Cancel"或新打开文件 → 通过 `LoadingProgress.is_cancelled` 标记
- 已完成的 segment 保留(Append)或被新 Replace 替换(Replace 模式中 Cancel 直接返回 Idle)
- 未完成的 future 通过 GPUI `Task` 的 drop 自动停止(GPUI Task 是可取消的)

### 4.5 性能预期

- 3 个 50MB BLF 文件并发解析:≈ 单文件解析时间的 1.2-1.5 倍(vs 顺序解析的 3 倍)
- 流式追加:用户在加载开始 ~1 秒后就能看到第一个文件的数据

---

## 5. UI 改动

### 5.1 File 菜单

```
File
├── Open BLF...            (单选,替换)  ← 保留
└── Open Multiple BLF...   (多选,追加)  ← 新增
```

### 5.2 StatusBar 文件管理入口

当前 StatusBar 显示 `current_file_name` 和 `blf_bytes_*`。改为:

```
[✅ Loaded 3 files, 1.2M messages] [📁 Files ▾]  ← 点击展开文件管理弹框
```

- 左侧 status_msg 改为多文件汇总
- 右侧新增 `📁 Files` 按钮,点击展开 popover
- Loading 状态下显示 `[Cancel]` 按钮替代 `📁 Files`

### 5.3 文件管理 Popover(新增)

```
┌─ Loaded Files ─────────────────────────────────┐
│  ● session1.blf      12,345 msgs   3.2 MB   ✅   │
│  ● session2.blf       8,901 msgs   2.1 MB   ✅   │
│  ● bad.blf               0 msgs   1.5 MB   ❌   │  ← 标红,hover 显示错误
│                                                  │
│  [Remove All] [Done]                              │
└──────────────────────────────────────────────────┘
```

- 每行:文件名、消息数、文件大小、状态(✅/❌)
- 右侧每行一个 ✕ 按钮可单独移除(点击后重建 merged)
- `Remove All` 清空所有
- `Done` 关闭弹框
- 错误状态行:点击展开错误详情(复用现有 `blf_parse_errors` popover 机制)

### 5.4 LogView 与 Signal Plot

按需求"折线图 + 日志列表可滚动",不引入播放头/时间游标:
- LogView 现有 `UniformListScrollHandle` 不变,仍从 `messages` 渲染
- 现有筛选(ID/channel/signal)继续作用于合并后的 `messages`
- 折线图:`plot_data`/`plot_full_data` 现有逻辑不变,新增多文件信号时直接拼接(不区分来源)

---

## 6. 加载命令与状态机

### 6.1 新增命令结构

```rust
// src/view/src/app/commands/load.rs

pub struct LoadBlfFiles {
    pub paths: Vec<PathBuf>,
    pub mode: LoadMode,
}

pub enum LoadMode {
    Replace,  // Open BLF... 单选
    Append,   // Open Multiple BLF... 多选
}

pub struct RemoveFile {
    pub file_id: u32,
}

pub struct RemoveAllFiles;
```

### 6.2 apply 流程拆分

现有 `apply_blf_result` 拆为:

```rust
impl CanViewApp {
    /// 单文件 Replace:清空 files,加入单个 segment,重建 merged
    pub fn apply_blf_result_single(&mut self, result: BlfResult, path: PathBuf, cx: &mut Context<Self>);

    /// 多文件 Append:从初始空状态或现有状态追加 N 个 segment
    pub fn apply_blf_results_append(&mut self, results: Vec<Result<BlfResult, Error>>, paths: Vec<PathBuf>, cx: &mut Context<Self>);

    /// 流式:单个文件解析完成时调用
    pub fn apply_blf_result_append_one(&mut self, result: Result<BlfResult, Error>, path: PathBuf, cx: &mut Context<Self>);

    /// 移除单个文件
    pub fn remove_file(&mut self, file_id: u32, cx: &mut Context<Self>);

    /// 移除所有文件
    pub fn remove_all_files(&mut self, cx: &mut Context<Self>);

    /// 重建 merged(所有 file 增删后调用)
    fn rebuild_merged(&mut self);
}
```

### 6.3 状态机

```
                ┌─────────────┐
                │   Idle      │ ← 无文件加载
                └──────┬──────┘
                       │ Open BLF.../Open Multiple BLF...
                       ▼
                ┌─────────────┐
       ┌───────►│  Loading    │──── Cancel ───┐
       │        │             │               │
       │        └──────┬──────┘               ▼
       │               │ All tasks done  ┌─────────────┐
       │               ▼                 │  Cancelled  │
       │        ┌─────────────┐          └─────────────┘
       │        │   Loaded    │
       │        │             │
       │        └──────┬──────┘
       │               │ Remove All / Open BLF...(Replace)
       └───────────────┘
```

- Loading 状态下 StatusBar 显示 `[Cancel]` 按钮
- 点击 Cancel → `loading_progress.is_cancelled = true`
- 已完成 segment 保留(Append)或被新 Replace 替换(Replace 模式中 Cancel 直接返回 Idle)
- 未完成的 future 通过 GPUI `Task` 的 drop 自动停止

### 6.4 与现有 `apply_blf_result` 的兼容

- 现有调用点:`impls_rendering.rs:1942` `view.apply_blf_result(result, fname);` 改为 `view.apply_blf_result_single(result?, path, cx);`
- `config_controller.rs:148` `apply_blf_result(app, result)` 同步改为 `apply_blf_result_single`
- 删除旧 `apply_blf_result`,编译期发现所有调用点

### 6.5 file_id 生成

- `AtomicU32` 全局计数器,从 1 开始
- 跨 session 不持久化(重启重置)
- 用于 `RemoveFile` 命令定位

---

## 7. 错误处理与边界

### 7.1 单个文件解析失败的隔离原则

- 一个文件失败**不阻塞**其他文件加载(Append 模式)
- 失败文件进入 `files`,状态为 `errors: Vec<String>` + `messages: Arc::from([])`
- StatusBar 汇总:`⚠️ Loaded 2/3 files (1 failed) — click Files to inspect`
- 文件管理 popover 中失败行标红,hover 显示 `❌ File Error: <error>`

### 7.2 Replace 模式下的失败

- 单选 Replace:新文件失败 → **不清空**之前的文件,保留旧数据,StatusBar 显示 `❌ File Error: <e>`(沿用现有 `apply_blf_result` 错误分支语义)
- 这是现有行为,不破坏用户预期

### 7.3 文件读取错误分类

| 错误类型 | 处理 |
|---|---|
| 文件不存在/权限 | 加入 `files`,标红,记录错误 |
| BLF 格式损坏 | `BlfResult.errors` 非空但 `objects` 非空 → 正常加载,popover 显示 parse errors(沿用现有机制) |
| BLF 完全无法解析(0 objects + errors) | 加入 `files` 标红,但 messages 为空 |
| 文件过大触发 OOM 风险 | 加载前先检查 `fs::metadata().len()`,超过阈值(如 1GB)弹确认对话框 |

### 7.4 路径冲突

- 同一文件路径被多次加载(用户在多选中重复选了,或追加时已存在)→ 仍允许,每个 segment 独立
- popover 中显示重复文件名时不做特殊处理(用 file_id 区分)

### 7.5 配置持久化(multi_channel_config.json)

- 现有配置文件**不保存**已加载的 BLF 文件路径(沿用现状)
- 重启后 `files` 为空,用户需重新打开
- 这与现有"打开文件 = 用户主动选择"的语义一致

### 7.6 is_streaming_mode 字段清理

- 删除 `CanViewApp.is_streaming_mode` 及 `RuntimeState.is_streaming_mode`
- 移除 `save_runtime_state`/`restore_runtime_state` 中的对应字段
- 编译期发现所有引用

### 7.7 RuntimeState 持久化(窗口操作间)

`RuntimeState` 需要新增 `files` 和 `merged` 字段,确保 maximize/restore 窗口操作不丢失数据:

```rust
pub struct RuntimeState {
    pub current_view: AppView,
    pub files: Vec<Arc<FileSegment>>,  // 新增
    pub merged: MergedView,            // 新增
    // 删除:messages(由 merged.messages 派生), is_streaming_mode
    // current_file_name 保留(CanViewApp 字段,不进 RuntimeState,因为可由 files[0] 派生)
    pub current_file_name: Option<String>,  // 保留,显示"最近加载的文件名"
    pub plot_data: Arc<[Series]>,
    pub plot_full_data: Arc<[Series]>,
    pub plot_zoom_start: Option<f64>,
    pub plot_zoom_end: Option<f64>,
    pub selected_signals: Vec<String>,
    pub dbc_channels: HashMap<u16, DbcDatabase>,
    pub ldf_channels: HashMap<u16, LdfDatabase>,
    pub start_time: Option<NaiveDateTime>,
    pub active_library_id: Option<String>,
    pub active_version_name: Option<String>,
}
```

注意:`current_file_name` 仍保留在 `RuntimeState` 中,因为它表示"最近加载的文件名",用于窗口操作间保持 StatusBar 短摘要显示。`messages` 字段从 `RuntimeState` 删除,因为窗口恢复时可由 `merged.messages` 重新派生为 `CanViewApp.messages` 兼容快照。

---

## 8. 测试策略

### 8.1 单元测试

- `FileSegment` 构造:从 `BlfResult` 转换,正确填充 `time_min`/`time_max`/`object_count`/`errors`
- `MergedView` 重建:
  - 0 个 segment → 空 messages、`time_min=None`
  - 1 个 segment → 直接拷贝,时间范围正确
  - 2 个 segment,时间重叠 → 按 (timestamp, file_id, msg_idx) 稳定排序
  - 2 个 segment,时间不重叠 → 拼接顺序正确
  - 移除中间 segment → 重建后剩余 segments 顺序正确
- `LoadMode`:
  - Replace 清空现有 files 后加入
  - Append 不清空,新文件追加到末尾
- 命令解析:`LoadBlfFiles::new(paths, mode)` 路径与 mode 正确传递

### 8.2 集成测试(tests/ 目录)

- 多文件加载:构造 2 个测试 BLF(已有 `sample.blf`,再生成一个),验证合并后 messages 按时间全局排序
- 流式加载:mock 单文件完成回调,验证 `merged` 增量更新
- 失败隔离:1 个有效 BLF + 1 个损坏 BLF,验证有效文件仍正常加载
- 移除文件:加载 3 个文件后移除中间 1 个,验证剩余 2 个的 merged 正确

### 8.3 手动 UI 验证清单

1. 单选 Open BLF... → 替换已有数据,UI 正常
2. 多选 Open Multiple BLF... → 追加,StatusBar 显示流式进度
3. 加载中点击 Cancel → 已完成文件保留,未完成取消
4. 加载 3 个文件,移除中间 1 个 → 折线图/日志列表立即更新
5. 多选其中 1 个文件损坏 → 其他正常加载,popover 标红
6. 多选重复同一文件 → 两个 segment 共存
7. 5MB BLF × 10 并发加载 → 内存稳定,UI 流畅
8. 50MB BLF × 3 并发加载 → 完成时间 ≈ 单文件 1.5 倍
9. LogView 滚动 → 多文件消息按时间混合显示,筛选正常
10. Signal Plot → 同名信号多文件拼接为一条波形
11. 窗口 maximize/restore → files/merged/plot 状态完整保留

### 8.4 回归测试

- 现有 `cargo test --workspace` 全部通过
- 现有 BLF 解析、DBC/LDF 解析、信号解码、绘图相关测试不受影响
- 新增测试覆盖 `FileSegment`/`MergedView`/`LoadMode`/`LoadBlfFiles`

### 8.5 性能基准(可选)

- 加载 1×50MB vs 5×10MB vs 25×2MB 完成时间对比
- 内存峰值对比
- 用 `cargo bench` 或简单 `Instant::now()` 计时

---

## 9. 受影响文件清单

### 新增
- `src/view/src/domain/multi_file.rs` — `FileSegment`/`MergedView` 结构 + `rebuild_merged` 逻辑(放在 domain 层,与 `log_processor.rs` 同级)
- `src/view/src/app/commands/multi_file.rs` — `LoadBlfFiles`/`LoadMode`/`RemoveFile`/`RemoveAllFiles` 命令结构(放在 commands 目录,与 `load.rs` 同级)

### 修改
- `src/view/src/app/state.rs` — 新增 `FileSegment`/`MergedView`/`LoadingProgress` 结构,`CanViewApp` 新增 `files`/`merged`/`show_files_popover`/`loading_progress` 字段,删除 `is_streaming_mode`;`RuntimeState` 新增 `files`/`merged`,删除 `messages`/`is_streaming_mode`(`current_file_name` 保留)
- `src/view/src/app/commands/load.rs` — 新增 `LoadBlfFiles`/`LoadMode`/`RemoveFile`/`RemoveAllFiles`
- `src/view/src/app/impls.rs` — `apply_blf_result` → `apply_blf_result_single`;新增 `apply_blf_results_append`/`apply_blf_result_append_one`/`remove_file`/`remove_all_files`/`rebuild_merged`;`save_runtime_state`/`restore_runtime_state` 适配新字段
- `src/view/src/app/impls_rendering.rs` — File 菜单新增 `Open Multiple BLF...` child;StatusBar 新增 `📁 Files` 按钮和 `Cancel` 按钮;新增文件管理 popover 渲染
- `src/view/src/controllers/config_controller.rs` — `apply_blf_result` 调用点改为 `apply_blf_result_single`

### 不变
- `src/blf/` — 整个 crate 不变,复用 `read_blf_from_file`
- `src/parser/` — DBC/LDF 解析不变
- `src/view/src/views/log_view.rs` — 渲染逻辑不变,从 `messages` 读
- `src/view/src/views/chart_view.rs` — 渲染逻辑不变,从 `plot_data` 读
- `src/view/src/domain/log_processor.rs` — 接口不变,从 `messages` 读

---

## 10. 实施顺序建议

1. **数据模型**:`FileSegment` + `MergedView` + 单元测试
2. **命令层**:`LoadBlfFiles`/`LoadMode`/`RemoveFile`/`RemoveAllFiles`
3. **apply 流程**:`apply_blf_result_single`/`apply_blf_result_append_one`/`rebuild_merged` + 单元测试
4. **UI:File 菜单**:新增 `Open Multiple BLF...` child,接通多选追加
5. **UI:StatusBar**:多文件汇总显示 + `📁 Files` 按钮 + `Cancel` 按钮
6. **UI:文件管理 popover**:渲染 + 移除/Remove All 接通
7. **RuntimeState 适配**:`files`/`merged` 跨窗口操作保留
8. **清理**:删除 `is_streaming_mode`,编译期发现所有引用
9. **集成测试**:多文件加载、流式、失败隔离、移除
10. **手动验证**:按 8.3 清单逐项验证
